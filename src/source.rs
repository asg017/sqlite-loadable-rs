//! A stable, C-ABI "source" API that lets one SQLite extension read remote
//! objects (HTTP, S3, …) through another extension, without either linking
//! the other or sharing Rust types across the cdylib boundary.
//!
//! A *producer* extension (e.g. `sqlite-fetch`, `sqlite-objectstore`) registers
//! a scalar function such as `_http_api()` that returns a
//! [`sqlite3_result_pointer`](https://www.sqlite.org/bindptr.html) whose
//! payload is a [`SourceApiRaw`] vtable (pointer type name
//! [`SOURCE_API_POINTER_NAME`]). A *consumer* extension (e.g. `sqlite-xsv`,
//! `sqlite-parquet`) calls [`resolve_source_api`] with the function name, gets
//! back a refcounted [`SourceHandle`], and uses [`SourceHandle::head`],
//! [`SourceHandle::get`] and [`SourceHandle::get_range`].
//!
//! The C struct layout is documented in `sqlite-source.h` at the crate root;
//! the two must be kept in sync (see the `layout` test at the bottom).
//!
//! Ownership rules (ABI v1):
//! - `ctx` is owned by the producer and refcounted via `retain`/`release`.
//!   It must be safe to use from any thread.
//! - Every `char *` / buffer handed to the consumer is allocated by the
//!   producer and must be returned through `free_string` (NUL-terminated
//!   strings: error messages, etag, content type) or `free_buffer`
//!   (`get_range` payloads, with their exact length).
//! - All operations return [`SOURCE_RC_OK`] on success; on failure they
//!   return non-zero and set `*errmsg` to a producer-allocated string.
//!   [`SOURCE_RC_CHANGED`] means an `if_match` precondition failed: the
//!   object no longer has the ETag the consumer read at `head()` time.
//! - `get` and `get_range` take an optional `if_match` ETag (NULL = none) so
//!   a consumer that spreads one logical read over many requests can detect
//!   the object changing underneath it.
//! - `sqlite3_result_pointer` frees the vtable struct when the producing
//!   statement is reset, so consumers copy the struct and `retain` the ctx
//!   while the statement is still live. [`resolve_source_api`] does this.

use std::ffi::{c_char, c_void, CStr, CString};
use std::fmt;
use std::io::Read;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use crate::api;
use crate::exec::Statement;
use crate::ext::{sqlite3, sqlite3_context};

/// Pointer type name passed to `sqlite3_result_pointer` / `sqlite3_value_pointer`.
pub const SOURCE_API_POINTER_NAME: &[u8] = b"sqlite-source-api-v1\0";
/// ABI version stored in [`SourceApiRaw::abi_version`].
pub const SOURCE_ABI_VERSION: u32 = 1;

/// Return codes for `head` / `get` / `get_range` / stream `read`.
pub const SOURCE_RC_OK: i32 = 0;
pub const SOURCE_RC_ERROR: i32 = 1;
/// The `if_match` ETag no longer matches the object.
pub const SOURCE_RC_CHANGED: i32 = 2;

/// Sentinel for "unknown size" in [`SourceMetaRaw::size`].
pub const SOURCE_SIZE_UNKNOWN: u64 = u64::MAX;
/// Sentinel for "unknown last-modified" in [`SourceMetaRaw::last_modified_ms`].
pub const SOURCE_LAST_MODIFIED_UNKNOWN: i64 = -1;

// ---------------------------------------------------------------------------
// C layout
// ---------------------------------------------------------------------------

/// Result of `head`. Mirrors `sqlite_source_meta` in `sqlite-source.h`.
#[repr(C)]
pub struct SourceMetaRaw {
    pub size: u64,
    pub last_modified_ms: i64,
    pub etag: *mut c_char,
    pub content_type: *mut c_char,
}

