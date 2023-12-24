use sqlite_loadable::ext::{sqlite3ext_vfs_find, sqlite3ext_context_db_handle, sqlite3ext_file_control};
use sqlite_loadable::vfs::shim::ShimVfs;
use sqlite_loadable::vfs::vfs::create_vfs;
use sqlite_loadable::vfs::file::{FileWithAux, prepare_file_ptr};

use sqlite_loadable::{prelude::*, SqliteIoMethods, register_boxed_vfs, define_scalar_function, api, vfs::traits::SqliteVfs};

use std::ffi::{CString, CStr};
use std::io::{Write, Read, self};
use std::os::raw::{c_void, c_char};
use std::{ptr, mem};

use sqlite_loadable::ext::{sqlite3_syscall_ptr, sqlite3_file, sqlite3_vfs, sqlite3_io_methods};
use sqlite3ext_sys::{
    SQLITE_CANTOPEN, SQLITE_OPEN_MAIN_DB, SQLITE_IOCAP_ATOMIC, SQLITE_IOCAP_POWERSAFE_OVERWRITE, SQLITE_IOCAP_UNDELETABLE_WHEN_OPEN,
    SQLITE_IOCAP_SAFE_APPEND, SQLITE_IOCAP_SEQUENTIAL, SQLITE_LOCK_EXCLUSIVE, SQLITE_LOCK_SHARED, SQLITE_OK};

use std::io::{Error, Result, ErrorKind};

/// Inspired by https://www.sqlite.org/src/file/ext/misc/memvfs.c
/// See https://www.sqlite.org/debugging.html for debugging methods
struct MemVfs {
    default_vfs: Option<ShimVfs>,
    name: CString,
}

const EXTENSION_NAME: &str = "memvfs";

fn write_file_to_vec_u8(path: &str, dest: &mut Vec<u8>) -> Result<()> {
    let metadata = std::fs::metadata(path)?;
    let file_size = metadata.len() as usize;

    let mut file = std::fs::File::open(path)?;

    file.read_to_end(dest)?;
    
    Ok(())
}

impl SqliteVfs for MemVfs {
    fn open(&mut self, z_name: *const c_char, p_file: *mut sqlite3_file, flags: i32, p_res_out: *mut i32) -> Result<()> {
        let mut mem_file = MemFile {
            file_contents: vec![],
        };

        unsafe {
            let file_name_cstr = CStr::from_ptr(z_name);
            let file_name = file_name_cstr.to_str()
                .map_err(|_| Error::new(ErrorKind::Other, "conversion to string failed"))?;
            write_file_to_vec_u8(file_name, &mut mem_file.file_contents);
            prepare_file_ptr(p_file, mem_file);
        }
    
        Ok(())
    }

    fn delete(&mut self, z_name: *const c_char, sync_dir: i32) -> Result<()> {
        Ok(())
    }

    fn access(&mut self, z_name: *const c_char, flags: i32, p_res_out: *mut i32) -> Result<()> {
        unsafe {
            *p_res_out = 0;
        }
        Ok(())
    }

    fn full_pathname(&mut self, z_name: *const c_char, n_out: i32, z_out: *mut c_char) -> Result<()> {
        unsafe {
            // don't rely on type conversion of n_out to determine the end line char
            let name = CString::from_raw(z_name.cast_mut());
            let src_ptr = name.as_ptr();
            let dst_ptr = z_out;
            let len = name.as_bytes().len() + 1;
            ptr::copy_nonoverlapping(src_ptr, dst_ptr.cast(), len);
            name.into_raw();
        }

        Ok(())
    }

    /// From here onwards, all calls are redirected to the default vfs
    // fn dl_open(&mut self, z_filename: *const c_char) -> *mut c_void {
    //     self.default_vfs.dl_open(z_filename)
    // }

    // fn dl_error(&mut self, n_byte: i32, z_err_msg: *mut c_char) {
    //     self.default_vfs.dl_error(n_byte, z_err_msg)
    // }

    // fn dl_sym(&mut self, arg2: *mut c_void, z_symbol: *const c_char)
    //     -> Option<unsafe extern "C" fn(arg1: *mut sqlite3_vfs, arg2: *mut c_void, z_symbol: *const c_char)> {
    //     self.default_vfs.dl_sym(arg2, z_symbol)
    // }

    // fn dl_close(&mut self, arg2: *mut c_void) {
    //     self.default_vfs.dl_close(arg2)
    // }

    fn randomness(&mut self, n_byte: i32, z_out: *mut c_char) -> i32 {
        if let Some(vfs) = &mut self.default_vfs {
            return vfs.randomness(n_byte, z_out);
        }
        0
    }

    fn sleep(&mut self, microseconds: i32) -> i32 {
        if let Some(vfs) = &mut self.default_vfs {
            return vfs.sleep(microseconds);
        }
        0
    }

    fn current_time(&mut self, arg2: *mut f64) -> i32 {
        if let Some(vfs) = &mut self.default_vfs {
            return vfs.current_time(arg2);
        }
        0
    }

    fn get_last_error(&mut self, arg2: i32, arg3: *mut c_char) -> Result<()> {
        if let Some(vfs) = &mut self.default_vfs {
            vfs.get_last_error(arg2, arg3);
        }
        Ok(())
    }

    fn current_time_int64(&mut self, arg2: *mut i64) -> i32 {
        if let Some(vfs) = &mut self.default_vfs {
            return vfs.current_time_int64(arg2);
        }
        0
    }

    // fn set_system_call(&mut self, z_name: *const c_char, arg2: sqlite3_syscall_ptr) -> i32 {
    //     self.default_vfs.set_system_call(z_name, arg2)
    // }

