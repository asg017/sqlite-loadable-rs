#![allow(unused)]
#![allow(non_snake_case)]

use crate::SqliteIoMethods;

use std::io::{Error, ErrorKind, Result};

use super::super::vfs::traits::SqliteVfs;

use std::{
    os::raw::{c_char, c_int, c_void},
    ptr,
};

use sqlite3ext_sys::{
    sqlite3_file, sqlite3_int64, sqlite3_io_methods, sqlite3_syscall_ptr, sqlite3_vfs,
    sqlite3_vfs_find, SQLITE_ERROR, SQLITE_LOCK_NONE, SQLITE_OK,
};

pub struct DefaultVfs {
    default_vfs: *mut sqlite3_vfs,
}

impl DefaultVfs {
    pub fn from_ptr(vfs: *mut sqlite3_vfs) -> Self {
        DefaultVfs { default_vfs: vfs }
    }
}

impl SqliteVfs for DefaultVfs {
    fn open(
        &mut self,
        z_name: *const c_char,
        p_file: *mut sqlite3_file,
        flags: c_int,
        p_out_flags: *mut c_int,
    ) -> Result<()> {
        unsafe {
            if let Some(xOpen) = (*self.default_vfs).xOpen {
                let result = xOpen(self.default_vfs, z_name, p_file, flags, p_out_flags);
                if result == SQLITE_OK {
                    Ok(())
                } else {
                    Err(Error::new(
                        ErrorKind::Other,
                        "An error occurred while opening a file",
                    ))
                }
            } else {
                Ok(())
            }
        }
    }

    fn delete(&mut self, z_name: *const c_char, sync_dir: c_int) -> Result<()> {
        unsafe {
            if let Some(xDelete) = (*self.default_vfs).xDelete {
                let result = xDelete(self.default_vfs, z_name, sync_dir);
                if result == SQLITE_OK {
                    Ok(())
                } else {
                    Err(Error::new(
                        ErrorKind::Other,
                        "An error occurred while removing a file",
                    ))
                }
            } else {
                Ok(())
            }
        }
    }

    fn access(&mut self, z_name: *const c_char, flags: c_int, p_res_out: *mut c_int) -> Result<()> {
        unsafe {
            if let Some(xAccess) = (*self.default_vfs).xAccess {
                let result = xAccess(self.default_vfs, z_name, flags, p_res_out);
                if result == SQLITE_OK {
                    Ok(())
                } else {
                    Err(Error::new(
                        ErrorKind::Other,
                        "An error occurred while determining the permissions of a file",
                    ))
                }
            } else {
                Ok(())
            }
        }
    }

    fn full_pathname(
        &mut self,
        z_name: *const c_char,
        n_out: c_int,
        z_out: *mut c_char,
    ) -> Result<()> {
        unsafe {
            if let Some(xFullPathname) = (*self.default_vfs).xFullPathname {
                let result = xFullPathname(self.default_vfs, z_name, n_out, z_out);
                if result == SQLITE_OK {
                    Ok(())
                } else {
                    Err(Error::new(
                        ErrorKind::Other,
                        "An error occurred while determining the full pathname of a file",
                    ))
                }
            } else {
                Ok(())
            }
        }
    }

    #[cfg(feature = "vfs_loadext")]
    fn dl_open(&mut self, z_filename: *const c_char) -> *mut c_void {
        unsafe {
            if let Some(xDlOpen) = (*self.default_vfs).xDlOpen {
                xDlOpen(self.default_vfs, z_filename)
            } else {
                ptr::null_mut()
            }
        }
    }

    #[cfg(feature = "vfs_loadext")]
    fn dl_error(&mut self, n_byte: c_int, z_err_msg: *mut c_char) {
        unsafe {
            if let Some(xDlError) = (*self.default_vfs).xDlError {
                xDlError(self.default_vfs, n_byte, z_err_msg);
            }
        }
    }

    #[cfg(feature = "vfs_loadext")]
    fn dl_sym(
        &mut self,
        arg2: *mut c_void,
        z_symbol: *const c_char,
    ) -> Option<unsafe extern "C" fn(*mut sqlite3_vfs, *mut c_void, *const c_char)> {
        unsafe {
            if let Some(func) = (*self.default_vfs).xDlSym {
                func(self.default_vfs, arg2, z_symbol);
            }
            None
        }
    }

