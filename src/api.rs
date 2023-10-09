//! (Mostly) safe wrappers around low-level sqlite3 C API.
//!
//! Uses the unsafe low-level API's defined in [`crate::ext`].
//!
//! Useful when working with sqlite3_value or sqlite3_context.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use crate::constants::SQLITE_OKAY;
use crate::ext::{
    sqlite3, sqlite3_context, sqlite3_value, sqlite3ext_context_db_handle, sqlite3ext_get_auxdata,
    sqlite3ext_mprintf, sqlite3ext_overload_function, sqlite3ext_result_blob,
    sqlite3ext_result_double, sqlite3ext_result_error, sqlite3ext_result_error_code,
    sqlite3ext_result_int, sqlite3ext_result_int64, sqlite3ext_result_null,
    sqlite3ext_result_pointer, sqlite3ext_result_subtype, sqlite3ext_result_text,
    sqlite3ext_set_auxdata, sqlite3ext_value_blob, sqlite3ext_value_bytes, sqlite3ext_value_double,
    sqlite3ext_value_int, sqlite3ext_value_int64, sqlite3ext_value_pointer,
    sqlite3ext_value_subtype, sqlite3ext_value_text, sqlite3ext_value_type,
};
use crate::Error;
use sqlite3ext_sys::{SQLITE_BLOB, SQLITE_FLOAT, SQLITE_INTEGER, SQLITE_NULL, SQLITE_TEXT};
use sqlite3ext_sys::sqlite3_aggregate_context;
use std::os::raw::c_int;
use std::slice::from_raw_parts;
use std::str::Utf8Error;
use std::{
    ffi::{CStr, CString, NulError},
    os::raw::{c_char, c_void},
};

/// Ergonomic wrapper around a raw sqlite3_value. It is the caller's reponsibility
/// to ensure that a given pointer points to a valid sqlite3_value object.
/// There seems to be a 5-10% perf cost when using Value vs calling functions on
/// raw pointers
pub struct Value {
    value: *mut sqlite3_value,
    value_type: ValueType,
}

impl Value {
    /// Create a Value struct from a borrowed sqlite3_value pointer
    pub fn from(value: &*mut sqlite3_value) -> crate::Result<Value> {
        let value_type = value_type(value);
        Ok(Value {
            value: value.to_owned(),
            value_type,
        })
    }
    /// Create a Value struct from a sqlite3_value pointer slice
    /// at the given index.
    pub fn at(values: &[*mut sqlite3_value], at: usize) -> Option<Value> {
        let value = values.get(at)?;
        let value_type = value_type(value);
        Some(Value {
            value: value.to_owned(),
            value_type,
        })
    }

    /// Ensure that the value's type isn't SQLITE_NULL - return the
    /// given error as an Err.
    pub fn notnull_or(&self, error: Error) -> crate::Result<&Self> {
        if self.value_type != ValueType::Null {
            Ok(self)
        } else {
            Err(error)
        }
    }

    /// Ensure that the value's type isn't SQLITE_NULL - otherwise
    /// call the error function and return as Err.
    pub fn notnull_or_else<F>(&self, err: F) -> crate::Result<&Self>
    where
        F: FnOnce() -> Error,
    {
        if self.value_type != ValueType::Null {
            Ok(self)
        } else {
            Err(err())
        }
    }

    /// Returns the UTF8 representation of the underlying sqlite_value.
    /// Fails if the value type is SQLITE_NULL, or if there's a UTF8
    /// error on the resulting string.
    pub fn text_or_else<F>(&self, error: F) -> crate::Result<&str>
    where
        F: FnOnce(Error) -> Error,
    {
        match value_text(&self.value) {
            Ok(value) => Ok(value),
            Err(err) => Err(error(err.into())),
        }
    }
}

/// Possible error cases when calling [`mprintf`], aka the sqlite3_mprintf function.
#[derive(Debug)]
pub enum MprintfError {
    Nul(NulError),
    Oom,
}

