#![ allow(unused)]
#![ allow(non_snake_case)]

use crate::{Error, SqliteIoMethods};

use super::super::{Result, vfs::traits::SqliteVfs};

use std::{os::raw::{c_int, c_void, c_char}, ptr};

use sqlite3ext_sys::{sqlite3_file, sqlite3_int64, sqlite3_syscall_ptr, sqlite3_io_methods, SQLITE_OK, sqlite3_vfs, sqlite3_vfs_find};


pub struct DefaultVfs {
    default_vfs_ptr: *mut sqlite3_vfs,
}

impl DefaultVfs {
    pub fn new() -> Self {
        let vfs = unsafe { sqlite3_vfs_find(ptr::null::<i8>()) };
        Self {
            default_vfs_ptr: vfs,
        }
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
        // This won't be used probably
        /*
        unsafe {
            if let Some(xOpen) = ((*self.default_vfs_ptr).xOpen) {
                let result = xOpen(
                    self.default_vfs_ptr,
                    z_name,
                    p_file,
                    flags,
                    p_out_flags,
                );

                if result == 0 {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Err(Error::new_message("Missing function"))
            }
        }
        */
        Ok(())
    }

    fn delete(
        &mut self,
        z_name: *const c_char,
        sync_dir: c_int,
    ) -> Result<()> {
        unsafe {
            if let Some(xDelete) = ((*self.default_vfs_ptr).xDelete) {
                let result = xDelete(
                    self.default_vfs_ptr,
                    z_name,
                    sync_dir,
                );

                if result == 0 {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Err(Error::new_message("Missing function"))
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
            if let Some(xAccess) = ((*self.default_vfs_ptr).xAccess) {
                let result = xAccess(
                    self.default_vfs_ptr,
                    z_name,
                    flags,
                    p_res_out,
                );

                if result == 0 {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Err(Error::new_message("Missing function"))
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
            if let Some(xFullPathname) = ((*self.default_vfs_ptr).xFullPathname) {
                let result = xFullPathname(
                    self.default_vfs_ptr,
                    z_name,
                    n_out,
                    z_out,
                );

                if result == 0 {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Err(Error::new_message("Missing function"))
            }
        }
    }

    fn dl_open(
        &mut self,
        z_filename: *const c_char,
    ) -> *mut c_void {
        unsafe {
            if let Some(xDlOpen) = ((*self.default_vfs_ptr).xDlOpen) {
                xDlOpen(self.default_vfs_ptr, z_filename)
            } else {
                std::ptr::null_mut() // Return null pointer or handle missing function appropriately
            }
        }
    }

    fn dl_error(
        &mut self,
        n_byte: c_int,
        z_err_msg: *mut c_char,
    ) {
        unsafe {
            if let Some(xDlError) = ((*self.default_vfs_ptr).xDlError) {
                xDlError(self.default_vfs_ptr, n_byte, z_err_msg);
            }
        }
    }

    fn dl_sym(
        &mut self,
        arg2: *mut c_void,
        z_symbol: *const c_char,
    ) -> Option<
        unsafe extern "C" fn(
            arg1: *mut sqlite3_vfs,
            arg2: *mut c_void,
            z_symbol: *const c_char,
        ),
    > {
        unsafe {
            if let Some(xDlSym) = ((*self.default_vfs_ptr).xDlSym) {
                xDlSym(self.default_vfs_ptr, arg2, z_symbol)
            } else {
                None // TODO Handle missing function appropriately
            }
        }
    }

    fn dl_close(&mut self, arg2: *mut c_void) {
        unsafe {
            if let Some(xDlClose) = ((*self.default_vfs_ptr).xDlClose) {
                xDlClose(self.default_vfs_ptr, arg2);
            } else {
                // TODO Handle missing function appropriately (e.g., log an error)
            }
        }
    }

    fn randomness(
        &mut self,
        n_byte: c_int,
        z_out: *mut c_char,
    ) -> Result<()> {
        unsafe {
            if let Some(xRandomness) = ((*self.default_vfs_ptr).xRandomness) {
                let result = xRandomness(
                    self.default_vfs_ptr,
                    n_byte,
                    z_out,
                );

                if result == 0 {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Err(Error::new_message("Missing function"))
            }
        }
    }

    fn sleep(
        &mut self,
        microseconds: c_int,
    ) -> Result<()> {
        unsafe {
            if let Some(xSleep) = ((*self.default_vfs_ptr).xSleep) {
                let result = xSleep(
                    self.default_vfs_ptr,
                    microseconds,
                );

                if result == 0 {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Err(Error::new_message("Missing function"))
            }
        }
    }

    fn current_time(&mut self, arg2: *mut f64) -> Result<()> {
        unsafe {
            if let Some(xCurrentTime) = ((*self.default_vfs_ptr).xCurrentTime) {
                let result = xCurrentTime(self.default_vfs_ptr, arg2);

                if result == 0 {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Err(Error::new_message("Missing function"))
            }
        }
    }

    fn get_last_error(
        &mut self,
        arg2: c_int,
        arg3: *mut c_char,
    ) -> Result<()> {
        unsafe {
            if let Some(xGetLastError) = ((*self.default_vfs_ptr).xGetLastError) {
                let result = xGetLastError(
                    self.default_vfs_ptr,
                    arg2,
                    arg3,
                );

                if result == 0 {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Err(Error::new_message("Missing function"))
            }
        }
    }

    fn current_time_int64(
        &mut self,
        arg2: *mut sqlite3_int64,
    ) -> c_int {
        unsafe {
            if let Some(xCurrentTimeInt64) = ((*self.default_vfs_ptr).xCurrentTimeInt64) {
                xCurrentTimeInt64(self.default_vfs_ptr, arg2)
            } else {
                // Err(Error::new_message("Missing function"))
                -1
            }
        }
    }

    fn set_system_call(
        &mut self,
        z_name: *const c_char,
        arg2: sqlite3_syscall_ptr,
    ) -> c_int {
        unsafe {
            if let Some(xSetSystemCall) = ((*self.default_vfs_ptr).xSetSystemCall) {
                xSetSystemCall(
                    self.default_vfs_ptr,
                    z_name,
                    arg2,
                )

            } else {
                // Err(Error::new_message("Missing function"))
                -1
            }
        }
    }

    fn get_system_call(
        &mut self,
        z_name: *const c_char,
    ) -> sqlite3_syscall_ptr {
        unsafe {
            if let Some(xGetSystemCall) = ((*self.default_vfs_ptr).xGetSystemCall) {
                xGetSystemCall(self.default_vfs_ptr, z_name)
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
            if let Some(xNextSystemCall) = ((*self.default_vfs_ptr).xNextSystemCall) {
                xNextSystemCall(self.default_vfs_ptr, z_name)
            } else {
                std::ptr::null()
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
    /// See sqlite3_vfs::x_open sqlite3_file parameter
    fn from_file_ptr(file_ptr: *mut sqlite3_file) -> Self {
        Self {
            default_file_ptr: file_ptr,
            default_methods_ptr: (unsafe { *file_ptr }).pMethods.cast_mut()
        }
    }
}

impl SqliteIoMethods for DefaultFile {
    fn close(&mut self) -> Result<()> {
        // This won't be used probably
        /*
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
        */
        Ok(())
    }

    fn read(&mut self, buf: *mut c_void, i_amt: usize, i_ofst: usize) -> Result<()> {
        // This won't be used probably
        /*
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
        */
        Ok(())
    }

    fn write(
        &mut self,
        buf: *const c_void,
        i_amt: usize,
        i_ofst: usize,
    ) -> Result<()> {
        /* 
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
        */
        Ok(())
    }
    
    fn truncate(&mut self, size: usize) -> Result<()> {
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

    fn fetch(&mut self, i_ofst: usize, i_amt: usize, pp: *mut *mut c_void) -> Result<()> {
        /*
        unsafe {
            if let Some(xFetch) = ((*self.default_methods_ptr).xFetch) {
                let result = xFetch(self.default_file_ptr,i_ofst, i_amt, pp);
                if result == 1 {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Err(Error::new_message("Missing function"))
            }
        }
        */
        Ok(())
    }

    fn unfetch(&mut self, i_ofst: usize, p: *mut c_void) -> Result<()> {
        /*
        unsafe {
            if let Some(xUnfetch) = ((*self.default_methods_ptr).xUnfetch) {
                let result = xUnfetch(self.default_file_ptr,i_ofst, p);
                if result == 1 {
                    Ok(())
                } else {
                    Err(Error::new(crate::ErrorKind::DefineVfs(result)))
                }
            } else {
                Err(Error::new_message("Missing function"))
            }
        }
        */
        Ok(())
    }
}
    