/// A streaming body. Mirrors `sqlite_source_stream` in `sqlite-source.h`.
#[repr(C)]
pub struct SourceStreamRaw {
    pub ctx: *mut c_void,
    /// Bytes read, `0` at EOF, `<0` on error (with `*errmsg` set).
    pub read: unsafe extern "C" fn(
        ctx: *mut c_void,
        buf: *mut u8,
        len: u64,
        errmsg: *mut *mut c_char,
    ) -> i64,
    /// Frees the stream (including `ctx` and the struct itself).
    pub close: unsafe extern "C" fn(stream: *mut SourceStreamRaw),
}

/// The vtable behind the pointer. Mirrors `sqlite_source_api` in `sqlite-source.h`.
#[repr(C)]
pub struct SourceApiRaw {
    pub abi_version: u32,
    pub struct_size: u32,
    pub ctx: *mut c_void,
    pub retain: unsafe extern "C" fn(ctx: *mut c_void),
    pub release: unsafe extern "C" fn(ctx: *mut c_void),
    pub head: unsafe extern "C" fn(
        ctx: *mut c_void,
        url: *const c_char,
        out: *mut SourceMetaRaw,
        errmsg: *mut *mut c_char,
    ) -> i32,
    pub get: unsafe extern "C" fn(
        ctx: *mut c_void,
        url: *const c_char,
        if_match: *const c_char,
        out: *mut *mut SourceStreamRaw,
        errmsg: *mut *mut c_char,
    ) -> i32,
    pub get_range: unsafe extern "C" fn(
        ctx: *mut c_void,
        url: *const c_char,
        start: u64,
        len: u64,
        if_match: *const c_char,
        buf: *mut *mut u8,
        buf_len: *mut u64,
        errmsg: *mut *mut c_char,
    ) -> i32,
    pub free_string: unsafe extern "C" fn(ctx: *mut c_void, s: *mut c_char),
    pub free_buffer: unsafe extern "C" fn(ctx: *mut c_void, p: *mut u8, len: u64),
    /// Reserved for a future `list(prefix)`; always NULL in ABI v1.
    pub list: *mut c_void,
}

// ---------------------------------------------------------------------------
// Shared Rust types
// ---------------------------------------------------------------------------

/// Metadata about a remote object.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceMeta {
    pub size: Option<u64>,
    pub last_modified_ms: Option<i64>,
    pub etag: Option<String>,
    pub content_type: Option<String>,
}

/// Errors from resolving or using a source API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceError {
    /// The resolver function (e.g. `_http_api`) does not exist on this connection.
    NotLoaded { function: String },
    /// Preparing/stepping the resolver statement failed for another reason.
    Resolve { function: String, message: String },
    /// The function returned something that isn't a `sqlite-source-api-v1` pointer.
    NotASourceApi { function: String },
    /// The producer speaks a different ABI version.
    AbiMismatch { function: String, found: u32 },
    /// An `if_match` precondition failed: the object was modified since the
    /// consumer read its ETag.
    Changed { url: String, expected: String },
    /// An error reported by the producer (network, auth, 404, …) or a Rust I/O error.
    Message(String),
}

impl SourceError {
    /// The message users should see. `scheme` is used to make the
    /// "not loaded" error actionable, e.g. `"https"`.
    pub fn user_message(&self, scheme: Option<&str>) -> String {
        match self {
            SourceError::NotLoaded { function } => match scheme {
                Some(s) => format!(
                    "no source extension loaded for \"{}://\" URLs (expected a {}() function)",
                    s, function
                ),
                None => format!("no source extension loaded (expected a {}() function)", function),
            },
            SourceError::Resolve { function, message } => {
                format!("could not resolve source API {}(): {}", function, message)
            }
            SourceError::NotASourceApi { function } => format!(
                "{}() did not return a {} pointer",
                function,
                pointer_name_str()
            ),
            SourceError::AbiMismatch { function, found } => format!(
                "{}() returned source ABI version {} but this extension expects {}",
                function, found, SOURCE_ABI_VERSION
            ),
            SourceError::Changed { url, expected } => format!(
                "{} changed since it was opened (ETag {} no longer matches); \
                 re-create the virtual table to read the new version",
                url, expected
            ),
            SourceError::Message(m) => m.clone(),
        }
    }
}

impl fmt::Display for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.user_message(None))
    }
}
impl std::error::Error for SourceError {}