/// Calls [`sqlite3_mprintf`](https://sqlite.org/c3ref/mprintf.html) on the
/// given string, with memory allocated by sqlite3.
/// Meant to be passed into sqlite APIs that require sqlite-allocated strings,
/// like virtual table's `zErrMsg` or xBestIndex's `idxStr`
pub fn mprintf(base: &str) -> Result<*mut c_char, MprintfError> {
    let cbase = CString::new(base.as_bytes()).map_err(MprintfError::Nul)?;

    let result = unsafe { sqlite3ext_mprintf(cbase.as_ptr()) };
    if result.is_null() {
        Err(MprintfError::Oom)
    } else {
        Ok(result as *mut c_char)
    }
}

/// Returns the [`sqlite3_value_blob`](https://www.sqlite.org/c3ref/value_blob.html) result
/// from the given sqlite3_value, as a u8 slice.
pub fn value_blob<'a>(value: &*mut sqlite3_value) -> &'a [u8] {
    let n = value_bytes(value);
    let b = unsafe { sqlite3ext_value_blob(value.to_owned()) };
    return unsafe { from_raw_parts(b.cast::<u8>(), n as usize) };
}

/// Returns the [`sqlite3_value_bytes`](https://www.sqlite.org/c3ref/value_blob.html) result
/// from the given sqlite3_value, as i32.
pub fn value_bytes(value: &*mut sqlite3_value) -> i32 {
    unsafe { sqlite3ext_value_bytes(value.to_owned()) }
}

/// Returns the [`sqlite3_value_text`](https://www.sqlite.org/c3ref/value_blob.html) result
/// from the given sqlite3_value, as a str. If the number of bytes of the underlying value
/// is 0, then an empty string is returned. A UTF8 Error is returned if there are problems
/// encoding the string.
pub fn value_text<'a>(value: &*mut sqlite3_value) -> Result<&'a str, Utf8Error> {
    let n = value_bytes(value);
    if n == 0 {
        return Ok("");
    }
    unsafe {
        let c_string = sqlite3ext_value_text(value.to_owned());
        // TODO can i32 always fit as usize? maybe not all architectures...
        std::str::from_utf8(from_raw_parts(c_string, n as usize))
    }
}

pub fn value_text_notnull<'a>(value: &*mut sqlite3_value) -> Result<&'a str, Error> {
    if value_type(value) == ValueType::Null {
        return Err(Error::new_message("Unexpected null value"));
    }
    let c_string = unsafe { sqlite3ext_value_text(value.to_owned()) };
    let string = unsafe { CStr::from_ptr(c_string as *const c_char) };
    Ok(string.to_str()?)
}

/// [`sqlite3_value_pointer`](https://www.sqlite.org/bindptr.html)
///
/// # Safety
/// should this really be unsafe???
pub unsafe fn value_pointer<T>(value: &*mut sqlite3_value, c_name: &[u8]) -> Option<*mut T> {
    let result = sqlite3ext_value_pointer(
        value.to_owned(),
        c_name.as_ptr().cast::<c_char>().cast_mut(),
    );

    if result.is_null() {
        return None;
    }

    Some(result.cast::<T>())
}

/// Returns the [`sqlite3_value_int`](https://www.sqlite.org/c3ref/value_blob.html) result
/// from the given sqlite3_value, as i32.
pub fn value_int(value: &*mut sqlite3_value) -> i32 {
    unsafe { sqlite3ext_value_int(value.to_owned()) }
}

/// Returns the [`sqlite3_value_int64`](https://www.sqlite.org/c3ref/value_blob.html) result
/// from the given sqlite3_value, as i64.
pub fn value_int64(value: &*mut sqlite3_value) -> i64 {
    unsafe { sqlite3ext_value_int64(value.to_owned()) }
}

/// Returns the [`sqlite3_value_double`](https://www.sqlite.org/c3ref/value_blob.html) result
/// from the given sqlite3_value, as f64.
pub fn value_double(value: &*mut sqlite3_value) -> f64 {
    unsafe { sqlite3ext_value_double(value.to_owned()) }
}
pub fn value_json(value: &*mut sqlite3_value) -> serde_json::Result<serde_json::Value> {
    serde_json::from_slice(value_blob(value))
}

