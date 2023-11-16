#![allow(non_snake_case)]
#![allow(unused)]

use crate::ext::{
    sqlite3_file, sqlite3_int64, sqlite3_syscall_ptr, sqlite3_vfs,
};

use sqlite3ext_sys::{
    SQLITE_CANTOPEN_FULLPATH, SQLITE_ERROR, SQLITE_IOERR_ACCESS, SQLITE_IOERR_DELETE, SQLITE_OK, SQLITE_CANTOPEN
};

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::rc::Rc;

use crate::ext::{sqlite3ext_vfs_find, sqlite3ext_vfs_register};
use std::io::{Error, ErrorKind, Result};

use super::traits::SqliteVfs;

pub(crate) fn handle_int(result: Result<c_int>, ext_io_err: Option<c_int>) -> c_int {
    match result {
        Ok(i) => i,
        Err(e) => {
            if let Some(inner_err) = e.into_inner() {
                println!("error: {inner_err}");
            }
            if let Some(extended) = ext_io_err {
                extended
            } else {
                SQLITE_ERROR
            }
        }
    }
}

pub(crate) fn handle_error(result: Result<()>, ext_io_err: Option<c_int>) -> c_int {
    match result {
        Ok(()) => SQLITE_OK,
        Err(e) => {
            if let Some(inner_err) = e.into_inner() {
                println!("error: {inner_err}");
            }
            if let Some(extended) = ext_io_err {
                extended
            } else {
                SQLITE_ERROR
            }
        }
    }
}

unsafe extern "C" fn x_open<T: SqliteVfs>(
    p_vfs: *mut sqlite3_vfs,
    z_name: *const c_char,
    p_file: *mut sqlite3_file,
    flags: c_int,
    p_out_flags: *mut c_int,
) -> c_int {
    let mut vfs = Box::<sqlite3_vfs>::from_raw(p_vfs);
    let mut b = Box::<T>::from_raw(vfs.pAppData.cast::<T>());

    let result = b.open(z_name, p_file, flags, p_out_flags);
    Box::into_raw(b);
    Box::into_raw(vfs);

    handle_error(result, Some(SQLITE_CANTOPEN))
}

unsafe extern "C" fn x_delete<T: SqliteVfs>(
    p_vfs: *mut sqlite3_vfs,
    z_name: *const c_char,
    sync_dir: c_int,
) -> c_int {
    let mut vfs = Box::<sqlite3_vfs>::from_raw(p_vfs);
    let mut b = Box::<T>::from_raw(vfs.pAppData.cast::<T>());

    let result = b.delete(z_name, sync_dir);
    Box::into_raw(b);
    Box::into_raw(vfs);

    handle_error(result, Some(SQLITE_IOERR_DELETE))
}

unsafe extern "C" fn x_access<T: SqliteVfs>(
    p_vfs: *mut sqlite3_vfs,
    z_name: *const c_char,
    flags: c_int,
    p_res_out: *mut c_int,
) -> c_int {
    let mut vfs = Box::<sqlite3_vfs>::from_raw(p_vfs);
    let mut b = Box::<T>::from_raw(vfs.pAppData.cast::<T>());

    let result = b.access(z_name, flags, p_res_out);
    Box::into_raw(b);
    Box::into_raw(vfs);

    handle_error(result, Some(SQLITE_IOERR_ACCESS))
}

unsafe extern "C" fn x_full_pathname<T: SqliteVfs>(
    p_vfs: *mut sqlite3_vfs,
    z_name: *const c_char,
    n_out: c_int,
    z_out: *mut c_char,
) -> c_int {
    let mut vfs = Box::<sqlite3_vfs>::from_raw(p_vfs);
    let mut b = Box::<T>::from_raw(vfs.pAppData.cast::<T>());

    let result = b.full_pathname(z_name, n_out, z_out);
    Box::into_raw(b);
    Box::into_raw(vfs);

    handle_error(result, Some(SQLITE_CANTOPEN_FULLPATH))
}

#[cfg(feature = "vfs_loadext")]
unsafe extern "C" fn x_dl_open<T: SqliteVfs>(
    p_vfs: *mut sqlite3_vfs,
    z_filename: *const c_char,
) -> *mut c_void {
    let mut vfs = Box::<sqlite3_vfs>::from_raw(p_vfs);
    let mut b = Box::<T>::from_raw(vfs.pAppData.cast::<T>());

    let out = b.dl_open(z_filename);
    Box::into_raw(b);
    Box::into_raw(vfs);

    out
}