impl From<String> for SourceError {
    fn from(s: String) -> Self {
        SourceError::Message(s)
    }
}
impl From<&str> for SourceError {
    fn from(s: &str) -> Self {
        SourceError::Message(s.to_owned())
    }
}
impl From<std::io::Error> for SourceError {
    fn from(e: std::io::Error) -> Self {
        SourceError::Message(e.to_string())
    }
}
impl From<SourceError> for crate::Error {
    fn from(e: SourceError) -> Self {
        crate::Error::new_message(e.user_message(None))
    }
}
impl From<SourceError> for std::io::Error {
    fn from(e: SourceError) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, e.user_message(None))
    }
}

pub type SourceResult<T> = Result<T, SourceError>;

fn pointer_name_str() -> &'static str {
    std::str::from_utf8(&SOURCE_API_POINTER_NAME[..SOURCE_API_POINTER_NAME.len() - 1]).unwrap()
}

// ---------------------------------------------------------------------------
// Scheme dispatch (hard-coded allowlist, by design)
// ---------------------------------------------------------------------------

/// A URL scheme that has a known source provider function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceScheme {
    /// e.g. `"https"`
    pub scheme: &'static str,
    /// e.g. `"_http_api"`
    pub function: &'static str,
}

const SCHEMES: &[SourceScheme] = &[
    SourceScheme { scheme: "http", function: "_http_api" },
    SourceScheme { scheme: "https", function: "_http_api" },
    SourceScheme { scheme: "s3", function: "_s3_api" },
];

/// If `source` starts with a supported `scheme://`, return the scheme and the
/// provider function to resolve. Local paths (including Windows drive letters
/// like `C:\…`) and unknown schemes return `None`.
pub fn source_function_for(source: &str) -> Option<SourceScheme> {
    let (scheme, _) = source.split_once("://")?;
    SCHEMES
        .iter()
        .find(|s| s.scheme.eq_ignore_ascii_case(scheme))
        .copied()
}

/// True if `source` looks like a URL with *any* scheme (`xyz://…`), whether or
/// not a provider is known. Useful for "unsupported scheme" errors.
pub fn looks_like_url(source: &str) -> bool {
    match source.split_once("://") {
        Some((scheme, _)) => {
            !scheme.is_empty()
                && scheme
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'+' || b == b'-' || b == b'.')
        }
        None => false,
    }
}

// ---------------------------------------------------------------------------
// Producer side
// ---------------------------------------------------------------------------

/// Implemented by source providers. Must be thread-safe: consumers may use the
/// handle from any thread and from several cursors at once.
pub trait SourceApi: Send + Sync + 'static {
    fn head(&self, url: &str) -> SourceResult<SourceMeta>;
    /// Stream the whole object. If `if_match` is given and the object's ETag
    /// differs, return [`SourceError::Changed`].
    fn get(&self, url: &str, if_match: Option<&str>) -> SourceResult<Box<dyn Read + Send>>;
    /// Read `len` bytes at `start` (fewer only at EOF). Same `if_match` rule.
    fn get_range(
        &self,
        url: &str,
        start: u64,
        len: u64,
        if_match: Option<&str>,
    ) -> SourceResult<Vec<u8>>;
}

/// The thing `ctx` points at on the producer side. Sized so the Arc pointer is thin.
struct ProducerCtx {
    api: Box<dyn SourceApi>,
}

/// Return `api` from a scalar function as a `sqlite-source-api-v1` pointer.
///
/// ```ignore
/// fn http_api(context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
///     source::result_source_api(context, Arc::new(HttpApi::new(...)));
///     Ok(())
/// }
/// ```
pub fn result_source_api<T: SourceApi>(context: *mut sqlite3_context, api: T) {
    let ctx: Arc<ProducerCtx> = Arc::new(ProducerCtx { api: Box::new(api) });
    let raw = SourceApiRaw {
        abi_version: SOURCE_ABI_VERSION,
        struct_size: std::mem::size_of::<SourceApiRaw>() as u32,
        ctx: Arc::into_raw(ctx) as *mut c_void,
        retain: producer_retain,
        release: producer_release,
        head: producer_head,
        get: producer_get,
        get_range: producer_get_range,
        free_string: producer_free_string,
        free_buffer: producer_free_buffer,
        list: std::ptr::null_mut(),
    };
    // Boxed struct owns one reference to ctx; released by the destructor
    // SQLite calls when the value is freed.
    api::result_pointer_with_destructor(
        context,
        SOURCE_API_POINTER_NAME,
        raw,
        Some(producer_struct_destroy),
    );
}