/// Possible values that sqlite3_value_type will return for a value.
#[derive(Eq, PartialEq)]
pub enum ValueType {
    /// text or a string, aka SQLITE_TEXT
    Text,
    /// Integer, aka  SQLITE_INTEGER
    Integer,
    /// Float/double, aka  SQLITE_FLOAT
    Float,
    /// blob, aka  SQLITE_BLOB
    Blob,
    /// NULL, aka  SQLITE_NULL
    Null,
}

/// Returns the [`sqlite3_value_type`](https://www.sqlite.org/c3ref/value_blob.html)
/// result of the given value, one of TEXT/INT/FLOAT/BLOB/NULL.
pub fn value_type(value: &*mut sqlite3_value) -> ValueType {
    let raw_type = unsafe { sqlite3ext_value_type(value.to_owned()) };
    // "as u32" because bindings for constants are u32 for some reason???
    match raw_type as u32 {
        SQLITE_TEXT => ValueType::Text,
        SQLITE_INTEGER => ValueType::Integer,
        SQLITE_FLOAT => ValueType::Float,
        SQLITE_BLOB => ValueType::Blob,
        SQLITE_NULL => ValueType::Null,
        // rationale: SQLite is never going to add a new value type as
        // long as sqlite3 is version 3. Certain extensions also make
        // this same extension, so we can as well
        _ => unreachable!(),
    }
}
pub fn value_is_null(value: &*mut sqlite3_value) -> bool {
    let raw_type = unsafe { sqlite3ext_value_type(value.to_owned()) };
    (raw_type as u32) == SQLITE_NULL
}

pub fn value_subtype(value: &*mut sqlite3_value) -> u32 {
    unsafe { sqlite3ext_value_subtype(value.to_owned()) }
}

// TODO test
pub fn value_has_pointer_subtype(value: &*mut sqlite3_value) -> bool {
    // https://github.com/sqlite/sqlite/blob/cc19bed8b10f4584d39aeb3e72fb6c30c3355955/src/vdbemem.c#L957
    // 112 == 'p'
    value_subtype(value) == 112
}
pub fn value_has_json_subtype(value: &*mut sqlite3_value) -> bool {
    // https://github.com/sqlite/sqlite/blob/cc19bed8b10f4584d39aeb3e72fb6c30c3355955/src/json.c#L89
    // 74 == 'p'
    value_subtype(value) == 74
}

/// Calls [`sqlite3_result_text`](https://www.sqlite.org/c3ref/result_blob.html)
/// to represent that a function returns a string with the given value. Fails if
/// the string length is larger than i32 maximum value.
pub fn result_text<S: AsRef<str>>(context: *mut sqlite3_context, text: S) -> crate::Result<()> {
    let bytes = text.as_ref().as_bytes();
    unsafe {
        // Rational: why not use CString::new here? Turns out, SQLite strings can have NUL characters
        // inside of strings. It fucks with LENGTH()/QUOTE(), but is totally valid. So, we should allow
        // returning strings with NULL values, as the "n" parameter sets the size limit of the string.
        // <https://www.sqlite.org/nulinstr.html>
        let s = CString::from_vec_unchecked(bytes.into());

        let n: i32 = bytes
            .len()
            .try_into()
            .map_err(|_| Error::new_message("i32 overflow, string to large"))?;
        // CString and into_raw() is needed here, that way we can pass in a proper destructor so
        // SQLite can drop the allocated memory (avoids segfaults)
        sqlite3ext_result_text(context, s.into_raw(), n, Some(result_text_destructor));
    }
    Ok(())
}
unsafe extern "C" fn result_text_destructor(raw: *mut c_void) {
    drop(CString::from_raw(raw.cast::<c_char>()));
}

/// Calls [`sqlite3_result_int`](https://www.sqlite.org/c3ref/result_blob.html)
/// to represent that a function returns an int32 with the given value.
pub fn result_int(context: *mut sqlite3_context, i: i32) {
    unsafe { sqlite3ext_result_int(context, i) };
}