#[cfg(feature = "vfs_loadext")]
unsafe extern "C" fn x_dl_error<T: SqliteVfs>(
    p_vfs: *mut sqlite3_vfs,
    n_byte: c_int,
    z_err_msg: *mut c_char,
) {
    let mut vfs = Box::<sqlite3_vfs>::from_raw(p_vfs);
    let mut b = Box::<T>::from_raw(vfs.pAppData.cast::<T>());

    b.dl_error(n_byte, z_err_msg);
    Box::into_raw(b);
    Box::into_raw(vfs);
}

#[cfg(feature = "vfs_loadext")]
unsafe extern "C" fn x_dl_sym<T: SqliteVfs>(
    p_vfs: *mut sqlite3_vfs,
    p_handle: *mut c_void,
    z_symbol: *const c_char,
) -> Option<unsafe extern "C" fn(*mut sqlite3_vfs, *mut c_void, *const c_char)> {
    let mut vfs = Box::<sqlite3_vfs>::from_raw(p_vfs);
    let mut b = Box::<T>::from_raw(vfs.pAppData.cast::<T>());

    b.dl_sym(p_handle, z_symbol);
    Box::into_raw(b);
    Box::into_raw(vfs);

    None
}

#[cfg(feature = "vfs_loadext")]
/// Let Boxes go out of scope, thus drop
unsafe extern "C" fn x_dl_close<T: SqliteVfs>(p_vfs: *mut sqlite3_vfs, p_handle: *mut c_void) {
    let mut vfs = Box::<sqlite3_vfs>::from_raw(p_vfs);
    let mut b = Box::<T>::from_raw(vfs.pAppData.cast::<T>());
    b.dl_close(p_handle);
}

unsafe extern "C" fn x_randomness<T: SqliteVfs>(
    p_vfs: *mut sqlite3_vfs,
    n_byte: c_int,
    z_out: *mut c_char,
) -> c_int {
    let mut vfs = Box::<sqlite3_vfs>::from_raw(p_vfs);
    let mut b = Box::<T>::from_raw(vfs.pAppData.cast::<T>());

    let result = b.randomness(n_byte, z_out);
    Box::into_raw(b);
    Box::into_raw(vfs);

    result
}

unsafe extern "C" fn x_sleep<T: SqliteVfs>(p_vfs: *mut sqlite3_vfs, microseconds: c_int) -> c_int {
    let mut vfs = Box::<sqlite3_vfs>::from_raw(p_vfs);
    let mut b = Box::<T>::from_raw(vfs.pAppData.cast::<T>());

    let result = b.sleep(microseconds);
    Box::into_raw(b);
    Box::into_raw(vfs);

    result
}

unsafe extern "C" fn x_current_time<T: SqliteVfs>(
    p_vfs: *mut sqlite3_vfs,
    p_time: *mut f64,
) -> c_int {
    let mut vfs = Box::<sqlite3_vfs>::from_raw(p_vfs);
    let mut b = Box::<T>::from_raw(vfs.pAppData.cast::<T>());

    let result = b.current_time(p_time);
    Box::into_raw(b);
    Box::into_raw(vfs);

    result
}

unsafe extern "C" fn x_get_last_error<T: SqliteVfs>(
    p_vfs: *mut sqlite3_vfs,
    err_code: c_int,
    z_err_msg: *mut c_char,
) -> c_int {
    let mut vfs = Box::<sqlite3_vfs>::from_raw(p_vfs);
    let mut b = Box::<T>::from_raw(vfs.pAppData.cast::<T>());

    let result = b.get_last_error(err_code, z_err_msg);
    Box::into_raw(b);
    Box::into_raw(vfs);

    handle_error(result, None)
}

unsafe extern "C" fn x_current_time_int64<T: SqliteVfs>(
    p_vfs: *mut sqlite3_vfs,
    p_time: *mut sqlite3_int64,
) -> c_int {
    let mut vfs = Box::<sqlite3_vfs>::from_raw(p_vfs);
    let mut b = Box::<T>::from_raw(vfs.pAppData.cast::<T>());

    let result = b.current_time_int64(p_time);
    Box::into_raw(b);
    Box::into_raw(vfs);

    result
}

#[cfg(feature = "vfs_syscall")]
unsafe extern "C" fn x_set_system_call<T: SqliteVfs>(
    p_vfs: *mut sqlite3_vfs,
    z_name: *const c_char,
    p_call: sqlite3_syscall_ptr,
) -> c_int {
    let mut vfs = Box::<sqlite3_vfs>::from_raw(p_vfs);
    let mut b = Box::<T>::from_raw(vfs.pAppData.cast::<T>());

    let result = b.set_system_call(z_name, p_call);
    Box::into_raw(b);
    Box::into_raw(vfs);

    result
}