unsafe extern "C" fn producer_struct_destroy(p: *mut c_void) {
    let raw: Box<SourceApiRaw> = Box::from_raw(p.cast());
    (raw.release)(raw.ctx);
}

unsafe fn ctx_api<'a>(ctx: *mut c_void) -> &'a dyn SourceApi {
    &*(*(ctx as *const ProducerCtx)).api
}

fn set_errmsg(errmsg: *mut *mut c_char, msg: impl AsRef<str>) {
    if errmsg.is_null() {
        return;
    }
    let msg = msg.as_ref().replace('\0', " ");
    let c = CString::new(msg).unwrap_or_else(|_| CString::new("error").unwrap());
    unsafe { *errmsg = c.into_raw() };
}

fn panic_message(e: Box<dyn std::any::Any + Send>) -> String {
    let inner = if let Some(s) = e.downcast_ref::<&str>() {
        (*s).to_owned()
    } else if let Some(s) = e.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic".to_owned()
    };
    format!("panic in source provider: {}", inner)
}

unsafe extern "C" fn producer_retain(ctx: *mut c_void) {
    Arc::increment_strong_count(ctx as *const ProducerCtx);
}
unsafe extern "C" fn producer_release(ctx: *mut c_void) {
    Arc::decrement_strong_count(ctx as *const ProducerCtx);
}

unsafe fn url_from_c<'a>(url: *const c_char) -> Result<&'a str, String> {
    if url.is_null() {
        return Err("url is NULL".into());
    }
    CStr::from_ptr(url)
        .to_str()
        .map_err(|_| "url is not valid UTF-8".to_owned())
}

unsafe fn opt_str_from_c<'a>(p: *const c_char) -> Result<Option<&'a str>, String> {
    if p.is_null() {
        return Ok(None);
    }
    CStr::from_ptr(p)
        .to_str()
        .map(Some)
        .map_err(|_| "if_match is not valid UTF-8".to_owned())
}

/// Map a producer-side error to (return code, message).
fn error_rc(e: SourceError) -> (i32, String) {
    match &e {
        SourceError::Changed { .. } => (SOURCE_RC_CHANGED, e.user_message(None)),
        _ => (SOURCE_RC_ERROR, e.user_message(None)),
    }
}

fn opt_cstring(s: Option<String>) -> *mut c_char {
    match s {
        Some(s) => CString::new(s.replace('\0', ""))
            .map(|c| c.into_raw())
            .unwrap_or(std::ptr::null_mut()),
        None => std::ptr::null_mut(),
    }
}

unsafe extern "C" fn producer_head(
    ctx: *mut c_void,
    url: *const c_char,
    out: *mut SourceMetaRaw,
    errmsg: *mut *mut c_char,
) -> i32 {
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<SourceMeta, String> {
        let url = url_from_c(url)?;
        ctx_api(ctx).head(url).map_err(|e| e.user_message(None))
    }));
    match result {
        Ok(Ok(meta)) => {
            if out.is_null() {
                set_errmsg(errmsg, "out is NULL");
                return 1;
            }
            *out = SourceMetaRaw {
                size: meta.size.unwrap_or(SOURCE_SIZE_UNKNOWN),
                last_modified_ms: meta.last_modified_ms.unwrap_or(SOURCE_LAST_MODIFIED_UNKNOWN),
                etag: opt_cstring(meta.etag),
                content_type: opt_cstring(meta.content_type),
            };
            0
        }
        Ok(Err(msg)) => {
            set_errmsg(errmsg, msg);
            1
        }
        Err(e) => {
            set_errmsg(errmsg, panic_message(e));
            1
        }
    }
}