    // fn get_system_call(&mut self, z_name: *const c_char) -> sqlite3_syscall_ptr {
    //     self.default_vfs.get_system_call(z_name)
    // }

    // fn next_system_call(&mut self, z_name: *const c_char) -> *const c_char {
    //     self.default_vfs.next_system_call(z_name)
    // }
}

struct MemFile {
    file_contents: Vec<u8>,
}

impl SqliteIoMethods for MemFile {
    fn close(&mut self) -> Result<()> {
        Ok(())
    }

    fn read(&mut self, buf: *mut c_void, s: i32, ofst: i64) -> Result<()> {
        let size: usize = s.try_into().unwrap();
        let offset = ofst.try_into().unwrap();
        let source = &mut self.file_contents;
        if source.len() < size {
            let new_len = offset + size;
            let prev_len = source.len();
            source.resize(new_len, 0);
            source.extend(vec![0; new_len - prev_len]);
        }

        let src_ptr = source[offset..(offset + size-1)].as_ptr();
        unsafe { ptr::copy_nonoverlapping(src_ptr, buf.cast(), size) }
    
        Ok(())
    }

    fn write(&mut self, buf: *const c_void, s: i32, ofst: i64) -> Result<()> {
        let size = s.try_into().unwrap();
        let offset = ofst.try_into().unwrap();
        let new_length = size + offset;
        if new_length > self.file_contents.len() {
            self.file_contents.resize(new_length, 0);
        }

        let dest = &mut self.file_contents;

        let src_slice = unsafe { std::slice::from_raw_parts(buf as *const u8, size) };

        dest[offset..offset + src_slice.len()].copy_from_slice(src_slice);

        Ok(())
    }

    fn truncate(&mut self, size: i64) -> Result<()> {
        self.file_contents.resize(size.try_into().unwrap(), 0);

        Ok(())
    }

    fn sync(&mut self, flags: i32) -> Result<()> {
        Ok(())
    }

    fn file_size(&mut self, p_size: *mut i64) -> Result<()> {
        unsafe { *p_size = self.file_contents.len().try_into().unwrap(); }
        Ok(())
    }

    fn lock(&mut self, arg2: i32) -> Result<i32> {
        Ok(0) // or SQLITE_LOCK_BUSY
    }

    fn unlock(&mut self, arg2: i32) -> Result<i32> {
        Ok(0)
    }

    fn check_reserved_lock(&mut self, p_res_out: *mut i32) -> Result<()> {
        unsafe{ *p_res_out = 0; }
        Ok(())
    }

    fn file_control(&mut self, op: i32, p_arg: *mut c_void) -> Result<()> {
        Ok(())
    }

    fn sector_size(&mut self) -> Result<i32> {
        Ok(1024)
    }

    fn device_characteristics(&mut self) -> Result<i32> {
        let settings = SQLITE_IOCAP_ATOMIC | 
        SQLITE_IOCAP_POWERSAFE_OVERWRITE |
        SQLITE_IOCAP_SAFE_APPEND |
        // SQLITE_IOCAP_UNDELETABLE_WHEN_OPEN |
        SQLITE_IOCAP_SEQUENTIAL;
        Ok(settings)
    }

    fn shm_map(&mut self, i_pg: i32, pgsz: i32, arg2: i32, arg3: *mut *mut c_void) -> Result<()> {
        // SQLITE_IOERR_SHMMAP
        Err(Error::new(ErrorKind::Other, "Unsupported"))
    }

    fn shm_lock(&mut self, offset: i32, n: i32, flags: i32) -> Result<()> {
        // SQLITE_IOERR_SHMLOCK
        Err(Error::new(ErrorKind::Other, "Unsupported"))
    }

    fn shm_barrier(&mut self) -> Result<()> {
        Ok(())
    }

    fn shm_unmap(&mut self, delete_flag: i32) -> Result<()> {
        Ok(())
    }

    fn fetch(&mut self, ofst: i64, size: i32, pp: *mut *mut c_void) -> Result<()> {
        let memory_location = self.file_contents.as_mut_ptr();
        unsafe { *pp = memory_location.add(ofst.try_into().unwrap()).cast(); }
        Ok(())
    }

    fn unfetch(&mut self, i_ofst: i64, p: *mut c_void) -> Result<()> {
        Ok(())
    }
}

fn print_uri(context: *mut sqlite3_context, _: &[*mut sqlite3_value]) -> sqlite_loadable::Result<()> {
    let text_output = format!("file:___mem___?vfs={}", EXTENSION_NAME);

    api::result_text(context, text_output);

    Ok(())
}

#[sqlite_entrypoint]
pub fn sqlite3_memvfs_init(db: *mut sqlite3) -> sqlite_loadable::Result<()> {
    let name = CString::new(EXTENSION_NAME).expect("should be a valid utf-8 string");
    let mem_vfs = MemVfs {
        default_vfs: unsafe {
            // pass thru
            // Some(ShimVfs::from_ptr(sqlite3ext_vfs_find(ptr::null())))
            None
        },
        name
    };
    let name_ptr = mem_vfs.name.as_ptr();

    let vfs: sqlite3_vfs = create_vfs(mem_vfs, name_ptr, 1024, std::mem::size_of::<FileWithAux<MemFile>>() as i32);

    register_boxed_vfs(vfs, true)?;

    let flags = FunctionFlags::UTF8 | FunctionFlags::DETERMINISTIC;
    define_scalar_function(db, "mem_vfs_uri", 0, print_uri, flags)?;

    Ok(())
}
