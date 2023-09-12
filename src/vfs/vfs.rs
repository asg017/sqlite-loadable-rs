#![ allow(non_snake_case)]
#![ allow(unused)]

use sqlite3ext_sys::{sqlite3_file, sqlite3_int64, sqlite3_vfs, sqlite3_syscall_ptr, sqlite3_vfs_register, sqlite3_vfs_find};
use std::ffi::CString;
use std::os::raw::{c_int, c_char, c_void};
use std::ptr;

use super::traits::SqliteVfs;

pub unsafe extern "C" fn x_open<T: SqliteVfs>(
    p_vfs: *mut sqlite3_vfs,
    z_name: *const c_char,
    p_file: *mut sqlite3_file,
    flags: c_int,
    p_out_flags: *mut c_int,
) -> c_int {
    let mut b = Box::<dyn SqliteVfs>::from_raw(p_vfs.cast::<T>());
    
    match b.open(z_name, p_file, flags, p_out_flags) {
        Ok(()) => (),
        Err(e) => {
            // TODO define error handling
            // if api::result_error(context, &e.result_error_message()).is_err() {
            //     api::result_error_code(context, SQLITE_INTERNAL);
            // }
        }
    }
    Box::into_raw(b); // Drop in close
    0 // TODO figure out what to do here
}

pub unsafe extern "C" fn x_delete<T: SqliteVfs>(p_vfs: *mut sqlite3_vfs, z_name: *const c_char, sync_dir: c_int) -> c_int {
    let mut b = Box::<dyn SqliteVfs>::from_raw(p_vfs.cast::<T>());
    match b.delete(z_name, sync_dir) {
        Ok(()) => (),
        Err(e) => {
            // TODO define error handling
            // if api::result_error(context, &e.result_error_message()).is_err() {
            //     api::result_error_code(context, SQLITE_INTERNAL);
            // }
        }
    }
    Box::into_raw(b); // Drop in close
    0 // TODO figure out what to do here
}

pub unsafe extern "C" fn x_access<T: SqliteVfs>(p_vfs: *mut sqlite3_vfs, z_name: *const c_char, flags: c_int, p_res_out: *mut c_int) -> c_int {
    let mut b = Box::<dyn SqliteVfs>::from_raw(p_vfs.cast::<T>());
    match b.access(z_name, flags, p_res_out) {
        Ok(()) => (),
        Err(e) => {
            // TODO define error handling
            // if api::result_error(context, &e.result_error_message()).is_err() {
            //     api::result_error_code(context, SQLITE_INTERNAL);
            // }
        }
    }
    Box::into_raw(b); // Drop in close
    0 // TODO figure out what to do here
}

pub unsafe extern "C" fn x_full_pathname<T: SqliteVfs>(p_vfs: *mut sqlite3_vfs, z_name: *const c_char, n_out: c_int, z_out: *mut c_char) -> c_int {
    let mut b = Box::<dyn SqliteVfs>::from_raw(p_vfs.cast::<T>());
    match b.full_pathname(z_name, n_out, z_out) {
        Ok(()) => (),
        Err(e) => {
            // TODO define error handling
            // if api::result_error(context, &e.result_error_message()).is_err() {
            //     api::result_error_code(context, SQLITE_INTERNAL);
            // }
        }
    }
    Box::into_raw(b); // Drop in close
    0 // TODO figure out what to do here
}

pub unsafe extern "C" fn x_dl_open<T: SqliteVfs>(p_vfs: *mut sqlite3_vfs, z_filename: *const c_char) -> *mut c_void {
    let mut b = Box::<dyn SqliteVfs>::from_raw(p_vfs.cast::<T>());
    let out = b.dl_open(z_filename);
    // TODO
    // match b.dl_open(z_filename) {
    //     Ok(()) => (),
    //     Err(e) => {
    //         // TODO define error handling
    //         // if api::result_error(context, &e.result_error_message()).is_err() {
    //         //     api::result_error_code(context, SQLITE_INTERNAL);
    //         // }
    //     }
    // }
    Box::into_raw(b); // Drop in close
    out
}

pub unsafe extern "C" fn x_dl_error<T: SqliteVfs>(p_vfs: *mut sqlite3_vfs, n_byte: c_int, z_err_msg: *mut c_char) {
    let mut b = Box::<dyn SqliteVfs>::from_raw(p_vfs.cast::<T>());
    b.dl_error(n_byte, z_err_msg);
    // match b.dl_error(n_byte, z_err_msg) {
    //     Ok(()) => (),
    //     Err(e) => {
    //         // TODO define error handling
    //         // if api::result_error(context, &e.result_error_message()).is_err() {
    //         //     api::result_error_code(context, SQLITE_INTERNAL);
    //         // }
    //     }
    // }
    Box::into_raw(b); // Drop in close
}