struct ProducerStream {
    reader: Box<dyn Read + Send>,
}

unsafe extern "C" fn producer_stream_read(
    ctx: *mut c_void,
    buf: *mut u8,
    len: u64,
    errmsg: *mut *mut c_char,
) -> i64 {
    let stream = &mut *(ctx as *mut ProducerStream);
    let len = usize::try_from(len).unwrap_or(usize::MAX);
    let slice = std::slice::from_raw_parts_mut(buf, len);
    let result = catch_unwind(AssertUnwindSafe(|| stream.reader.read(slice)));
    match result {
        Ok(Ok(n)) => n as i64,
        Ok(Err(e)) => {
            set_errmsg(errmsg, e.to_string());
            -1
        }
        Err(e) => {
            set_errmsg(errmsg, panic_message(e));
            -1
        }
    }
}

unsafe extern "C" fn producer_stream_close(stream: *mut SourceStreamRaw) {
    if stream.is_null() {
        return;
    }
    let stream: Box<SourceStreamRaw> = Box::from_raw(stream);
    let _ = catch_unwind(AssertUnwindSafe(|| {
        drop(Box::from_raw(stream.ctx as *mut ProducerStream));
    }));
}

unsafe extern "C" fn producer_get(
    ctx: *mut c_void,
    url: *const c_char,
    if_match: *const c_char,
    out: *mut *mut SourceStreamRaw,
    errmsg: *mut *mut c_char,
) -> i32 {
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<Box<dyn Read + Send>, (i32, String)> {
        let url = url_from_c(url).map_err(|m| (SOURCE_RC_ERROR, m))?;
        let if_match = opt_str_from_c(if_match).map_err(|m| (SOURCE_RC_ERROR, m))?;
        ctx_api(ctx).get(url, if_match).map_err(error_rc)
    }));
    match result {
        Ok(Ok(reader)) => {
            if out.is_null() {
                set_errmsg(errmsg, "out is NULL");
                return 1;
            }
            let stream = Box::new(ProducerStream { reader });
            let raw = Box::new(SourceStreamRaw {
                ctx: Box::into_raw(stream) as *mut c_void,
                read: producer_stream_read,
                close: producer_stream_close,
            });
            *out = Box::into_raw(raw);
            0
        }
        Ok(Err((rc, msg))) => {
            set_errmsg(errmsg, msg);
            rc
        }
        Err(e) => {
            set_errmsg(errmsg, panic_message(e));
            1
        }
    }
}

unsafe extern "C" fn producer_get_range(
    ctx: *mut c_void,
    url: *const c_char,
    start: u64,
    len: u64,
    if_match: *const c_char,
    buf: *mut *mut u8,
    buf_len: *mut u64,
    errmsg: *mut *mut c_char,
) -> i32 {
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<Vec<u8>, (i32, String)> {
        let url = url_from_c(url).map_err(|m| (SOURCE_RC_ERROR, m))?;
        let if_match = opt_str_from_c(if_match).map_err(|m| (SOURCE_RC_ERROR, m))?;
        ctx_api(ctx).get_range(url, start, len, if_match).map_err(error_rc)
    }));
    match result {
        Ok(Ok(bytes)) => {
            if buf.is_null() || buf_len.is_null() {
                set_errmsg(errmsg, "out is NULL");
                return 1;
            }
            let boxed: Box<[u8]> = bytes.into_boxed_slice();
            let n = boxed.len() as u64;
            *buf = Box::into_raw(boxed) as *mut u8;
            *buf_len = n;
            0
        }
        Ok(Err((rc, msg))) => {
            set_errmsg(errmsg, msg);
            rc
        }
        Err(e) => {
            set_errmsg(errmsg, panic_message(e));
            1
        }
    }
}