#[cfg(feature = "vfs_syscall")]
unsafe extern "C" fn x_get_system_call<T: SqliteVfs>(
    p_vfs: *mut sqlite3_vfs,
    z_name: *const c_char,
) -> sqlite3_syscall_ptr {
    let mut vfs = Box::<sqlite3_vfs>::from_raw(p_vfs);
    let mut b = Box::<T>::from_raw(vfs.pAppData.cast::<T>());

    let result = b.get_system_call(z_name);
    Box::into_raw(b);
    Box::into_raw(vfs);

    result
}

#[cfg(feature = "vfs_syscall")]
unsafe extern "C" fn x_next_system_call<T: SqliteVfs>(
    p_vfs: *mut sqlite3_vfs,
    z_name: *const c_char,
) -> *const c_char {
    let mut vfs = Box::<sqlite3_vfs>::from_raw(p_vfs);
    let mut b = Box::<T>::from_raw(vfs.pAppData.cast::<T>());

    let result = b.next_system_call(z_name);
    Box::into_raw(b);
    Box::into_raw(vfs);

    result
}

pub fn create_vfs<T: SqliteVfs>(
    aux: T,
    name_ptr: *const c_char,
    max_path_name_size: i32,
    vfs_file_size: i32,
) -> sqlite3_vfs {
    unsafe {
        let vfs_ptr = Box::into_raw(Box::<T>::new(aux));

        /// According to the documentation:
        /// At least vfs_file_size bytes of memory are allocated by SQLite to hold the sqlite3_file
        /// structure passed as the third argument to xOpen. The xOpen method does not have to
        /// allocate the structure; it should just fill it in.
        sqlite3_vfs {
            iVersion: 3,
            pNext: ptr::null_mut(),
            pAppData: vfs_ptr.cast(),
            // raw box pointers sizes are all the same
            szOsFile: vfs_file_size,
            mxPathname: max_path_name_size,
            zName: name_ptr,

            xOpen: Some(x_open::<T>),
            xDelete: Some(x_delete::<T>),
            xAccess: Some(x_access::<T>),
            xFullPathname: Some(x_full_pathname::<T>),

            /// The following four VFS methods:
            ///
            ///   xDlOpen
            ///   xDlError
            ///   xDlSym
            ///   xDlClose
            ///
            /// are supposed to implement the functionality needed by SQLite to load
            /// extensions compiled as shared objects.
            #[cfg(feature = "vfs_loadext")]
            xDlOpen: Some(x_dl_open::<T>),
            #[cfg(feature = "vfs_loadext")]
            xDlError: Some(x_dl_error::<T>),
            #[cfg(feature = "vfs_loadext")]
            xDlSym: Some(x_dl_sym::<T>),
            #[cfg(feature = "vfs_loadext")]
            xDlClose: Some(x_dl_close::<T>),

            #[cfg(not(feature = "vfs_loadext"))]
            xDlOpen: None,
            #[cfg(not(feature = "vfs_loadext"))]
            xDlError: None,
            #[cfg(not(feature = "vfs_loadext"))]
            xDlSym: None,
            #[cfg(not(feature = "vfs_loadext"))]
            xDlClose: None,

            xRandomness: Some(x_randomness::<T>),
            xSleep: Some(x_sleep::<T>),
            xCurrentTime: Some(x_current_time::<T>),
            xGetLastError: Some(x_get_last_error::<T>),
            xCurrentTimeInt64: Some(x_current_time_int64::<T>),

            #[cfg(feature = "vfs_syscall")]
            xSetSystemCall: Some(x_set_system_call::<T>),
            #[cfg(feature = "vfs_syscall")]
            xGetSystemCall: Some(x_get_system_call::<T>),
            #[cfg(feature = "vfs_syscall")]
            xNextSystemCall: Some(x_next_system_call::<T>),

            #[cfg(not(feature = "vfs_syscall"))]
            xSetSystemCall: None,
            #[cfg(not(feature = "vfs_syscall"))]
            xGetSystemCall: None,
            #[cfg(not(feature = "vfs_syscall"))]
            xNextSystemCall: None,
        }
    }
}

fn handle_vfs_result(result: i32) -> crate::Result<()> {
    if result == SQLITE_OK {
        Ok(())
    } else {
        Err(crate::errors::Error::new_message(format!(
            "sqlite3_vfs_register failed with error code: {}",
            result
        )))
    }
}

pub fn register_boxed_vfs(vfs: sqlite3_vfs, make_default: bool) -> crate::Result<()> {
    let translate_to_int = if make_default { 1 } else { 0 };

    let boxed_vfs = Box::into_raw(Box::new(vfs));

    let result = unsafe { sqlite3ext_vfs_register(boxed_vfs, translate_to_int) };

    handle_vfs_result(result)
}
