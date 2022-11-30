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

use sqlite3ext_sys::{
    sqlite3, sqlite3_api_routines, sqlite3_bind_text, sqlite3_column_text, sqlite3_column_value,
    sqlite3_context, sqlite3_create_function_v2, sqlite3_create_module_v2, sqlite3_declare_vtab,
    sqlite3_finalize, sqlite3_get_auxdata, sqlite3_index_info, sqlite3_module, sqlite3_prepare_v2,
    sqlite3_result_blob, sqlite3_result_double, sqlite3_result_error, sqlite3_result_error_code,
    sqlite3_result_int, sqlite3_result_int64, sqlite3_result_null, sqlite3_result_pointer,
    sqlite3_result_text, sqlite3_set_auxdata, sqlite3_step, sqlite3_stmt, sqlite3_value,
    sqlite3_value_blob, sqlite3_value_bytes, sqlite3_value_double, sqlite3_value_int,
    sqlite3_value_int64, sqlite3_value_pointer, sqlite3_value_text, sqlite3_value_type,
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
    SQLITE3_API = api;
}

//definex!("value_text", c_uchar);
static EXPECT_MESSAGE: &str =
    "sqlite-loadable error: expected method on SQLITE3_API. Please file an issue";

pub unsafe fn sqlite3ext_value_text(arg1: *mut sqlite3_value) -> *const ::std::os::raw::c_uchar {
    if SQLITE3_API.is_null() {
        return sqlite3_value_text(arg1);
    }
    ((*SQLITE3_API).value_text.expect(EXPECT_MESSAGE))(arg1)
}

pub unsafe fn sqlite3ext_value_type(value: *mut sqlite3_value) -> i32 {
    if SQLITE3_API.is_null() {
        return sqlite3_value_type(value);
    }
    ((*SQLITE3_API).value_type.expect(EXPECT_MESSAGE))(value)
}

pub unsafe fn sqlite3ext_value_bytes(arg1: *mut sqlite3_value) -> i32 {
    if SQLITE3_API.is_null() {
        return sqlite3_value_bytes(arg1);
    }
    ((*SQLITE3_API).value_bytes.expect(EXPECT_MESSAGE))(arg1)
}

pub unsafe fn sqlite3ext_value_blob(arg1: *mut sqlite3_value) -> *const c_void {
    if SQLITE3_API.is_null() {
        return sqlite3_value_blob(arg1);
    }
    ((*SQLITE3_API).value_blob.expect(EXPECT_MESSAGE))(arg1)
}

pub unsafe fn sqlite3ext_bind_pointer(
    db: *mut sqlite3_stmt,
    i: i32,
    p: *mut c_void,
    t: *const c_char,
) -> i32 {
    ((*SQLITE3_API).bind_pointer.expect(EXPECT_MESSAGE))(db, i, p, t, None)
}
pub unsafe fn sqlite3ext_step(stmt: *mut sqlite3_stmt) -> c_int {
    if SQLITE3_API.is_null() {
        return sqlite3_step(stmt);
    }
    ((*SQLITE3_API).step.expect(EXPECT_MESSAGE))(stmt)
}

pub unsafe fn sqlite3ext_finalize(stmt: *mut sqlite3_stmt) -> c_int {
    if SQLITE3_API.is_null() {
        return sqlite3_finalize(stmt);
    }
    ((*SQLITE3_API).finalize.expect(EXPECT_MESSAGE))(stmt)
}

pub unsafe fn sqlite3ext_column_text(stmt: *mut sqlite3_stmt, c: c_int) -> *const c_uchar {
    if SQLITE3_API.is_null() {
        return sqlite3_column_text(stmt, c);
    }
    ((*SQLITE3_API).column_text.expect(EXPECT_MESSAGE))(stmt, c)
}

pub unsafe fn sqlite3ext_column_value(stmt: *mut sqlite3_stmt, c: c_int) -> *mut sqlite3_value {
    if SQLITE3_API.is_null() {
        return sqlite3_column_value(stmt, c);
    }
    ((*SQLITE3_API).column_value.expect(EXPECT_MESSAGE))(stmt, c)
}