///[`sqlite3_result_int64`](https://www.sqlite.org/c3ref/result_blob.html)
/// to represent that a function returns an int64 with the given value.
pub fn result_int64(context: *mut sqlite3_context, i: i64) {
    unsafe { sqlite3ext_result_int64(context, i) };
}

/// Calls [`sqlite3_result_double`](https://www.sqlite.org/c3ref/result_blob.html)
/// to represent that a function returns a double/float with the given value.
pub fn result_double(context: *mut sqlite3_context, i: f64) {
    unsafe { sqlite3ext_result_double(context, i) };
}

/// Calls [`sqlite3_result_blob`](https://www.sqlite.org/c3ref/result_blob.html)
/// to represent that a function returns a blob with the given value.
pub fn result_blob(context: *mut sqlite3_context, blob: &[u8]) {
    // TODO try_into(), err on too big (check against limit? idk)
    let len = blob.len() as c_int;
    unsafe { sqlite3ext_result_blob(context, blob.as_ptr().cast::<c_void>(), len) };
}

/// Calls [`sqlite3_result_null`](https://www.sqlite.org/c3ref/result_blob.html)
/// to represent that a function returns null with the given value.
pub fn result_null(context: *mut sqlite3_context) {
    unsafe { sqlite3ext_result_null(context) };
}

/// Calls [`sqlite3_result_error`](https://www.sqlite.org/c3ref/result_blob.html)
/// to represent that a function returns an error with the given value.
/// Note: You can typically rely on [`crate::Result`] to do this for you.
pub fn result_error(context: *mut sqlite3_context, text: &str) -> crate::Result<()> {
    let s = CString::new(text.as_bytes())?;
    let n = text.len() as i32;

    unsafe {
        // According to the docs: https://www.sqlite.org/c3ref/result_blob.html
        // "The sqlite3_result_error() and sqlite3_result_error16() routines make a
        // private copy of the error message text before they return. Hence, the
        // calling function can deallocate or modify the text after they return
        // without harm."
        let s_ptr = s.into_raw();
        sqlite3ext_result_error(context, s_ptr, n);
        drop(CString::from_raw(s_ptr));
    }

    Ok(())
}

/// Calls [`sqlite3_result_error_code`](https://www.sqlite.org/c3ref/result_blob.html)
/// to represent that a function returns xx with the given value.
pub fn result_error_code(context: *mut sqlite3_context, code: i32) {
    unsafe { sqlite3ext_result_error_code(context, code) };
}

/// Calls [`result_int`] with `value=1` for true, or `value=0` for false.
pub fn result_bool(context: *mut sqlite3_context, value: bool) {
    if value {
        result_int(context, 1)
    } else {
        result_int(context, 0)
    }
}

/// Result the given JSON as a value that other SQLite JSON functions expect: a stringified
/// text result with subtype of 'J'.
pub fn result_json(context: *mut sqlite3_context, value: serde_json::Value) -> crate::Result<()> {
    result_text(context, value.to_string().as_str())?;
    // https://github.com/sqlite/sqlite/blob/master/src/json.c#L88-L89
    result_subtype(context, b'J');
    Ok(())
}

/// Calls [`sqlite3_result_subtype`](https://www.sqlite.org/c3ref/result_subtype.html)
pub fn result_subtype(context: *mut sqlite3_context, subtype: u8) {
    // Explanation for u8: "Only the lower 8 bits of the subtype T are preserved
    // in current versions of SQLite; higher order bits are discarded"
    unsafe { sqlite3ext_result_subtype(context, subtype.into()) };
}

unsafe extern "C" fn pointer_destroy<T>(pointer: *mut c_void) {
    drop(Box::from_raw(pointer.cast::<T>()))
}

/// [sqlite3_result_pointer](https://www.sqlite.org/bindptr.html)
pub fn result_pointer<T>(context: *mut sqlite3_context, name: &[u8], object: T) {
    let b = Box::new(object);
    let pointer = Box::into_raw(b).cast::<c_void>();
    unsafe {
        sqlite3ext_result_pointer(
            context,
            pointer,
            name.as_ptr().cast::<c_char>().cast_mut(),
            Some(pointer_destroy::<T>),
        )
    };
}