pub unsafe extern "C" fn x_dl_sym<T: SqliteVfs>(
    p_vfs: *mut sqlite3_vfs,
    p_handle: *mut c_void,
    z_symbol: *const c_char,
) -> Option<unsafe extern "C" fn(*mut sqlite3_vfs, *mut c_void, *const c_char)> {
    let mut b = Box::<dyn SqliteVfs>::from_raw(p_vfs.cast::<T>());
    b.dl_sym(p_handle, z_symbol);
    // match b.dl_error(n_byte, z_err_msg) {
    //     Ok(()) => (),
    //     Err(e) => {
    //         // TODO define error handling
    //         // if api::result_error(context, &e.result_error_message()).is_err() {
    //         //     api::result_error_code(context, SQLITE_INTERNAL);
    //         // }
    //     }
    // }
    Box::into_raw(b); // Drop in close
    None
}

/// Let Box go out of scope, thus drop // TODO valgrind
pub unsafe extern "C" fn x_dl_close<T: SqliteVfs>(p_vfs: *mut sqlite3_vfs, p_handle: *mut c_void) {
    let mut b = Box::<dyn SqliteVfs>::from_raw(p_vfs.cast::<T>());
    b.dl_close(p_handle);
    // match b.dl_close(p_handle) {
    //     Ok(()) => (),
    //     Err(e) => {
    //         // TODO define error handling
    //         // if api::result_error(context, &e.result_error_message()).is_err() {
    //         //     api::result_error_code(context, SQLITE_INTERNAL);
    //         // }
    //     }
    // }
}

pub unsafe extern "C" fn x_randomness<T: SqliteVfs>(p_vfs: *mut sqlite3_vfs, n_byte: c_int, z_out: *mut c_char) -> c_int {
    let mut b = Box::<dyn SqliteVfs>::from_raw(p_vfs.cast::<T>());
    match b.randomness(n_byte, z_out) {
        Ok(()) => (),
        Err(e) => {
            // TODO define error handling
            // if api::result_error(context, &e.result_error_message()).is_err() {
            //     api::result_error_code(context, SQLITE_INTERNAL);
            // }
        }
    }
    Box::into_raw(b); // Drop in close
    0 // TODO figure out what to do here
}

pub unsafe extern "C" fn x_sleep<T: SqliteVfs>(p_vfs: *mut sqlite3_vfs, microseconds: c_int) -> c_int {
    let mut b = Box::<dyn SqliteVfs>::from_raw(p_vfs.cast::<T>());
    match b.sleep(microseconds) {
        Ok(()) => (),
        Err(e) => {
            // TODO define error handling
            // if api::result_error(context, &e.result_error_message()).is_err() {
            //     api::result_error_code(context, SQLITE_INTERNAL);
            // }
        }
    }
    Box::into_raw(b); // Drop in close
    0 // TODO figure out what to do here
}

pub unsafe extern "C" fn x_current_time<T: SqliteVfs>(p_vfs: *mut sqlite3_vfs, p_time: *mut f64) -> c_int {
    let mut b = Box::<dyn SqliteVfs>::from_raw(p_vfs.cast::<T>());
    match b.current_time(p_time) {
        Ok(()) => (),
        Err(e) => {
            // TODO define error handling
            // if api::result_error(context, &e.result_error_message()).is_err() {
            //     api::result_error_code(context, SQLITE_INTERNAL);
            // }
        }
    }
    Box::into_raw(b); // Drop in close
    0 // TODO figure out what to do here
}

pub unsafe extern "C" fn x_get_last_error<T: SqliteVfs>(
    p_vfs: *mut sqlite3_vfs,
    err_code: c_int,
    z_err_msg: *mut c_char,
) -> c_int {
    let mut b = Box::<dyn SqliteVfs>::from_raw(p_vfs.cast::<T>());
    match b.get_last_error(err_code, z_err_msg) {
        Ok(()) => (),
        Err(e) => {
            // TODO define error handling
            // if api::result_error(context, &e.result_error_message()).is_err() {
            //     api::result_error_code(context, SQLITE_INTERNAL);
            // }
        }
    }
    Box::into_raw(b); // Drop in close
    0 // TODO figure out what to do here
}

pub unsafe extern "C" fn x_current_time_int64<T: SqliteVfs>(
    p_vfs: *mut sqlite3_vfs,
    p_time: *mut sqlite3_int64,
) -> c_int {
    let mut b = Box::<dyn SqliteVfs>::from_raw(p_vfs.cast::<T>());
    match b.current_time_int64(p_time) {
        Ok(()) => (),
        Err(e) => {
            // TODO define error handling
            // if api::result_error(context, &e.result_error_message()).is_err() {
            //     api::result_error_code(context, SQLITE_INTERNAL);
            // }
        }
    }
    Box::into_raw(b); // Drop in close
    0 // TODO figure out what to do here
}