pub unsafe fn sqlite3ext_bind_text(
    stmt: *mut sqlite3_stmt,
    c: c_int,
    s: *const c_char,
    n: c_int,
) -> i32 {
    if SQLITE3_API.is_null() {
        return sqlite3_bind_text(stmt, c, s, n, None);
    }
    ((*SQLITE3_API).bind_text.expect(EXPECT_MESSAGE))(stmt, c, s, n, None)
}

pub unsafe fn sqlite3ext_prepare_v2(
    db: *mut sqlite3,
    sql: *const c_char,
    n: i32,
    stmt: *mut *mut sqlite3_stmt,
    leftover: *mut *const c_char,
) -> i32 {
    if SQLITE3_API.is_null() {
        return sqlite3_prepare_v2(db, sql, n, stmt, leftover);
    }
    ((*SQLITE3_API).prepare_v2.expect(EXPECT_MESSAGE))(db, sql, n, stmt, leftover)
}

pub unsafe fn sqlite3ext_value_int(arg1: *mut sqlite3_value) -> i32 {
    if SQLITE3_API.is_null() {
        return sqlite3_value_int(arg1);
    }
    ((*SQLITE3_API).value_int.expect(EXPECT_MESSAGE))(arg1)
}

pub unsafe fn sqlite3ext_value_int64(arg1: *mut sqlite3_value) -> i64 {
    if SQLITE3_API.is_null() {
        return sqlite3_value_int64(arg1);
    }
    ((*SQLITE3_API).value_int64.expect(EXPECT_MESSAGE))(arg1)
}

pub unsafe fn sqlite3ext_value_double(arg1: *mut sqlite3_value) -> f64 {
    if SQLITE3_API.is_null() {
        return sqlite3_value_double(arg1);
    }
    ((*SQLITE3_API).value_double.expect(EXPECT_MESSAGE))(arg1)
}

pub unsafe fn sqlite3ext_value_pointer(arg1: *mut sqlite3_value, p: *mut c_char) -> *mut c_void {
    if SQLITE3_API.is_null() {
        return sqlite3_value_pointer(arg1, p);
    }
    ((*SQLITE3_API).value_pointer.expect(EXPECT_MESSAGE))(arg1, p)
}

pub unsafe fn sqlite3ext_result_int(context: *mut sqlite3_context, v: c_int) {
    if SQLITE3_API.is_null() {
        sqlite3_result_int(context, v);
    } else {
        ((*SQLITE3_API).result_int.expect(EXPECT_MESSAGE))(context, v);
    }
}

pub unsafe fn sqlite3ext_result_blob(context: *mut sqlite3_context, p: *const c_void, n: i32) {
    if SQLITE3_API.is_null() {
        sqlite3_result_blob(context, p, n, Some(mem::transmute(-1_isize)));
    } else {
        ((*SQLITE3_API).result_blob.expect(EXPECT_MESSAGE))(
            context,
            p,
            n,
            Some(mem::transmute(-1_isize)),
        );
    }
}
pub unsafe fn sqlite3ext_result_int64(context: *mut sqlite3_context, v: i64) {
    if SQLITE3_API.is_null() {
        sqlite3_result_int64(context, v);
    } else {
        ((*SQLITE3_API).result_int64.expect(EXPECT_MESSAGE))(context, v);
    }
}

pub unsafe fn sqlite3ext_result_double(context: *mut sqlite3_context, f: f64) {
    if SQLITE3_API.is_null() {
        sqlite3_result_double(context, f);
    } else {
        ((*SQLITE3_API).result_double.expect(EXPECT_MESSAGE))(context, f);
    }
}

pub unsafe fn sqlite3ext_result_null(context: *mut sqlite3_context) {
    if SQLITE3_API.is_null() {
        sqlite3_result_null(context);
    } else {
        ((*SQLITE3_API).result_null.expect(EXPECT_MESSAGE))(context);
    }
}
pub unsafe fn sqlite3ext_result_pointer(
    context: *mut sqlite3_context,
    pointer: *mut c_void,
    name: *mut c_char,
    destructor: Option<unsafe extern "C" fn(*mut std::ffi::c_void)>,
) {
    if SQLITE3_API.is_null() {
        sqlite3_result_pointer(context, pointer, name, destructor);
    } else {
        ((*SQLITE3_API).result_pointer.expect(EXPECT_MESSAGE))(context, pointer, name, destructor);
    }
}