// TODO maybe take in a Box<T>?
/// [`sqlite3_set_auxdata`](https://www.sqlite.org/c3ref/get_auxdata.html)
pub fn auxdata_set(
    context: *mut sqlite3_context,
    col: i32,
    p: *mut c_void,
    d: Option<unsafe extern "C" fn(*mut c_void)>,
) {
    unsafe {
        sqlite3ext_set_auxdata(context, col, p, d);
    }
}

// TODO maybe return a Box<T>?
/// [`sqlite3_get_auxdata`](https://www.sqlite.org/c3ref/get_auxdata.html)
pub fn auxdata_get(context: *mut sqlite3_context, col: i32) -> *mut c_void {
    unsafe { sqlite3ext_get_auxdata(context, col) }
}

pub fn context_db_handle(context: *mut sqlite3_context) -> *mut sqlite3 {
    unsafe { sqlite3ext_context_db_handle(context) }
}
pub fn overload_function(db: *mut sqlite3, func_name: &str, n_args: i32) -> crate::Result<()> {
    let cname = CString::new(func_name)?;
    let result = unsafe { sqlite3ext_overload_function(db, cname.as_ptr(), n_args) };
    if result != SQLITE_OKAY {
        return Err(Error::new_message("TODO"));
    }
    Ok(())
}
/// A columns "affinity". <https://www.sqlite.org/datatype3.html#type_affinity>
/* TODO maybe include extra affinities?
- JSON - parse as text, see if it's JSON, if so then set subtype
- boolean - 1 or 0, then 1 or 0. What about YES/NO or TRUE/FALSE or T/F?
- datetime - idk man
- interval - idk man
 */
pub enum ColumnAffinity {
    /// "char", "clob", or "text"
    Text,
    /// "int"
    Integer,
    /// "real", "floa", or "doub"
    Real,
    /// "blob" or empty
    Blob,
    /// else, no other matches
    Numeric,
}

impl ColumnAffinity {
    /// Determines a column's affinity based on its declared typed, from
    /// <https://www.sqlite.org/datatype3.html#determination_of_column_affinity>
    pub fn from_declared_type(declared_type: &str) -> Self {
        let lowered = declared_type.trim().to_lowercase();
        // "If the declared type contains the string "INT" then it is assigned INTEGER affinity."
        if lowered.contains("int") {
            return ColumnAffinity::Integer;
        };

        // "If the declared type of the column contains any of the strings "CHAR",
        // "CLOB", or "TEXT" then that column has TEXT affinity.
        // Notice that the type VARCHAR contains the string "CHAR" and is
        // thus assigned TEXT affinity."

        if lowered.contains("char") || lowered.contains("clob") || lowered.contains("text") {
            return ColumnAffinity::Text;
        };

        // "If the declared type for a column contains the string "BLOB" or if no
        // type is specified then the column has affinity BLOB."

        if lowered.contains("blob") || lowered.is_empty() {
            return ColumnAffinity::Blob;
        };

        // "If the declared type for a column contains any of the strings "REAL",
        // "FLOA", or "DOUB" then the column has REAL affinity."
        if lowered.contains("real") || lowered.contains("floa") || lowered.contains("doub") {
            return ColumnAffinity::Real;
        };

        // "Otherwise, the affinity is NUMERIC"
        ColumnAffinity::Numeric
    }

    /// Result the given value on the given sqlite3_context, while applying
    /// the proper affinity rules. It may instead result as an i32, i64,
    /// or f64 numberor default back to just text.