unsafe extern "C" fn producer_free_string(_ctx: *mut c_void, s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

unsafe extern "C" fn producer_free_buffer(_ctx: *mut c_void, p: *mut u8, len: u64) {
    if !p.is_null() {
        let slice = std::ptr::slice_from_raw_parts_mut(p, len as usize);
        drop(Box::from_raw(slice));
    }
}

// ---------------------------------------------------------------------------
// Consumer side
// ---------------------------------------------------------------------------

/// A refcounted handle to a producer's API. `Clone` retains, `Drop` releases.
pub struct SourceHandle {
    raw: SourceApiRaw,
}

// Contract: producers guarantee ctx is usable from any thread.
unsafe impl Send for SourceHandle {}
unsafe impl Sync for SourceHandle {}

impl Clone for SourceHandle {
    fn clone(&self) -> Self {
        unsafe { (self.raw.retain)(self.raw.ctx) };
        SourceHandle { raw: copy_raw(&self.raw) }
    }
}

impl Drop for SourceHandle {
    fn drop(&mut self) {
        unsafe { (self.raw.release)(self.raw.ctx) };
    }
}

impl fmt::Debug for SourceHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SourceHandle")
            .field("abi_version", &self.raw.abi_version)
            .field("ctx", &self.raw.ctx)
            .finish()
    }
}

fn copy_raw(raw: &SourceApiRaw) -> SourceApiRaw {
    // SourceApiRaw is plain data; a bitwise copy is the intended way to hold it.
    unsafe { std::ptr::read(raw) }
}

impl SourceHandle {
    /// Take ownership of a vtable pointer obtained from `sqlite3_value_pointer`.
    /// Validates the ABI version and retains `ctx`. `function` is only used in
    /// error messages.
    ///
    /// # Safety
    /// `p` must point at a live `SourceApiRaw` (i.e. the producing statement
    /// has not been reset/finalized yet).
    pub unsafe fn from_raw_ptr(p: *const SourceApiRaw, function: &str) -> SourceResult<Self> {
        if p.is_null() {
            return Err(SourceError::NotASourceApi { function: function.to_owned() });
        }
        let abi_version = (*p).abi_version;
        if abi_version != SOURCE_ABI_VERSION {
            return Err(SourceError::AbiMismatch { function: function.to_owned(), found: abi_version });
        }
        if ((*p).struct_size as usize) < std::mem::size_of::<SourceApiRaw>() {
            return Err(SourceError::AbiMismatch { function: function.to_owned(), found: abi_version });
        }
        let raw = copy_raw(&*p);
        (raw.retain)(raw.ctx);
        Ok(SourceHandle { raw })
    }

    fn take_errmsg(&self, errmsg: *mut c_char, fallback: &str) -> SourceError {
        self.take_error(SOURCE_RC_ERROR, errmsg, fallback, "", None)
    }

    /// Build the consumer-side error for a non-zero return code, freeing the
    /// producer's message.
    fn take_error(
        &self,
        rc: i32,
        errmsg: *mut c_char,
        fallback: &str,
        url: &str,
        if_match: Option<&str>,
    ) -> SourceError {
        let msg = if errmsg.is_null() {
            fallback.to_owned()
        } else {
            let m = unsafe { CStr::from_ptr(errmsg) }.to_string_lossy().into_owned();
            unsafe { (self.raw.free_string)(self.raw.ctx, errmsg) };
            m
        };
        if rc == SOURCE_RC_CHANGED {
            SourceError::Changed {
                url: url.to_owned(),
                expected: if_match.unwrap_or("").to_owned(),
            }
        } else {
            SourceError::Message(msg)
        }
    }

    fn take_string(&self, s: *mut c_char) -> Option<String> {
        if s.is_null() {
            return None;
        }
        let out = unsafe { CStr::from_ptr(s) }.to_string_lossy().into_owned();
        unsafe { (self.raw.free_string)(self.raw.ctx, s) };
        Some(out)
    }

