//! Utilities for working with SQLite's "sqlite3_extension_init"-style
//! entrypoints.
use crate::{
    errors::Result,
    ext::{faux_sqlite_extension_init2, sqlite3, sqlite3_api_routines},
};

use sqlite3ext_sys::SQLITE_OK;

use std::os::raw::{c_char, c_uint};

/// Low-level wrapper around a typical entrypoint to a SQLite extension.
/// You shouldn't have to use this directly - the sqlite_entrypoint
/// macro will do this for you.
pub fn register_entrypoint<F>(
    db: *mut sqlite3,
    _pz_err_msg: *mut *mut c_char,
    p_api: *mut sqlite3_api_routines,
    callback: F,
) -> c_uint
where
    F: Fn(*mut sqlite3) -> Result<()>,
{
    unsafe {
        faux_sqlite_extension_init2(p_api);
    }
    match callback(db) {
        Ok(()) => SQLITE_OK,
        Err(err) => err.code_extended(),
    }
}

/// Low-level wrapper around an entrypoint to a SQLite extension that loads permanently.  You
/// shouldn't have to use this directly - the sqlite_entrypoint_permanent macro will do this
/// for you.
pub fn register_entrypoint_load_permanently<F>(
    db: *mut sqlite3,
    _pz_err_msg: *mut *mut c_char,
    p_api: *mut sqlite3_api_routines,
    callback: F,
) -> c_uint
where
    F: Fn(*mut sqlite3) -> Result<()>,
{
    unsafe {
        faux_sqlite_extension_init2(p_api);
    }
    match callback(db) {
        Ok(()) => 256, // https://www.sqlite.org/rescode.html#ok_load_permanently
        Err(err) => err.code_extended(),
    }
}
