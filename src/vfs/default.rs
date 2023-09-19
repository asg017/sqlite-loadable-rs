#![ allow(unused)]
#![ allow(non_snake_case)]

use crate::{Error, SqliteIoMethods};

use super::super::{Result, vfs::traits::SqliteVfs};

use std::{os::raw::{c_int, c_void, c_char}, ptr};

use sqlite3ext_sys::{sqlite3_file, sqlite3_int64, sqlite3_syscall_ptr, sqlite3_io_methods, sqlite3_vfs, sqlite3_vfs_find, SQLITE_OK, SQLITE_ERROR};

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
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Ok(())
            }
        }
    }

    fn delete(
        &mut self,
        z_name: *const c_char,
        sync_dir: c_int,
    ) -> Result<()> {
        unsafe {
            if let Some(xDelete) = (*self.default_vfs).xDelete {
                let result = xDelete(self.default_vfs, z_name, sync_dir);
                if result == SQLITE_OK {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Ok(())
            }
        }
    }

    fn access(
        &mut self,
        z_name: *const c_char,
        flags: c_int,
        p_res_out: *mut c_int,
    ) -> Result<()> {
        unsafe {
            if let Some(xAccess) = (*self.default_vfs).xAccess {
                let result = xAccess(self.default_vfs, z_name, flags, p_res_out);
                if result == SQLITE_OK {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
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
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Ok(())
            }
        }
    }

    fn dl_open(
        &mut self,
        z_filename: *const c_char,
    ) -> *mut c_void {
        unsafe {
            if let Some(xDlOpen) = (*self.default_vfs).xDlOpen {
                xDlOpen(self.default_vfs, z_filename)
            } else {
                ptr::null_mut()
            }
        }
    }

    fn dl_error(
        &mut self,
        n_byte: c_int,
        z_err_msg: *mut c_char,
    ) {
        unsafe {
            if let Some(xDlError) = (*self.default_vfs).xDlError {
                xDlError(self.default_vfs, n_byte, z_err_msg);
            }
        }
    }

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

    fn dl_close(
        &mut self,
        arg2: *mut c_void,
    ) {
        unsafe {
            if let Some(xDlClose) = (*self.default_vfs).xDlClose {
                xDlClose(self.default_vfs, arg2);
            }
        }
    }

    fn randomness(
        &mut self,
        n_byte: c_int,
        z_out: *mut c_char,
    ) -> c_int {
        unsafe {
            if let Some(xRandomness) = (*self.default_vfs).xRandomness {
                xRandomness(self.default_vfs, n_byte, z_out)
            } else {
                SQLITE_ERROR
            }
        }
    }

    fn sleep(
        &mut self,
        microseconds: c_int,
    ) -> c_int {
        unsafe {
            if let Some(xSleep) = (*self.default_vfs).xSleep {
                xSleep(self.default_vfs, microseconds)
            } else {
                SQLITE_ERROR
            }
        }
    }

    fn current_time(
        &mut self,
        arg2: *mut f64,
    ) -> c_int {
        unsafe {
            if let Some(xCurrentTime) = (*self.default_vfs).xCurrentTime {
                xCurrentTime(self.default_vfs, arg2)
            } else {
                SQLITE_ERROR
            }
        }
    }

    fn get_last_error(
        &mut self,
        arg2: c_int,
        arg3: *mut c_char,
    ) -> Result<()> {
        unsafe {
            if let Some(xGetLastError) = (*self.default_vfs).xGetLastError {
                let result = xGetLastError(self.default_vfs, arg2, arg3);
                if result == SQLITE_OK {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Ok(())
            }
        }
    }

    fn current_time_int64(
        &mut self,
        arg2: *mut sqlite3_int64,
    ) -> c_int {
        unsafe {
            if let Some(xCurrentTimeInt64) = (*self.default_vfs).xCurrentTimeInt64 {
                xCurrentTimeInt64(self.default_vfs, arg2)
            } else {
                SQLITE_ERROR
            }
        }
    }

    fn set_system_call(
        &mut self,
        z_name: *const c_char,
        arg2: sqlite3_syscall_ptr,
    ) -> c_int {
        unsafe {
            if let Some(xSetSystemCall) = (*self.default_vfs).xSetSystemCall {
                xSetSystemCall(self.default_vfs, z_name, arg2)
            } else {
                SQLITE_ERROR
            }
        }
    }

    fn get_system_call(
        &mut self,
        z_name: *const c_char,
    ) -> sqlite3_syscall_ptr {
        unsafe {
            if let Some(xGetSystemCall) = (*self.default_vfs).xGetSystemCall {
                xGetSystemCall(self.default_vfs, z_name)
            } else {
                None
            }
        }
    }

    fn next_system_call(
        &mut self,
        z_name: *const c_char,
    ) -> *const c_char {
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
struct DefaultFile {
    default_methods_ptr: *mut sqlite3_io_methods,
    default_file_ptr: *mut sqlite3_file,
}

impl DefaultFile {
    fn from_ptr(file_ptr: *mut sqlite3_file) -> Self {
        Self {
            default_file_ptr: file_ptr,
            default_methods_ptr: (unsafe { *file_ptr }).pMethods.cast_mut()
        }
    }
}

impl SqliteIoMethods for DefaultFile {
    fn close(&mut self) -> Result<()> {
        unsafe {
            if let Some(xClose) = ((*self.default_methods_ptr).xClose) {
                let result = xClose(self.default_file_ptr);
                if result == 1 {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Err(Error::new_message("Missing function"))
            }
        }
    }

    fn read(&mut self, buf: *mut c_void, i_amt: i32, i_ofst: i64) -> Result<()> {
        unsafe {
            if let Some(xRead) = ((*self.default_methods_ptr).xRead) {
                let result = xRead(self.default_file_ptr, buf, i_amt.try_into().unwrap(), i_ofst.try_into().unwrap());
                if result == 1 {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Err(Error::new_message("Missing function"))
            }
        }
    }

    fn write(
        &mut self,
        buf: *const c_void,
        i_amt: i32,
        i_ofst: i64,
    ) -> Result<()> {
        unsafe {
            if let Some(xWrite) = ((*self.default_methods_ptr).xWrite) {
                let result = xWrite(self.default_file_ptr, buf, i_amt.try_into().unwrap(), i_ofst.try_into().unwrap());
                if result == 1 {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Err(Error::new_message("Missing function"))
            }
        }
    }
    
    fn truncate(&mut self, size: i64) -> Result<()> {
        unsafe {
            if let Some(xTruncate) = ((*self.default_methods_ptr).xTruncate) {
                let result = xTruncate(self.default_file_ptr, size.try_into().unwrap());
                if result == 1 {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Err(Error::new_message("Missing function"))
            }
        }
    }

    fn sync(&mut self, flags: c_int) -> Result<()> {
        unsafe {
            if let Some(xSync) = ((*self.default_methods_ptr).xSync) {
                let result = xSync(self.default_file_ptr,flags);
                if result == 1 {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Err(Error::new_message("Missing function"))
            }
        }
    }
    
    fn file_size(&mut self, p_size: *mut sqlite3_int64) -> Result<()> {
        unsafe {
            if let Some(xFileSize) = ((*self.default_methods_ptr).xFileSize) {
                let result = xFileSize(self.default_file_ptr,p_size);
                if result == 1 {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Err(Error::new_message("Missing function"))
            }
        }
    }

    fn lock(&mut self, arg2: c_int) -> Result<()> {
        unsafe {
            if let Some(xLock) = ((*self.default_methods_ptr).xLock) {
                let result = xLock(self.default_file_ptr,arg2);
                if result == 1 {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Err(Error::new_message("Missing function"))
            }
        }
    }

    fn unlock(&mut self, arg2: c_int) -> Result<()> {
        unsafe {
            if let Some(xUnlock) = ((*self.default_methods_ptr).xUnlock) {
                let result = xUnlock(self.default_file_ptr,arg2);
                if result == 1 {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Err(Error::new_message("Missing function"))
            }
        }
    }
            
    fn check_reserved_lock(&mut self, p_res_out: *mut c_int) -> Result<()> {
        unsafe {
            if let Some(xCheckReservedLock) = ((*self.default_methods_ptr).xCheckReservedLock) {
                let result = xCheckReservedLock(self.default_file_ptr, p_res_out);
                if result == 1 {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Err(Error::new_message("Missing function"))
            }
        }
    }

    fn file_control(&mut self, op: c_int, p_arg: *mut c_void) -> Result<()> {
        unsafe {
            if let Some(xFileControl) = ((*self.default_methods_ptr).xFileControl) {
                let result = xFileControl(self.default_file_ptr, op, p_arg);
                if result == 1 {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Err(Error::new_message("Missing function"))
            }
        }
    }
                
    fn sector_size(&mut self) -> c_int {
        unsafe {
            if let Some(xSectorSize) = ((*self.default_methods_ptr).xSectorSize) {
                xSectorSize(self.default_file_ptr)
            } else {
                -1
            }
        }
    }

    fn device_characteristics(&mut self) -> c_int {
        unsafe {
            if let Some(xDeviceCharacteristics) = ((*self.default_methods_ptr).xDeviceCharacteristics) {
                xDeviceCharacteristics(self.default_file_ptr)
            } else {
                -1
            }
        }
    }

    fn shm_map(&mut self, i_pg: c_int, pgsz: c_int, arg2: c_int, arg3: *mut *mut c_void) -> Result<()> {
        unsafe {
            if let Some(xShmMap) = ((*self.default_methods_ptr).xShmMap) {
                let result = xShmMap(self.default_file_ptr,i_pg, pgsz, arg2, arg3);
                if result >= 0 {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Err(Error::new_message("Missing function"))
            }
        }
    }

    fn shm_lock(&mut self, offset: c_int, n: c_int, flags: c_int) -> Result<()> {
        unsafe {
            if let Some(xShmLock) = ((*self.default_methods_ptr).xShmLock) {
                let result = xShmLock(self.default_file_ptr,offset, n, flags);
                if result == 1 {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Err(Error::new_message("Missing function"))
            }
        }
    }

    fn shm_barrier(&mut self) -> Result<()> {
        unsafe {
            if let Some(xShmBarrier) = ((*self.default_methods_ptr).xShmBarrier) {
                xShmBarrier(self.default_file_ptr);
                Ok(())
            } else {
                Err(Error::new_message("Missing function"))
            }
        }
    }

    fn shm_unmap(&mut self, delete_flag: c_int) -> Result<()> {
        unsafe {
            if let Some(xShmUnmap) = ((*self.default_methods_ptr).xShmUnmap) {
                let result = xShmUnmap(self.default_file_ptr, delete_flag);
                if result == 1 {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Err(Error::new_message("Missing function"))
            }
        }
    }

    fn fetch(&mut self, i_ofst: i64, i_amt: i32, pp: *mut *mut c_void) -> Result<()> {
        unsafe {
            if let Some(xFetch) = ((*self.default_methods_ptr).xFetch) {
                let result = xFetch(self.default_file_ptr, i_ofst.try_into().unwrap(), i_amt.try_into().unwrap(), pp);
                if result == 1 {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Err(Error::new_message("Missing function"))
            }
        }
    }

    fn unfetch(&mut self, i_ofst: i64, p: *mut c_void) -> Result<()> {
        unsafe {
            if let Some(xUnfetch) = ((*self.default_methods_ptr).xUnfetch) {
                let result = xUnfetch(self.default_file_ptr,i_ofst.try_into().unwrap(), p);
                if result == 1 {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Err(Error::new_message("Missing function"))
            }
        }
    }
}
    