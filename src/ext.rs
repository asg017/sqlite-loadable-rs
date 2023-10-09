//! Unsafe wrappers around low-level sqlite3 C API.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::too_many_arguments)]

/// WARNING: This file should only carefully be updated. The code here essentially emulates the
/// SQLITE_EXTENSION_INIT1 and SQLITE_EXTENSION_INIT2 macros that aren't available in Rust,
/// making an broken unsafe-raw C API into a not-broken unsafe-raw C API.
/// The functions exposed here (prefixed with "sqlite3ext_" by convention) are still unsafe,
/// but at least work for both dynamically-loadable and statically built extensions.
/// One should not need to work with these functions directly - unless you're working internally
/// on the sqlite_loadable library.
use std::{
    mem,
    os::raw::{c_char, c_int, c_uchar, c_void},
};

#[cfg(feature = "static")]
pub use libsqlite3_sys::{
    sqlite3, sqlite3_api_routines, sqlite3_context,
    sqlite3_index_constraint as sqlite3_index_info_sqlite3_index_constraint,
    sqlite3_index_constraint_usage as sqlite3_index_info_sqlite3_index_constraint_usage,
    sqlite3_index_info, sqlite3_index_orderby as sqlite3_index_info_sqlite3_index_orderby,
    sqlite3_module, sqlite3_stmt, sqlite3_value, sqlite3_vtab, sqlite3_vtab_cursor,
};

#[cfg(not(feature = "static"))]
pub use sqlite3ext_sys::{
    sqlite3, sqlite3_api_routines, sqlite3_context, sqlite3_index_info,
    sqlite3_index_info_sqlite3_index_constraint, sqlite3_index_info_sqlite3_index_constraint_usage,
    sqlite3_index_info_sqlite3_index_orderby, sqlite3_module, sqlite3_stmt, sqlite3_value,
    sqlite3_vtab, sqlite3_vtab_cursor, sqlite3_create_window_function
};

/// If creating a dynmically loadable extension, this MUST be redefined to point
/// to a proper sqlite3_api_rountines module (from a entrypoint function).
/// The "sqlite_entrypoint" macro will do this for you usually.
static mut SQLITE3_API: *mut sqlite3_api_routines = std::ptr::null_mut();

/// This function MUST be called in loadable extension before any of the below functions are invoked.
/// (The sqlite_entrypoint function will do this for you).
/// This essentially emulates the SQLITE_EXTENSION_INIT2 macro that's not available in rust-land.
/// Without it, when dynamically loading extensions, calls to SQLite C-API functions in sqlite3ext_sys
/// like sqlite3_value_text will segfault, because sqlite3ext.h does not include their proper definitions.
/// Instead, a sqlite3_api_routines object is provided through the entrypoint at runtime, to which
/// sqlite_loadable will redefine the static SQLITE3_API variable that the functions below requre.
pub unsafe fn faux_sqlite_extension_init2(api: *mut sqlite3_api_routines) {
    if !api.is_null() {
        SQLITE3_API = api;
    }
}

