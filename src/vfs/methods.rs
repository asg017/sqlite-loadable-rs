use sqlite3ext_sys::{sqlite3_file, sqlite3_int64, sqlite3_io_methods};
use std::os::raw::{c_int, c_void};

use crate::vfs::traits::SqliteIoMethods;

pub fn empty_sqlite3_io_methods() -> sqlite3_io_methods {
    sqlite3_io_methods {
        iVersion: 0,
        xClose: None,
        xRead: None,
        xWrite: None,
        xTruncate: None,
        xSync: None,
        xFileSize: None,
        xLock: None,
        xUnlock: None,
        xCheckReservedLock: None,
        xFileControl: None,
        xSectorSize: None,
        xDeviceCharacteristics: None,
        xShmMap: None,
        xShmLock: None,
        xShmBarrier: None,
        xShmUnmap: None,
        xFetch: None,
        xUnfetch: None,
    }
}


pub fn define_io_methods<T: SqliteIoMethods>() {
    pub unsafe extern "C" fn x_close<T: SqliteIoMethods>(arg1: *mut sqlite3_file) -> c_int {
        let mut b = Box::<dyn SqliteIoMethods>::from_raw(arg1.cast::<T>());
        match b.close() {
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
    
    pub unsafe extern "C" fn x_read<T: SqliteIoMethods>(
        arg1: *mut sqlite3_file,
        buf: *mut c_void,
        iAmt: c_int,
        iOfst: sqlite3_int64,
    ) -> c_int {
        let mut b = Box::<dyn SqliteIoMethods>::from_raw(arg1.cast::<T>());
        match b.read(buf, iAmt, iOfst) {
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
    
    pub unsafe extern "C" fn x_write<T: SqliteIoMethods>(
        arg1: *mut sqlite3_file,
        buf: *const c_void,
        iAmt: c_int,
        iOfst: sqlite3_int64,
    ) -> c_int {
        let mut b = Box::<dyn SqliteIoMethods>::from_raw(arg1.cast::<T>());
        match b.write(buf, iAmt, iOfst) {
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
    
    pub unsafe extern "C" fn x_truncate<T: SqliteIoMethods>(
        arg1: *mut sqlite3_file,
        size: sqlite3_int64,
    ) -> c_int {
        let mut b = Box::<dyn SqliteIoMethods>::from_raw(arg1.cast::<T>());
        match b.truncate(size) {
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
    
    pub unsafe extern "C" fn x_sync<T: SqliteIoMethods>(
        arg1: *mut sqlite3_file, // TODO convert
        flags: c_int,
    ) -> c_int {
        let mut b = Box::<dyn SqliteIoMethods>::from_raw(arg1.cast::<T>());
        match b.sync(flags) {
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
    
    pub unsafe extern "C" fn x_file_size<T: SqliteIoMethods>(
        arg1: *mut sqlite3_file,
        pSize: *mut sqlite3_int64,
    ) -> c_int {
        let mut b = Box::<dyn SqliteIoMethods>::from_raw(arg1.cast::<T>());
        match b.file_size(pSize) {
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
    
    
    pub unsafe extern "C" fn x_lock<T: SqliteIoMethods>(
        arg1: *mut sqlite3_file,
        arg2: c_int,
    ) -> c_int {
        let mut b = Box::<dyn SqliteIoMethods>::from_raw(arg1.cast::<T>());
        match b.lock(arg2) {
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
    
    pub unsafe extern "C" fn x_unlock<T: SqliteIoMethods>(
        arg1: *mut sqlite3_file,
        arg2: c_int,
    ) -> c_int {
        let mut b = Box::<dyn SqliteIoMethods>::from_raw(arg1.cast::<T>());
        match b.unlock(arg2) {
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
    
    pub unsafe extern "C" fn x_check_reserved_lock<T: SqliteIoMethods>(
        arg1: *mut sqlite3_file,
        pResOut: *mut c_int,
    ) -> c_int {
        let mut b = Box::<dyn SqliteIoMethods>::from_raw(arg1.cast::<T>());
        match b.check_reserved_lock(pResOut) {
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
    
    pub unsafe extern "C" fn x_file_control<T: SqliteIoMethods>(
        arg1: *mut sqlite3_file,
        op: c_int,
        pArg: *mut c_void,
    ) -> c_int {
        let mut b = Box::<dyn SqliteIoMethods>::from_raw(arg1.cast::<T>());
        match b.file_control(op, pArg) {
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
    
    pub unsafe extern "C" fn x_sector_size<T: SqliteIoMethods>(arg1: *mut sqlite3_file) -> c_int {
        let mut b = Box::<dyn SqliteIoMethods>::from_raw(arg1.cast::<T>());
        match b.sector_size() {
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
    
    pub unsafe extern "C" fn x_device_characteristics<T: SqliteIoMethods>(arg1: *mut sqlite3_file) -> c_int {
        let mut b = Box::<dyn SqliteIoMethods>::from_raw(arg1.cast::<T>());
        match b.device_characteristics() {
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
    
    pub unsafe extern "C" fn x_shm_map<T: SqliteIoMethods>(
        arg1: *mut sqlite3_file,
        iPg: c_int,
        pgsz: c_int,
        arg2: c_int,
        arg3: *mut *mut c_void,
    ) -> c_int {
        let mut b = Box::<dyn SqliteIoMethods>::from_raw(arg1.cast::<T>());
        match b.shm_map(iPg, pgsz, arg2, arg3) {
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
    
    pub unsafe extern "C" fn x_shm_lock<T: SqliteIoMethods>(
        arg1: *mut sqlite3_file,
        offset: c_int,
        n: c_int,
        flags: c_int,
    ) -> c_int {
        let mut b = Box::<dyn SqliteIoMethods>::from_raw(arg1.cast::<T>());
        match b.shm_lock(offset, n, flags) {
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
    
    pub unsafe extern "C" fn x_shm_barrier<T: SqliteIoMethods>(arg1: *mut sqlite3_file) {
        let mut b = Box::<dyn SqliteIoMethods>::from_raw(arg1.cast::<T>());
        match b.shm_barrier() {
            Ok(()) => (),
            Err(e) => {
                // TODO define error handling
                // if api::result_error(context, &e.result_error_message()).is_err() {
                //     api::result_error_code(context, SQLITE_INTERNAL);
                // }
            }
        }
        Box::into_raw(b); // TODO drop in close
    }
    
    pub unsafe extern "C" fn x_shm_unmap<T: SqliteIoMethods>(
        arg1: *mut sqlite3_file,
        deleteFlag: c_int,
    ) -> c_int {
        let mut b = Box::<dyn SqliteIoMethods>::from_raw(arg1.cast::<T>());
        match b.shm_unmap(deleteFlag) {
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
    
    pub unsafe extern "C" fn x_fetch<T: SqliteIoMethods>(
        arg1: *mut sqlite3_file,
        iOfst: sqlite3_int64,
        iAmt: c_int,
        pp: *mut *mut c_void,
    ) -> c_int {
        let mut b = Box::<dyn SqliteIoMethods>::from_raw(arg1.cast::<T>());
        match b.fetch(iOfst, iAmt, pp) {
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
    
    pub unsafe extern "C" fn x_unfetch<T: SqliteIoMethods>(
        arg1: *mut sqlite3_file,
        iOfst: sqlite3_int64,
        p: *mut c_void,
    ) -> c_int {
        let mut b = Box::<dyn SqliteIoMethods>::from_raw(arg1.cast::<T>());
        match b.unfetch(iOfst, p) {
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
}