pub unsafe extern "C" fn x_set_system_call<T: SqliteVfs>(
    p_vfs: *mut sqlite3_vfs,
    z_name: *const c_char,
    p_call: sqlite3_syscall_ptr,
) -> c_int {
    let mut b = Box::<dyn SqliteVfs>::from_raw(p_vfs.cast::<T>());
    match b.set_system_call(z_name, p_call) {
        Ok(()) => (),
        Err(e) => {
            // TODO define error handling
            // if api::result_error(context, &e.result_error_message()).is_err() {
            //     api::result_error_code(context, SQLITE_INTERNAL);
            // }
        }
    }
    Box::into_raw(b); // Drop in close
    0 // TODO figure out what to do here
}

pub unsafe extern "C" fn x_get_system_call<T: SqliteVfs>(
    p_vfs: *mut sqlite3_vfs,
    z_name: *const c_char,
) -> sqlite3_syscall_ptr {
    let mut b = Box::<dyn SqliteVfs>::from_raw(p_vfs.cast::<T>());
    let out = b.get_system_call(z_name);
    // match b.get_system_call(z_name) {
    //     Ok(()) => (),
    //     Err(e) => {
    //         // TODO define error handling
    //         // if api::result_error(context, &e.result_error_message()).is_err() {
    //         //     api::result_error_code(context, SQLITE_INTERNAL);
    //         // }
    //     }
    // }
    Box::into_raw(b); // Drop in close
    // None // TODO figure out what to do here
    out
}

pub unsafe extern "C" fn x_next_system_call<T: SqliteVfs>(
    p_vfs: *mut sqlite3_vfs,
    z_name: *const c_char,
) -> *const c_char {
    let mut b = Box::<dyn SqliteVfs>::from_raw(p_vfs.cast::<T>());
    // match b.next_system_call(z_name) {
    //     Ok(()) => (),
    //     Err(e) => {
    //         // TODO define error handling
    //         // if api::result_error(context, &e.result_error_message()).is_err() {
    //         //     api::result_error_code(context, SQLITE_INTERNAL);
    //         // }
    //     }
    // }
    Box::into_raw(b); // Drop in close
    ptr::null() // TODO
}



pub fn create_vfs<T: SqliteVfs>(vfs: T, name: &str, max_path_name_size: i32) -> sqlite3_vfs {
    unsafe {
        let default_vfs_ptr = sqlite3_vfs_find(ptr::null());
        let vfs_ptr = Box::into_raw(Box::<T>::new(vfs));
        // TODO also put sqlite3_file with vfs
        let size_ptr = std::mem::size_of::<*mut T>(); // this should remain the same
        let vfs_name = CString::new(name)
            .expect("should be a C string").as_ptr().to_owned();
    
        sqlite3_vfs {
            iVersion: 3, // this library targets version 3?
            pNext: ptr::null_mut(), // sqlite3 will change this
            pAppData: vfs_ptr.cast(),
            szOsFile: size_ptr as i32,
            mxPathname: max_path_name_size,
            zName: vfs_name,
    
            // TODO some are optional, break down to multiple traits?
            // TODO investigate: maybe use attribute to generate a static dispatch type, if it's too slow
            xOpen: Some(x_open::<T>),
            xDelete: Some(x_delete::<T>),
            xAccess: Some(x_access::<T>),
            xFullPathname: Some(x_full_pathname::<T>),
            xDlOpen: Some(x_dl_open::<T>),
            xDlError: Some(x_dl_error::<T>),
            xDlSym: Some(x_dl_sym::<T>),
            xDlClose: Some(x_dl_close::<T>),
            xRandomness: Some(x_randomness::<T>),
            xSleep: Some(x_sleep::<T>),
            xCurrentTime: Some(x_current_time::<T>),
            xGetLastError: Some(x_get_last_error::<T>),
            xCurrentTimeInt64: Some(x_current_time_int64::<T>),
            xSetSystemCall: Some(x_set_system_call::<T>),
            xGetSystemCall: Some(x_get_system_call::<T>),
            xNextSystemCall: Some(x_next_system_call::<T>),
        }
    
    }
}



pub fn register_vfs(vfs: sqlite3_vfs, make_default: bool) -> Result<(), String> {
    let translate_to_int = if make_default { 1 } else { 0 };

    let result = unsafe { sqlite3_vfs_register(Box::into_raw(Box::new(vfs)),
        translate_to_int) };
    
    if result == 0 {
        Ok(())
    } else {
        Err(format!("sqlite3_vfs_register failed with error code: {}", result))
    }
}