pub unsafe fn sqlite3ext_result_error(context: *mut sqlite3_context, s: *const i8, n: i32) {
    if SQLITE3_API.is_null() {
        sqlite3_result_error(context, s, n);
    } else {
        ((*SQLITE3_API).result_error.expect(EXPECT_MESSAGE))(context, s, n);
    }
}

pub unsafe fn sqlite3ext_result_error_code(context: *mut sqlite3_context, code: i32) {
    if SQLITE3_API.is_null() {
        sqlite3_result_error_code(context, code);
    } else {
        ((*SQLITE3_API).result_error_code.expect(EXPECT_MESSAGE))(context, code);
    }
}
pub unsafe fn sqlite3ext_result_text(
    context: *mut sqlite3_context,
    s: *const i8,
    n: i32,
    d: Option<unsafe extern "C" fn(*mut c_void)>,
) {
    if SQLITE3_API.is_null() {
        sqlite3_result_text(context, s, n, d);
    } else {
        ((*SQLITE3_API).result_text.expect(EXPECT_MESSAGE))(context, s, n, d);
    }
}

pub unsafe fn sqlite3ext_result_subtype(context: *mut sqlite3_context, subtype: u32) {
    if SQLITE3_API.is_null() {
        //sqlite3_result_int(context, v);
    } else {
        ((*SQLITE3_API).result_subtype.expect(EXPECT_MESSAGE))(context, subtype);
    }
}

pub unsafe fn sqlite3ext_set_auxdata(
    context: *mut sqlite3_context,
    n: c_int,
    p: *mut c_void,
    d: Option<unsafe extern "C" fn(*mut c_void)>,
) {
    if SQLITE3_API.is_null() {
        sqlite3_set_auxdata(context, n, p, d);
    } else {
        ((*SQLITE3_API).set_auxdata.expect(EXPECT_MESSAGE))(context, n, p, d);
    }
}

pub unsafe fn sqlite3ext_get_auxdata(context: *mut sqlite3_context, n: c_int) -> *mut c_void {
    if SQLITE3_API.is_null() {
        return sqlite3_get_auxdata(context, n);
    }
    ((*SQLITE3_API).get_auxdata.expect(EXPECT_MESSAGE))(context, n)
}

pub unsafe fn sqlite3ext_create_function_v2(
    db: *mut sqlite3,
    s: *const i8,
    argc: i32,
    text_rep: i32,
    p_app: *mut c_void,
    x_func: Option<unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value)>,
    x_step: Option<unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value)>,
    x_final: Option<unsafe extern "C" fn(*mut sqlite3_context)>,
    destroy: Option<unsafe extern "C" fn(*mut c_void)>,
) -> c_int {
    if SQLITE3_API.is_null() {
        sqlite3_create_function_v2(
            db, s, argc, text_rep, p_app, x_func, x_step, x_final, destroy,
        )
    } else {
        ((*SQLITE3_API).create_function_v2.expect(EXPECT_MESSAGE))(
            db, s, argc, text_rep, p_app, x_func, x_step, x_final, destroy,
        )
    }
}

pub unsafe fn sqlite3ext_create_module_v2(
    db: *mut sqlite3,
    s: *const i8,
    module: *const sqlite3_module,
    p_app: *mut c_void,
    destroy: Option<unsafe extern "C" fn(*mut c_void)>,
) -> i32 {
    if SQLITE3_API.is_null() {
        sqlite3_create_module_v2(db, s, module, p_app, destroy)
    } else {
        ((*SQLITE3_API).create_module_v2.expect(EXPECT_MESSAGE))(db, s, module, p_app, destroy)
    }
}

pub unsafe fn sqlite3ext_vtab_distinct(index_info: *mut sqlite3_index_info) -> i32 {
    ((*SQLITE3_API).vtab_distinct.expect(EXPECT_MESSAGE))(index_info)
}

pub unsafe fn sqlitex_declare_vtab(db: *mut sqlite3, s: *const i8) -> i32 {
    if SQLITE3_API.is_null() {
        return sqlite3_declare_vtab(db, s);
    }
    ((*SQLITE3_API).declare_vtab.expect(EXPECT_MESSAGE))(db, s)
}