    pub fn result_text(&self, context: *mut sqlite3_context, value: &str) -> crate::Result<()> {
        match self {
            ColumnAffinity::Numeric => {
                if let Ok(value) = value.parse::<i32>() {
                    result_int(context, value)
                } else if let Ok(value) = value.parse::<i64>() {
                    result_int64(context, value)
                } else if let Ok(value) = value.parse::<f64>() {
                    result_double(context, value);
                } else {
                    result_text(context, value)?;
                }
            }
            ColumnAffinity::Integer => {
                if let Ok(value) = value.parse::<i32>() {
                    result_int(context, value)
                } else if let Ok(value) = value.parse::<i64>() {
                    result_int64(context, value)
                } else {
                    result_text(context, value)?;
                }
            }
            ColumnAffinity::Real => {
                if let Ok(value) = value.parse::<f64>() {
                    result_double(context, value);
                } else {
                    result_text(context, value)?;
                }
            }
            ColumnAffinity::Blob | ColumnAffinity::Text => result_text(context, value)?,
        };
        Ok(())
    }
}

/// A columns "extended affinity". The traditional affinity does
/// not include supplementary "types" that SQLite doesn't support
/// out of the box, like JSON, boolean, or datetime. This is an
/// experimental extension to tradition affinities, and may change
/// anytime.
/* TODO maybe include extra affinities?
- JSON - parse as text, see if it's JSON, if so then set subtype
- boolean - 1 or 0, then 1 or 0. What about YES/NO or TRUE/FALSE or T/F?
- datetime - idk man
- interval - idk man
*/
pub enum ExtendedColumnAffinity {
    /// "char", "clob", or "text"
    Text,
    /// "int"
    Integer,
    /// "real", "floa", or "doub"
    Real,
    /// "blob" or empty
    Blob,
    /// 0 or 1
    Boolean,
    Json,
    Datetime,
    Date,
    Time,
    /// else, no other matches
    Numeric,
}

impl ExtendedColumnAffinity {
    // https://www.sqlite.org/datatype3.html#determination_of_column_affinity
    pub fn extended_column_affinity_from_type(declared_type: &str) -> Self {
        let lowered = declared_type.to_lowercase();
        // "If the declared type contains the string "INT" then it is assigned INTEGER affinity."
        if lowered.contains("int") {
            return ExtendedColumnAffinity::Integer;
        };

        // "If the declared type of the column contains any of the strings "CHAR",
        // "CLOB", or "TEXT" then that column has TEXT affinity.
        // Notice that the type VARCHAR contains the string "CHAR" and is
        // thus assigned TEXT affinity."

        if lowered.contains("char") || lowered.contains("clob") || lowered.contains("text") {
            return ExtendedColumnAffinity::Text;
        };

        // "If the declared type for a column contains the string "BLOB" or if no
        // type is specified then the column has affinity BLOB."

        if lowered.contains("blob") || lowered.is_empty() {
            return ExtendedColumnAffinity::Blob;
        };

        // "If the declared type for a column contains any of the strings "REAL",
        // "FLOA", or "DOUB" then the column has REAL affinity."
        if lowered.contains("real") || lowered.contains("floa") || lowered.contains("doub") {
            return ExtendedColumnAffinity::Real;
        };
        if lowered.contains("json") {
            return ExtendedColumnAffinity::Json;
        };
        if lowered.contains("boolean") {
            return ExtendedColumnAffinity::Boolean;
        };

        // "Otherwise, the affinity is NUMERIC"
        ExtendedColumnAffinity::Numeric
    }
}

// TODO write test
pub fn get_aggregate_context_value<T>(context: *mut sqlite3_context) -> Result<T, String>
where
    T: Copy,
{
    let p_value: *mut T = unsafe {
        sqlite3_aggregate_context(context, std::mem::size_of::<T>() as i32) as *mut T
    };

    if p_value.is_null() {
        return Err("sqlite3_aggregate_context returned a null pointer.".to_string());
    }

    let value: T = unsafe { *p_value };

    Ok(value)
}

// TODO write test
pub fn set_aggregate_context_value<T>(context: *mut sqlite3_context, value: T) -> Result<(), String>
where
    T: Copy,
{
    let p_value: *mut T = unsafe {
        sqlite3_aggregate_context(context, std::mem::size_of::<T>() as i32) as *mut T
    };

    if p_value.is_null() {
        return Err("sqlite3_aggregate_context returned a null pointer.".to_string());
    }

    unsafe {
        *p_value = value;
    }

    Ok(())
}