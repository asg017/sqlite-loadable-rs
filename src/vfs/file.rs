use sqlite3ext_sys::{sqlite3_file, sqlite3_int64, sqlite3_vfs, sqlite3_syscall_ptr};
use std::os::raw::{c_int, c_char, c_void};
use std::ptr;

use super::traits::SqliteVfs;


pub fn define_vfs<T>() where T: SqliteVfs {
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
        Box::into_raw(b); // TODO drop in close
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
        Box::into_raw(b); // TODO drop in close
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
        Box::into_raw(b); // TODO drop in close
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
        Box::into_raw(b); // TODO drop in close
        0 // TODO figure out what to do here
    }
    
    pub unsafe extern "C" fn x_dl_open<T: SqliteVfs>(p_vfs: *mut sqlite3_vfs, z_filename: *const c_char) -> *mut c_void {
        let mut b = Box::<dyn SqliteVfs>::from_raw(p_vfs.cast::<T>());
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
        Box::into_raw(b); // TODO drop in close
        ptr::null_mut()
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
        Box::into_raw(b); // TODO drop in close
    }
    
    pub unsafe extern "C" fn x_dl_sym<T: SqliteVfs>(
        p_vfs: *mut sqlite3_vfs,
        p_handle: *mut c_void,
        z_symbol: *const c_char,
    ) -> Option<unsafe extern "C" fn(*mut sqlite3_vfs, *mut c_void, *const c_char)> {
        None
    }
    
    pub unsafe extern "C" fn x_dl_close<T: SqliteVfs>(p_vfs: *mut sqlite3_vfs, p_handle: *mut c_void) {
        let mut b = Box::<dyn SqliteVfs>::from_raw(p_vfs.cast::<T>());
        // match b.dl_close(p_handle) {
        //     Ok(()) => (),
        //     Err(e) => {
        //         // TODO define error handling
        //         // if api::result_error(context, &e.result_error_message()).is_err() {
        //         //     api::result_error_code(context, SQLITE_INTERNAL);
        //         // }
        //     }
        // }
        Box::into_raw(b); // TODO drop in close
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
        Box::into_raw(b); // TODO drop in close
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
        Box::into_raw(b); // TODO drop in close
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
        Box::into_raw(b); // TODO drop in close
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
        Box::into_raw(b); // TODO drop in close
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
        Box::into_raw(b); // TODO drop in close
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
        Box::into_raw(b); // TODO drop in close
        0 // TODO figure out what to do here
    }
    
    pub unsafe extern "C" fn x_get_system_call<T: SqliteVfs>(
        p_vfs: *mut sqlite3_vfs,
        z_name: *const c_char,
    ) -> sqlite3_syscall_ptr {
        let mut b = Box::<dyn SqliteVfs>::from_raw(p_vfs.cast::<T>());
        // match b.get_system_call(z_name) {
        //     Ok(()) => (),
        //     Err(e) => {
        //         // TODO define error handling
        //         // if api::result_error(context, &e.result_error_message()).is_err() {
        //         //     api::result_error_code(context, SQLITE_INTERNAL);
        //         // }
        //     }
        // }
        Box::into_raw(b); // TODO drop in close
        None // TODO figure out what to do here
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
        Box::into_raw(b); // TODO drop in close
        ptr::null() // TODO
    }
}