    pub fn head(&self, url: &str) -> SourceResult<SourceMeta> {
        let c_url = CString::new(url).map_err(|_| SourceError::Message("url contains NUL".into()))?;
        let mut out = SourceMetaRaw {
            size: SOURCE_SIZE_UNKNOWN,
            last_modified_ms: SOURCE_LAST_MODIFIED_UNKNOWN,
            etag: std::ptr::null_mut(),
            content_type: std::ptr::null_mut(),
        };
        let mut errmsg: *mut c_char = std::ptr::null_mut();
        let rc = unsafe { (self.raw.head)(self.raw.ctx, c_url.as_ptr(), &mut out, &mut errmsg) };
        if rc != 0 {
            return Err(self.take_errmsg(errmsg, "head failed"));
        }
        Ok(SourceMeta {
            size: (out.size != SOURCE_SIZE_UNKNOWN).then_some(out.size),
            last_modified_ms: (out.last_modified_ms != SOURCE_LAST_MODIFIED_UNKNOWN)
                .then_some(out.last_modified_ms),
            etag: self.take_string(out.etag),
            content_type: self.take_string(out.content_type),
        })
    }

    pub fn get(&self, url: &str) -> SourceResult<SourceStream> {
        self.get_if(url, None)
    }

    /// Like [`Self::get`], failing with [`SourceError::Changed`] if the
    /// object's ETag no longer equals `if_match`.
    pub fn get_if(&self, url: &str, if_match: Option<&str>) -> SourceResult<SourceStream> {
        let c_url = CString::new(url).map_err(|_| SourceError::Message("url contains NUL".into()))?;
        let c_if = opt_cstring_arg(if_match)?;
        let mut out: *mut SourceStreamRaw = std::ptr::null_mut();
        let mut errmsg: *mut c_char = std::ptr::null_mut();
        let rc = unsafe {
            (self.raw.get)(
                self.raw.ctx,
                c_url.as_ptr(),
                c_if.as_ref().map_or(std::ptr::null(), |c| c.as_ptr()),
                &mut out,
                &mut errmsg,
            )
        };
        if rc != 0 {
            return Err(self.take_error(rc, errmsg, "get failed", url, if_match));
        }
        if out.is_null() {
            return Err(SourceError::Message("get returned a NULL stream".into()));
        }
        Ok(SourceStream { api: self.clone(), raw: out })
    }

    pub fn get_range(&self, url: &str, start: u64, len: u64) -> SourceResult<Vec<u8>> {
        self.get_range_if(url, start, len, None)
    }

    /// Like [`Self::get_range`], failing with [`SourceError::Changed`] if the
    /// object's ETag no longer equals `if_match`.
    pub fn get_range_if(
        &self,
        url: &str,
        start: u64,
        len: u64,
        if_match: Option<&str>,
    ) -> SourceResult<Vec<u8>> {
        let c_url = CString::new(url).map_err(|_| SourceError::Message("url contains NUL".into()))?;
        let c_if = opt_cstring_arg(if_match)?;
        let mut buf: *mut u8 = std::ptr::null_mut();
        let mut buf_len: u64 = 0;
        let mut errmsg: *mut c_char = std::ptr::null_mut();
        let rc = unsafe {
            (self.raw.get_range)(
                self.raw.ctx,
                c_url.as_ptr(),
                start,
                len,
                c_if.as_ref().map_or(std::ptr::null(), |c| c.as_ptr()),
                &mut buf,
                &mut buf_len,
                &mut errmsg,
            )
        };
        if rc != 0 {
            return Err(self.take_error(rc, errmsg, "get_range failed", url, if_match));
        }
        if buf.is_null() {
            return Ok(Vec::new());
        }
        let out = unsafe { std::slice::from_raw_parts(buf, buf_len as usize) }.to_vec();
        unsafe { (self.raw.free_buffer)(self.raw.ctx, buf, buf_len) };
        Ok(out)
    }
}

fn opt_cstring_arg(s: Option<&str>) -> SourceResult<Option<CString>> {
    match s {
        None => Ok(None),
        Some(s) => CString::new(s)
            .map(Some)
            .map_err(|_| SourceError::Message("if_match contains NUL".into())),
    }
}

/// A streaming body from [`SourceHandle::get`]. Implements [`Read`].
pub struct SourceStream {
    api: SourceHandle,
    raw: *mut SourceStreamRaw,
}

unsafe impl Send for SourceStream {}