    #[cfg(feature = "vfs_loadext")]
    fn dl_close(&mut self, arg2: *mut c_void) {
        unsafe {
            if let Some(xDlClose) = (*self.default_vfs).xDlClose {
                xDlClose(self.default_vfs, arg2);
            }
        }
    }

    fn randomness(&mut self, n_byte: c_int, z_out: *mut c_char) -> c_int {
        unsafe {
            if let Some(xRandomness) = (*self.default_vfs).xRandomness {
                xRandomness(self.default_vfs, n_byte, z_out)
            } else {
                SQLITE_ERROR
            }
        }
    }

    fn sleep(&mut self, microseconds: c_int) -> c_int {
        unsafe {
            if let Some(xSleep) = (*self.default_vfs).xSleep {
                xSleep(self.default_vfs, microseconds)
            } else {
                SQLITE_ERROR
            }
        }
    }

    fn current_time(&mut self, arg2: *mut f64) -> c_int {
        unsafe {
            if let Some(xCurrentTime) = (*self.default_vfs).xCurrentTime {
                xCurrentTime(self.default_vfs, arg2)
            } else {
                SQLITE_ERROR
            }
        }
    }

    fn get_last_error(&mut self, arg2: c_int, arg3: *mut c_char) -> Result<()> {
        unsafe {
            if let Some(xGetLastError) = (*self.default_vfs).xGetLastError {
                let result = xGetLastError(self.default_vfs, arg2, arg3);
                if result == SQLITE_OK {
                    Ok(())
                } else {
                    Err(Error::new(
                        ErrorKind::Other,
                        "An error occurred while determining the last internal error",
                    ))
                }
            } else {
                Ok(())
            }
        }
    }

    fn current_time_int64(&mut self, arg2: *mut sqlite3_int64) -> c_int {
        unsafe {
            if let Some(xCurrentTimeInt64) = (*self.default_vfs).xCurrentTimeInt64 {
                xCurrentTimeInt64(self.default_vfs, arg2)
            } else {
                SQLITE_ERROR
            }
        }
    }

    #[cfg(feature = "vfs_syscall")]
    fn set_system_call(&mut self, z_name: *const c_char, arg2: sqlite3_syscall_ptr) -> c_int {
        unsafe {
            if let Some(xSetSystemCall) = (*self.default_vfs).xSetSystemCall {
                xSetSystemCall(self.default_vfs, z_name, arg2)
            } else {
                SQLITE_ERROR
            }
        }
    }

    #[cfg(feature = "vfs_syscall")]
    fn get_system_call(&mut self, z_name: *const c_char) -> sqlite3_syscall_ptr {
        unsafe {
            if let Some(xGetSystemCall) = (*self.default_vfs).xGetSystemCall {
                xGetSystemCall(self.default_vfs, z_name)
            } else {
                None
            }
        }
    }

    #[cfg(feature = "vfs_syscall")]
    fn next_system_call(&mut self, z_name: *const c_char) -> *const c_char {
        unsafe {
            if let Some(xNextSystemCall) = (*self.default_vfs).xNextSystemCall {
                xNextSystemCall(self.default_vfs, z_name)
            } else {
                ptr::null()
            }
        }
    }
}

/// See ORIGFILE https://www.sqlite.org/src/file?name=ext/misc/cksumvfs.c
pub struct DefaultFile {
    methods_ptr: *mut sqlite3_io_methods,
    file_ptr: *mut sqlite3_file,
}

impl DefaultFile {
    pub fn from_ptr(file_ptr: *mut sqlite3_file) -> Self {
        if file_ptr.is_null() {
            return Self {
                file_ptr,
                methods_ptr: ptr::null_mut(),
            };
        }
        Self {
            file_ptr,
            methods_ptr: (unsafe { *file_ptr }).pMethods.cast_mut(),
        }
    }
}