#[cfg(not(feature = "static"))]
static EXPECT_MESSAGE: &str =
    "sqlite-loadable error: expected method on SQLITE3_API. Please file an issue";

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_value_text(arg1: *mut sqlite3_value) -> *const ::std::os::raw::c_uchar {
    libsqlite3_sys::sqlite3_value_text(arg1)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_value_text(arg1: *mut sqlite3_value) -> *const ::std::os::raw::c_uchar {
    ((*SQLITE3_API).value_text.expect(EXPECT_MESSAGE))(arg1)
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_value_type(value: *mut sqlite3_value) -> i32 {
    libsqlite3_sys::sqlite3_value_type(value)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_value_type(value: *mut sqlite3_value) -> i32 {
    ((*SQLITE3_API).value_type.expect(EXPECT_MESSAGE))(value)
}
#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_value_subtype(value: *mut sqlite3_value) -> u32 {
    libsqlite3_sys::sqlite3_value_subtype(value)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_value_subtype(value: *mut sqlite3_value) -> u32 {
    ((*SQLITE3_API).value_subtype.expect(EXPECT_MESSAGE))(value)
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_value_bytes(arg1: *mut sqlite3_value) -> i32 {
    libsqlite3_sys::sqlite3_value_bytes(arg1)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_value_bytes(arg1: *mut sqlite3_value) -> i32 {
    ((*SQLITE3_API).value_bytes.expect(EXPECT_MESSAGE))(arg1)
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_value_blob(arg1: *mut sqlite3_value) -> *const c_void {
    libsqlite3_sys::sqlite3_value_blob(arg1)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_value_blob(arg1: *mut sqlite3_value) -> *const c_void {
    ((*SQLITE3_API).value_blob.expect(EXPECT_MESSAGE))(arg1)
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_bind_pointer(
    db: *mut sqlite3_stmt,
    i: i32,
    p: *mut c_void,
    t: *const c_char,
) -> i32 {
    libsqlite3_sys::sqlite3_bind_pointer(db, i, p, t, None)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_bind_pointer(
    db: *mut sqlite3_stmt,
    i: i32,
    p: *mut c_void,
    t: *const c_char,
) -> i32 {
    ((*SQLITE3_API).bind_pointer.expect(EXPECT_MESSAGE))(db, i, p, t, None)
}
#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_step(stmt: *mut sqlite3_stmt) -> c_int {
    libsqlite3_sys::sqlite3_step(stmt)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_step(stmt: *mut sqlite3_stmt) -> c_int {
    ((*SQLITE3_API).step.expect(EXPECT_MESSAGE))(stmt)
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_finalize(stmt: *mut sqlite3_stmt) -> c_int {
    libsqlite3_sys::sqlite3_finalize(stmt)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_finalize(stmt: *mut sqlite3_stmt) -> c_int {
    ((*SQLITE3_API).finalize.expect(EXPECT_MESSAGE))(stmt)
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_column_text(stmt: *mut sqlite3_stmt, c: c_int) -> *const c_uchar {
    libsqlite3_sys::sqlite3_column_text(stmt, c)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_column_text(stmt: *mut sqlite3_stmt, c: c_int) -> *const c_uchar {
    ((*SQLITE3_API).column_text.expect(EXPECT_MESSAGE))(stmt, c)
}
#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_column_int64(stmt: *mut sqlite3_stmt, c: c_int) -> i64 {
    libsqlite3_sys::sqlite3_column_int64(stmt, c)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_column_int64(stmt: *mut sqlite3_stmt, c: c_int) -> i64 {
    ((*SQLITE3_API).column_int64.expect(EXPECT_MESSAGE))(stmt, c)
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_column_bytes(stmt: *mut sqlite3_stmt, c: c_int) -> i32 {
    libsqlite3_sys::sqlite3_column_bytes(stmt, c)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_column_bytes(stmt: *mut sqlite3_stmt, c: c_int) -> i32 {
    ((*SQLITE3_API).column_bytes.expect(EXPECT_MESSAGE))(stmt, c)
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_column_value(stmt: *mut sqlite3_stmt, c: c_int) -> *mut sqlite3_value {
    libsqlite3_sys::sqlite3_column_value(stmt, c)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_column_value(stmt: *mut sqlite3_stmt, c: c_int) -> *mut sqlite3_value {
    ((*SQLITE3_API).column_value.expect(EXPECT_MESSAGE))(stmt, c)
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_bind_text(
    stmt: *mut sqlite3_stmt,
    c: c_int,
    s: *const c_char,
    n: c_int,
    destructor: Option<unsafe extern "C" fn(*mut c_void)>,
) -> i32 {
    libsqlite3_sys::sqlite3_bind_text(stmt, c, s, n, destructor)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_bind_text(
    stmt: *mut sqlite3_stmt,
    c: c_int,
    s: *const c_char,
    n: c_int,
    destructor: Option<unsafe extern "C" fn(*mut c_void)>,
) -> i32 {
    ((*SQLITE3_API).bind_text.expect(EXPECT_MESSAGE))(stmt, c, s, n, destructor)
}
#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_bind_int(stmt: *mut sqlite3_stmt, c: c_int, v: c_int) -> i32 {
    libsqlite3_sys::sqlite3_bind_int(stmt, c, v)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_bind_int(stmt: *mut sqlite3_stmt, c: c_int, v: c_int) -> i32 {
    ((*SQLITE3_API).bind_int.expect(EXPECT_MESSAGE))(stmt, c, v)
}
#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_bind_int64(stmt: *mut sqlite3_stmt, c: c_int, v: i64) -> i32 {
    libsqlite3_sys::sqlite3_bind_int64(stmt, c, v)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_bind_int64(stmt: *mut sqlite3_stmt, c: c_int, v: i64) -> i32 {
    ((*SQLITE3_API).bind_int64.expect(EXPECT_MESSAGE))(stmt, c, v)
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_prepare_v2(
    db: *mut sqlite3,
    sql: *const c_char,
    n: i32,
    stmt: *mut *mut sqlite3_stmt,
    leftover: *mut *const c_char,
) -> i32 {
    libsqlite3_sys::sqlite3_prepare_v2(db, sql, n, stmt, leftover)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_prepare_v2(
    db: *mut sqlite3,
    sql: *const c_char,
    n: i32,
    stmt: *mut *mut sqlite3_stmt,
    leftover: *mut *const c_char,
) -> i32 {
    ((*SQLITE3_API).prepare_v2.expect(EXPECT_MESSAGE))(db, sql, n, stmt, leftover)
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_value_int(arg1: *mut sqlite3_value) -> i32 {
    libsqlite3_sys::sqlite3_value_int(arg1)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_value_int(arg1: *mut sqlite3_value) -> i32 {
    ((*SQLITE3_API).value_int.expect(EXPECT_MESSAGE))(arg1)
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_value_int64(arg1: *mut sqlite3_value) -> i64 {
    libsqlite3_sys::sqlite3_value_int64(arg1)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_value_int64(arg1: *mut sqlite3_value) -> i64 {
    ((*SQLITE3_API).value_int64.expect(EXPECT_MESSAGE))(arg1)
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_value_double(arg1: *mut sqlite3_value) -> f64 {
    libsqlite3_sys::sqlite3_value_double(arg1)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_value_double(arg1: *mut sqlite3_value) -> f64 {
    ((*SQLITE3_API).value_double.expect(EXPECT_MESSAGE))(arg1)
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_value_pointer(arg1: *mut sqlite3_value, p: *mut c_char) -> *mut c_void {
    libsqlite3_sys::sqlite3_value_pointer(arg1, p)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_value_pointer(arg1: *mut sqlite3_value, p: *mut c_char) -> *mut c_void {
    ((*SQLITE3_API).value_pointer.expect(EXPECT_MESSAGE))(arg1, p)
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_result_int(context: *mut sqlite3_context, v: c_int) {
    libsqlite3_sys::sqlite3_result_int(context, v)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_result_int(context: *mut sqlite3_context, v: c_int) {
    if false {
    } else {
        ((*SQLITE3_API).result_int.expect(EXPECT_MESSAGE))(context, v);
    }
}

// https://www.sqlite.org/c3ref/c_static.html
// SQLITE_STATIC == 0
// SQLITE_TRANSIENT == -1
// TODO instead of making a copy every time, let's tranfer ownership to some Rust box
// also maybe pass in a box/T and do the box stuff ourself?
// or serde??
// or slice??
#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_result_blob(context: *mut sqlite3_context, p: *const c_void, n: i32) {
    libsqlite3_sys::sqlite3_result_blob(context, p, n, Some(mem::transmute(-1_isize)));
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_result_blob(context: *mut sqlite3_context, p: *const c_void, n: i32) {
    ((*SQLITE3_API).result_blob.expect(EXPECT_MESSAGE))(
        context,
        p,
        n,
        Some(mem::transmute(-1_isize)),
    );
}
#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_result_int64(context: *mut sqlite3_context, v: i64) {
    libsqlite3_sys::sqlite3_result_int64(context, v);
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_result_int64(context: *mut sqlite3_context, v: i64) {
    ((*SQLITE3_API).result_int64.expect(EXPECT_MESSAGE))(context, v);
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_result_double(context: *mut sqlite3_context, f: f64) {
    libsqlite3_sys::sqlite3_result_double(context, f)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_result_double(context: *mut sqlite3_context, f: f64) {
    ((*SQLITE3_API).result_double.expect(EXPECT_MESSAGE))(context, f);
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_result_null(context: *mut sqlite3_context) {
    libsqlite3_sys::sqlite3_result_null(context);
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_result_null(context: *mut sqlite3_context) {
    ((*SQLITE3_API).result_null.expect(EXPECT_MESSAGE))(context);
}
#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_result_pointer(
    context: *mut sqlite3_context,
    pointer: *mut c_void,
    name: *mut c_char,
    destructor: Option<unsafe extern "C" fn(*mut std::ffi::c_void)>,
) {
    libsqlite3_sys::sqlite3_result_pointer(context, pointer, name, destructor)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_result_pointer(
    context: *mut sqlite3_context,
    pointer: *mut c_void,
    name: *mut c_char,
    destructor: Option<unsafe extern "C" fn(*mut std::ffi::c_void)>,
) {
    ((*SQLITE3_API).result_pointer.expect(EXPECT_MESSAGE))(context, pointer, name, destructor);
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_result_error(context: *mut sqlite3_context, s: *const c_char, n: i32) {
    libsqlite3_sys::sqlite3_result_error(context, s, n);
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_result_error(context: *mut sqlite3_context, s: *const c_char, n: i32) {
    ((*SQLITE3_API).result_error.expect(EXPECT_MESSAGE))(context, s, n);
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_result_error_code(context: *mut sqlite3_context, code: i32) {
    libsqlite3_sys::sqlite3_result_error_code(context, code);
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_result_error_code(context: *mut sqlite3_context, code: i32) {
    ((*SQLITE3_API).result_error_code.expect(EXPECT_MESSAGE))(context, code);
}
#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_result_text(
    context: *mut sqlite3_context,
    s: *const c_char,
    n: i32,
    d: Option<unsafe extern "C" fn(*mut c_void)>,
) {
    libsqlite3_sys::sqlite3_result_text(context, s, n, d);
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_result_text(
    context: *mut sqlite3_context,
    s: *const c_char,
    n: i32,
    d: Option<unsafe extern "C" fn(*mut c_void)>,
) {
    ((*SQLITE3_API).result_text.expect(EXPECT_MESSAGE))(context, s, n, d);
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_result_subtype(context: *mut sqlite3_context, subtype: u32) {
    libsqlite3_sys::sqlite3_result_subtype(context, subtype)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_result_subtype(context: *mut sqlite3_context, subtype: u32) {
    ((*SQLITE3_API).result_subtype.expect(EXPECT_MESSAGE))(context, subtype);
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_set_auxdata(
    context: *mut sqlite3_context,
    n: c_int,
    p: *mut c_void,
    d: Option<unsafe extern "C" fn(*mut c_void)>,
) {
    libsqlite3_sys::sqlite3_set_auxdata(context, n, p, d);
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_set_auxdata(
    context: *mut sqlite3_context,
    n: c_int,
    p: *mut c_void,
    d: Option<unsafe extern "C" fn(*mut c_void)>,
) {
    ((*SQLITE3_API).set_auxdata.expect(EXPECT_MESSAGE))(context, n, p, d);
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_get_auxdata(context: *mut sqlite3_context, n: c_int) -> *mut c_void {
    libsqlite3_sys::sqlite3_get_auxdata(context, n)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_get_auxdata(context: *mut sqlite3_context, n: c_int) -> *mut c_void {
    ((*SQLITE3_API).get_auxdata.expect(EXPECT_MESSAGE))(context, n)
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_create_window_function(
    db: *mut sqlite3,
    s: *const c_char,
    argc: i32,
    text_rep: i32,
    p_app: *mut c_void,
    x_step: Option<unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value)>,
    x_final: Option<unsafe extern "C" fn(*mut sqlite3_context)>,
    x_value: Option<unsafe extern "C" fn(*mut sqlite3_context)>,
    x_inverse: Option<unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value)>,
    destroy: Option<unsafe extern "C" fn(*mut c_void)>
) -> c_int {
    sqlite3_create_window_function(
        db, s, argc, text_rep, p_app, x_step, x_final, x_value, x_inverse, destroy,
    )
}

#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_create_window_function(
    db: *mut sqlite3,
    s: *const c_char,
    argc: i32,
    text_rep: i32,
    p_app: *mut c_void,
    x_step: Option<unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value)>,
    x_final: Option<unsafe extern "C" fn(*mut sqlite3_context)>,
    x_value: Option<unsafe extern "C" fn(*mut sqlite3_context)>,
    x_inverse: Option<unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value)>,
    destroy: Option<unsafe extern "C" fn(*mut c_void)>
) -> c_int {
    ((*SQLITE3_API).create_window_function.expect(EXPECT_MESSAGE))(
        db, s, argc, text_rep, p_app, x_step, x_final, x_value, x_inverse, destroy,
    )
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_create_function_v2(
    db: *mut sqlite3,
    s: *const c_char,
    argc: i32,
    text_rep: i32,
    p_app: *mut c_void,
    x_func: Option<unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value)>,
    x_step: Option<unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value)>,
    x_final: Option<unsafe extern "C" fn(*mut sqlite3_context)>,
    destroy: Option<unsafe extern "C" fn(*mut c_void)>,
) -> c_int {
    libsqlite3_sys::sqlite3_create_function_v2(
        db, s, argc, text_rep, p_app, x_func, x_step, x_final, destroy,
    )
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_create_function_v2(
    db: *mut sqlite3,
    s: *const c_char,
    argc: i32,
    text_rep: i32,
    p_app: *mut c_void,
    x_func: Option<unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value)>,
    x_step: Option<unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value)>,
    x_final: Option<unsafe extern "C" fn(*mut sqlite3_context)>,
    destroy: Option<unsafe extern "C" fn(*mut c_void)>,
) -> c_int {
    ((*SQLITE3_API).create_function_v2.expect(EXPECT_MESSAGE))(
        db, s, argc, text_rep, p_app, x_func, x_step, x_final, destroy,
    )
}
#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_collation_v2(
    db: *mut sqlite3,
    s: *const c_char,
    text_rep: i32,
    p_app: *mut c_void,
    x_compare: Option<
        unsafe extern "C" fn(
            *mut ::std::os::raw::c_void,
            ::std::os::raw::c_int,
            *const ::std::os::raw::c_void,
            ::std::os::raw::c_int,
            *const ::std::os::raw::c_void,
        ) -> ::std::os::raw::c_int,
    >,
    destroy: Option<unsafe extern "C" fn(*mut c_void)>,
) -> c_int {
    libsqlite3_sys::sqlite3_create_collation_v2(db, s, text_rep, p_app, x_compare, destroy)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_collation_v2(
    db: *mut sqlite3,
    s: *const c_char,
    text_rep: i32,
    p_app: *mut c_void,
    x_compare: Option<
        unsafe extern "C" fn(
            *mut ::std::os::raw::c_void,
            ::std::os::raw::c_int,
            *const ::std::os::raw::c_void,
            ::std::os::raw::c_int,
            *const ::std::os::raw::c_void,
        ) -> ::std::os::raw::c_int,
    >,
    destroy: Option<unsafe extern "C" fn(*mut c_void)>,
) -> c_int {
    ((*SQLITE3_API).create_collation_v2.expect(EXPECT_MESSAGE))(
        db, s, text_rep, p_app, x_compare, destroy,
    )
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_create_module_v2(
    db: *mut sqlite3,
    s: *const c_char,
    module: *const sqlite3_module,
    p_app: *mut c_void,
    destroy: Option<unsafe extern "C" fn(*mut c_void)>,
) -> i32 {
    libsqlite3_sys::sqlite3_create_module_v2(db, s, module, p_app, destroy)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_create_module_v2(
    db: *mut sqlite3,
    s: *const c_char,
    module: *const sqlite3_module,
    p_app: *mut c_void,
    destroy: Option<unsafe extern "C" fn(*mut c_void)>,
) -> i32 {
    ((*SQLITE3_API).create_module_v2.expect(EXPECT_MESSAGE))(db, s, module, p_app, destroy)
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_vtab_distinct(index_info: *mut sqlite3_index_info) -> i32 {
    libsqlite3_sys::sqlite3_vtab_distinct(index_info)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_vtab_distinct(index_info: *mut sqlite3_index_info) -> i32 {
    ((*SQLITE3_API).vtab_distinct.expect(EXPECT_MESSAGE))(index_info)
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_vtab_in(
    index_info: *mut sqlite3_index_info,
    constraint_idx: i32,
    handle: i32,
) -> i32 {
    libsqlite3_sys::sqlite3_vtab_in(index_info, constraint_idx, handle)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_vtab_in(
    index_info: *mut sqlite3_index_info,
    constraint_idx: i32,
    handle: i32,
) -> i32 {
    ((*SQLITE3_API).vtab_in.expect(EXPECT_MESSAGE))(index_info, constraint_idx, handle)
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_vtab_in_first(
    value_list: *mut sqlite3_value,
    value_out: *mut *mut sqlite3_value,
) -> i32 {
    libsqlite3_sys::sqlite3_vtab_in_first(value_list, value_out)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_vtab_in_first(
    value_list: *mut sqlite3_value,
    value_out: *mut *mut sqlite3_value,
) -> i32 {
    ((*SQLITE3_API).vtab_in_first.expect(EXPECT_MESSAGE))(value_list, value_out)
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_vtab_in_next(
    value_list: *mut sqlite3_value,
    value_out: *mut *mut sqlite3_value,
) -> i32 {
    libsqlite3_sys::sqlite3_vtab_in_next(value_list, value_out)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_vtab_in_next(
    value_list: *mut sqlite3_value,
    value_out: *mut *mut sqlite3_value,
) -> i32 {
    ((*SQLITE3_API).vtab_in_next.expect(EXPECT_MESSAGE))(value_list, value_out)
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_declare_vtab(db: *mut sqlite3, s: *const c_char) -> i32 {
    libsqlite3_sys::sqlite3_declare_vtab(db, s)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_declare_vtab(db: *mut sqlite3, s: *const c_char) -> i32 {
    #[cfg(not(feature = "static"))]
    ((*SQLITE3_API).declare_vtab.expect(EXPECT_MESSAGE))(db, s)
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_overload_function(db: *mut sqlite3, s: *const c_char, n: i32) -> i32 {
    libsqlite3_sys::sqlite3_overload_function(db, s, n)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_overload_function(db: *mut sqlite3, s: *const c_char, n: i32) -> i32 {
    ((*SQLITE3_API).overload_function.expect(EXPECT_MESSAGE))(db, s, n)
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_context_db_handle(context: *mut sqlite3_context) -> *mut sqlite3 {
    libsqlite3_sys::sqlite3_context_db_handle(context)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_context_db_handle(context: *mut sqlite3_context) -> *mut sqlite3 {
    ((*SQLITE3_API).context_db_handle.expect(EXPECT_MESSAGE))(context)
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_user_data(context: *mut sqlite3_context) -> *mut c_void {
    libsqlite3_sys::sqlite3_user_data(context)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_user_data(context: *mut sqlite3_context) -> *mut c_void {
    ((*SQLITE3_API).user_data.expect(EXPECT_MESSAGE))(context)
}
#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_mprintf(s: *const c_char) -> *mut c_char {
    libsqlite3_sys::sqlite3_mprintf(s)
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_mprintf(s: *const c_char) -> *mut c_char {
    ((*SQLITE3_API).mprintf.expect(EXPECT_MESSAGE))(s)
}

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_auto_extension(f: unsafe extern "C" fn()) -> i32 {
    libsqlite3_sys::sqlite3_auto_extension(Some(f))
}
#[cfg(not(feature = "static"))]
pub unsafe fn sqlite3ext_auto_extension(f: unsafe extern "C" fn()) -> i32 {
    ((*SQLITE3_API).auto_extension.expect(EXPECT_MESSAGE))(Some(f))
}