impl Read for SourceStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        let mut errmsg: *mut c_char = std::ptr::null_mut();
        let n = unsafe {
            ((*self.raw).read)((*self.raw).ctx, buf.as_mut_ptr(), buf.len() as u64, &mut errmsg)
        };
        if n < 0 {
            return Err(self.api.take_errmsg(errmsg, "read failed").into());
        }
        Ok(n as usize)
    }
}

impl Drop for SourceStream {
    fn drop(&mut self) {
        unsafe { ((*self.raw).close)(self.raw) };
    }
}

/// Resolve a provider by calling `SELECT <function>()` on `db` and copying the
/// returned vtable out while the statement is still live.
pub fn resolve_source_api(db: *mut sqlite3, function: &str) -> SourceResult<SourceHandle> {
    if function.is_empty()
        || !function
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err(SourceError::Resolve {
            function: function.to_owned(),
            message: "invalid function name".into(),
        });
    }
    let sql = format!("SELECT \"{}\"()", function);
    let mut stmt = match Statement::prepare(db, &sql) {
        Ok(s) => s,
        Err(e) => {
            let message = e.to_string();
            return Err(if message.contains("no such function") {
                SourceError::NotLoaded { function: function.to_owned() }
            } else {
                SourceError::Resolve { function: function.to_owned(), message }
            });
        }
    };
    // Step by hand so an error raised *inside* the provider function (bad
    // config row, etc.) reaches the caller verbatim.
    let rc = unsafe { crate::ext::sqlite3ext_step(stmt.as_ptr()) };
    if rc != crate::constants::SQLITE_ROW {
        let message = unsafe { crate::exec::errmsg(db) };
        return Err(SourceError::Resolve {
            function: function.to_owned(),
            message: if message.is_empty() {
                format!("function returned no row (rc={})", rc)
            } else {
                message
            },
        });
    }
    let value = unsafe { crate::ext::sqlite3ext_column_value(stmt.as_ptr(), 0) };
    let p = unsafe { api::value_pointer::<SourceApiRaw>(&value, SOURCE_API_POINTER_NAME) };
    let result = match p {
        Some(p) => unsafe { SourceHandle::from_raw_ptr(p, function) },
        None => Err(SourceError::NotASourceApi { function: function.to_owned() }),
    };
    drop(stmt); // finalized after the handle has retained ctx
    result
}

/// Convenience: dispatch on `source`'s scheme and resolve the provider.
/// Returns `Ok(None)` for local paths.
pub fn resolve_for_source(
    db: *mut sqlite3,
    source: &str,
) -> SourceResult<Option<(SourceScheme, SourceHandle)>> {
    match source_function_for(source) {
        None => Ok(None),
        Some(scheme) => match resolve_source_api(db, scheme.function) {
            Ok(h) => Ok(Some((scheme, h))),
            Err(SourceError::NotLoaded { function }) => Err(SourceError::Message(
                SourceError::NotLoaded { function }.user_message(Some(scheme.scheme)),
            )),
            Err(e) => Err(e),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_matches_header() {
        // Keep in sync with the SQLITE_SOURCE_API_SIZEOF_* comments in sqlite-source.h
        // (64-bit pointers).
        assert_eq!(std::mem::size_of::<SourceMetaRaw>(), 32);
        assert_eq!(std::mem::size_of::<SourceStreamRaw>(), 24);
        assert_eq!(std::mem::size_of::<SourceApiRaw>(), 8 + 8 * 9);
        assert_eq!(std::mem::align_of::<SourceApiRaw>(), 8);
    }

    #[test]
    fn scheme_dispatch() {
        assert_eq!(source_function_for("https://x/y").unwrap().function, "_http_api");
        assert_eq!(source_function_for("HTTP://x/y").unwrap().function, "_http_api");
        assert_eq!(source_function_for("s3://b/k").unwrap().function, "_s3_api");
        assert!(source_function_for("/tmp/x.csv").is_none());
        assert!(source_function_for("C:\\data\\x.csv").is_none());
        assert!(source_function_for("gs://b/k").is_none());
        assert!(looks_like_url("gs://b/k"));
        assert!(!looks_like_url("data/x.csv"));
    }
}