impl SqliteIoMethods for DefaultFile {
    fn close(&mut self, file: *mut sqlite3_file) -> Result<()> {
        unsafe {
            if let Some(xClose) = ((*self.methods_ptr).xClose) {
                let result = xClose(self.file_ptr);
                if result == SQLITE_OK {
                    Ok(())
                } else {
                    Err(Error::new(
                        ErrorKind::Other,
                        "An error occurred while attempting to close a file",
                    ))
                }
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    "An undefined function was called",
                ))
            }
        }
    }

    fn read(&mut self, file: *mut sqlite3_file, buf: *mut c_void, s: i32, ofst: i64) -> Result<()> {
        unsafe {
            if let Some(xRead) = ((*self.methods_ptr).xRead) {
                let result = xRead(self.file_ptr, buf, s, ofst);
                if result == SQLITE_OK {
                    Ok(())
                } else {
                    Err(Error::new(
                        ErrorKind::Other,
                        "An error occurred while reading from a file",
                    ))
                }
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    "An undefined function was called",
                ))
            }
        }
    }

    fn write(
        &mut self,
        file: *mut sqlite3_file,
        buf: *const c_void,
        i_amt: i32,
        i_ofst: i64,
    ) -> Result<()> {
        unsafe {
            if let Some(xWrite) = ((*self.methods_ptr).xWrite) {
                let result = xWrite(self.file_ptr, buf, i_amt, i_ofst);
                if result == SQLITE_OK {
                    Ok(())
                } else {
                    Err(Error::new(
                        ErrorKind::Other,
                        "An error occurred while writing to a file",
                    ))
                }
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    "An undefined function was called",
                ))
            }
        }
    }

    fn truncate(&mut self, file: *mut sqlite3_file, size: i64) -> Result<()> {
        unsafe {
            if let Some(xTruncate) = ((*self.methods_ptr).xTruncate) {
                let result = xTruncate(self.file_ptr, size);
                if result == SQLITE_OK {
                    Ok(())
                } else {
                    Err(Error::new(
                        ErrorKind::Other,
                        "An error occurred while truncating a file",
                    ))
                }
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    "An undefined function was called",
                ))
            }
        }
    }

    fn sync(&mut self, file: *mut sqlite3_file, flags: c_int) -> Result<()> {
        unsafe {
            if let Some(xSync) = ((*self.methods_ptr).xSync) {
                let result = xSync(self.file_ptr, flags);
                if result == SQLITE_OK {
                    Ok(())
                } else {
                    Err(Error::new(
                        ErrorKind::Other,
                        "An error occurred while updating the metadata of a file",
                    ))
                }
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    "An undefined function was called",
                ))
            }
        }
    }

    fn file_size(&mut self, file: *mut sqlite3_file, p_size: *mut sqlite3_int64) -> Result<()> {
        unsafe {
            if let Some(xFileSize) = ((*self.methods_ptr).xFileSize) {
                let result = xFileSize(self.file_ptr, p_size);
                if result == SQLITE_OK {
                    Ok(())
                } else {
                    Err(Error::new(
                        ErrorKind::Other,
                        "An error occurred while determining the size a file",
                    ))
                }
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    "An undefined function was called",
                ))
            }
        }
    }

    fn lock(&mut self, file: *mut sqlite3_file, arg2: c_int) -> Result<i32> {
        unsafe {
            if let Some(xLock) = ((*self.methods_ptr).xLock) {
                Ok(xLock(self.file_ptr, arg2))
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    "An undefined function was called",
                ))
            }
        }
    }

    fn unlock(&mut self, file: *mut sqlite3_file, arg2: c_int) -> Result<i32> {
        unsafe {
            if let Some(xUnlock) = ((*self.methods_ptr).xUnlock) {
                Ok(xUnlock(self.file_ptr, arg2))
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    "An undefined function was called",
                ))
            }
        }
    }

    fn check_reserved_lock(
        &mut self,
        file: *mut sqlite3_file,
        p_res_out: *mut c_int,
    ) -> Result<()> {
        unsafe {
            if let Some(xCheckReservedLock) = ((*self.methods_ptr).xCheckReservedLock) {
                xCheckReservedLock(self.file_ptr, p_res_out);
                Ok(())
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    "An undefined function was called",
                ))
            }
        }
    }

    fn file_control(
        &mut self,
        file: *mut sqlite3_file,
        op: c_int,
        p_arg: *mut c_void,
    ) -> Result<()> {
        unsafe {
            if let Some(xFileControl) = ((*self.methods_ptr).xFileControl) {
                let result = xFileControl(self.file_ptr, op, p_arg);
                if result == SQLITE_OK {
                    Ok(())
                } else {
                    Err(Error::new(
                        ErrorKind::Other,
                        "An error occurred while setting file parameters",
                    ))
                }
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    "An undefined function was called",
                ))
            }
        }
    }

    fn sector_size(&mut self, file: *mut sqlite3_file) -> Result<c_int> {
        unsafe {
            if let Some(xSectorSize) = ((*self.methods_ptr).xSectorSize) {
                Ok(xSectorSize(self.file_ptr))
            } else {
                Err(Error::new(ErrorKind::Other, "Missing sector size"))
            }
        }
    }

    fn device_characteristics(&mut self, file: *mut sqlite3_file) -> Result<c_int> {
        unsafe {
            if let Some(xDeviceCharacteristics) = ((*self.methods_ptr).xDeviceCharacteristics) {
                Ok(xDeviceCharacteristics(self.file_ptr))
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    "Missing device characteristics",
                ))
            }
        }
    }

    fn shm_map(
        &mut self,
        file: *mut sqlite3_file,
        i_pg: c_int,
        pgsz: c_int,
        arg2: c_int,
        arg3: *mut *mut c_void,
    ) -> Result<()> {
        unsafe {
            if let Some(xShmMap) = ((*self.methods_ptr).xShmMap) {
                let result = xShmMap(self.file_ptr, i_pg, pgsz, arg2, arg3);
                if result == SQLITE_OK {
                    Ok(())
                } else {
                    Err(Error::new(
                        ErrorKind::Other,
                        "An error occurred while using mmap",
                    ))
                }
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    "An undefined function was called",
                ))
            }
        }
    }

    fn shm_lock(
        &mut self,
        file: *mut sqlite3_file,
        offset: c_int,
        n: c_int,
        flags: c_int,
    ) -> Result<()> {
        unsafe {
            if let Some(xShmLock) = ((*self.methods_ptr).xShmLock) {
                let result = xShmLock(self.file_ptr, offset, n, flags);
                if result == SQLITE_OK {
                    Ok(())
                } else {
                    Err(Error::new(
                        ErrorKind::Other,
                        "An error occurred while applying a lock to mmap",
                    ))
                }
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    "An undefined function was called",
                ))
            }
        }
    }

    fn shm_barrier(&mut self, file: *mut sqlite3_file) -> Result<()> {
        unsafe {
            if let Some(xShmBarrier) = ((*self.methods_ptr).xShmBarrier) {
                xShmBarrier(self.file_ptr);
                Ok(())
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    "An undefined function was called",
                ))
            }
        }
    }

    fn shm_unmap(&mut self, file: *mut sqlite3_file, delete_flag: c_int) -> Result<()> {
        unsafe {
            if let Some(xShmUnmap) = ((*self.methods_ptr).xShmUnmap) {
                let result = xShmUnmap(self.file_ptr, delete_flag);
                if result == SQLITE_OK {
                    Ok(())
                } else {
                    Err(Error::new(
                        ErrorKind::Other,
                        "An error occurred while unmapping",
                    ))
                }
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    "An undefined function was called",
                ))
            }
        }
    }

    fn fetch(
        &mut self,
        file: *mut sqlite3_file,
        i_ofst: i64,
        i_amt: i32,
        pp: *mut *mut c_void,
    ) -> Result<()> {
        unsafe {
            if let Some(xFetch) = ((*self.methods_ptr).xFetch) {
                let result = xFetch(self.file_ptr, i_ofst, i_amt, pp);
                if result == SQLITE_OK {
                    Ok(())
                } else {
                    Err(Error::new(
                        ErrorKind::Other,
                        "An error occurred while fetching",
                    ))
                }
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    "An undefined function was called",
                ))
            }
        }
    }

    fn unfetch(&mut self, file: *mut sqlite3_file, i_ofst: i64, p: *mut c_void) -> Result<()> {
        unsafe {
            if let Some(xUnfetch) = ((*self.methods_ptr).xUnfetch) {
                let result = xUnfetch(self.file_ptr, i_ofst, p);
                if result == SQLITE_OK {
                    Ok(())
                } else {
                    Err(Error::new(
                        ErrorKind::Other,
                        "An error occurred while unfetching",
                    ))
                }
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    "An undefined function was called",
                ))
            }
        }
    }
}
