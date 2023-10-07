use sqlite_loadable::ext::{sqlite3ext_vfs_find, sqlite3ext_context_db_handle, sqlite3ext_file_control};
use sqlite_loadable::vfs::default::DefaultVfs;
use sqlite_loadable::vfs::vfs::create_vfs;

use sqlite_loadable::{prelude::*, SqliteIoMethods, create_file_pointer, register_vfs, Error, ErrorKind, define_scalar_function, api, Result, vfs::traits::SqliteVfs};
use url::Url;

use std::ffi::{CString, CStr};
use std::fs::{File, self};
use std::io::{Write, Read, self};
use std::os::raw::{c_void, c_char};
use std::{ptr, mem};

use sqlite3ext_sys::{sqlite3_syscall_ptr, sqlite3_file, sqlite3_vfs, sqlite3_io_methods};
use sqlite3ext_sys::{SQLITE_IOERR_SHMMAP, SQLITE_IOERR_SHMLOCK};
use sqlite3ext_sys::{SQLITE_CANTOPEN, SQLITE_OPEN_MAIN_DB, SQLITE_IOERR_DELETE};
use sqlite3ext_sys::{SQLITE_IOCAP_ATOMIC, SQLITE_IOCAP_POWERSAFE_OVERWRITE,
    SQLITE_IOCAP_SAFE_APPEND, SQLITE_IOCAP_SEQUENTIAL};

/// There is some duplication between rusqlite / sqlite3ext / libsqlite3
/// 
/// The following dependency has to be copied by users to use this vfs implementation:
/// sqlite3ext-sys = {version="0.0.1", path="./sqlite3ext-sys"}
// TODO This lib should be released as the new 0.0.2 version, with the previously missing
// .default_macro_constant_type(bindgen::MacroTypeVariation::Signed) parameter

/// Inspired by https://www.sqlite.org/src/file/ext/misc/memvfs.c
/// See https://www.sqlite.org/debugging.html for debugging methods
struct MemVfs {
    default_vfs: DefaultVfs,
    name: CString,
}

const EXTENSION_NAME: &str = "memvfs";

fn write_file_to_vec_u8(path: &str, dest: &mut Vec<u8>) -> Result<()> {
    let metadata = fs::metadata(path).map_err(|_| Error::new_message("can't determine file size"))?;
    let file_size = metadata.len() as usize;

    let mut file = File::open(path).map_err(|_| Error::new_message("can't open file"))?;

    file.read_to_end(dest).map_err(|_| Error::new_message("can't read to the end"))?;
    
    Ok(())
}

impl SqliteVfs for MemVfs {
    fn open(&mut self, z_name: *const c_char, p_file: *mut sqlite3_file, flags: i32, p_res_out: *mut i32) -> Result<()> {
        let mut mem_file = MemFile {
            file_contents: Vec::new(),
            path: String::new()
        };
        
        let path_cstr = unsafe { CStr::from_ptr(z_name) };
        let path_str = path_cstr.to_str().expect("should be fine");
        mem_file.path = path_str.to_string();

        if !z_name.is_null() {
            write_file_to_vec_u8(path_str, &mut mem_file.file_contents)?;
        }
        
        unsafe { *p_file = *create_file_pointer( mem_file ); }
    
        Ok(())
    }

    fn delete(&mut self, z_name: *const c_char, sync_dir: i32) -> Result<()> {
        Err(Error::new(ErrorKind::DefineVfs(SQLITE_IOERR_DELETE)))
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
            ptr::copy_nonoverlapping(src_ptr, dst_ptr.cast(), name.as_bytes().len());
            name.into_raw();
        }

        Ok(())
    }

    /// From here onwards, all calls are redirected to the default vfs
    fn dl_open(&mut self, z_filename: *const c_char) -> *mut c_void {
        self.default_vfs.dl_open(z_filename)
    }

    fn dl_error(&mut self, n_byte: i32, z_err_msg: *mut c_char) {
        self.default_vfs.dl_error(n_byte, z_err_msg)
    }

    fn dl_sym(&mut self, arg2: *mut c_void, z_symbol: *const c_char)
        -> Option<unsafe extern "C" fn(arg1: *mut sqlite3_vfs, arg2: *mut c_void, z_symbol: *const c_char)> {
        self.default_vfs.dl_sym(arg2, z_symbol)
    }

    fn dl_close(&mut self, arg2: *mut c_void) {
        self.default_vfs.dl_close(arg2)
    }

    fn randomness(&mut self, n_byte: i32, z_out: *mut c_char) -> i32 {
         self.default_vfs.randomness(n_byte, z_out)
    }

    fn sleep(&mut self, microseconds: i32) -> i32 {
        self.default_vfs.sleep(microseconds)
    }

    fn current_time(&mut self, arg2: *mut f64) -> i32 {
        self.default_vfs.current_time(arg2)
    }

    fn get_last_error(&mut self, arg2: i32, arg3: *mut c_char) -> Result<()> {
        self.default_vfs.get_last_error(arg2, arg3)
    }

    fn current_time_int64(&mut self, arg2: *mut i64) -> i32 {
        self.default_vfs.current_time_int64(arg2)
    }

    fn set_system_call(&mut self, z_name: *const c_char, arg2: sqlite3_syscall_ptr) -> i32 {
        self.default_vfs.set_system_call(z_name, arg2)
    }

    fn get_system_call(&mut self, z_name: *const c_char) -> sqlite3_syscall_ptr {
        self.default_vfs.get_system_call(z_name)
    }

    fn next_system_call(&mut self, z_name: *const c_char) -> *const c_char {
        self.default_vfs.next_system_call(z_name)
    }
}

struct MemFile {
    file_contents: Vec<u8>,
    path: String,
}

impl Drop for MemFile {
    fn drop(&mut self) {
        if !self.file_contents.is_empty() {
            if let Err(err) = self.write_to_file() {
                eprintln!("Error writing to file {}: {}", self.path, err);
            }
        }
    }
}

impl MemFile {
    fn write_to_file(&self) -> io::Result<()> {
        let mut file = File::create(&self.path)?;
        file.write_all(&self.file_contents)?;
        Ok(())
    }
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

        let src_ptr = source[offset..(size-1)].as_ptr();
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

    fn lock(&mut self, arg2: i32) -> Result<()> {
        Ok(())
    }

    fn unlock(&mut self, arg2: i32) -> Result<()> {
        Ok(())
    }

    fn check_reserved_lock(&mut self, p_res_out: *mut i32) -> Result<()> {
        unsafe{ *p_res_out = 0; }
        Ok(())
    }

    fn file_control(&mut self, file: *mut sqlite3_file, op: i32, p_arg: *mut c_void) -> Result<()> {
        Ok(())
    }

    fn sector_size(&mut self) -> i32 {
        1024
    }

    fn device_characteristics(&mut self) -> i32 {
        SQLITE_IOCAP_ATOMIC | 
        SQLITE_IOCAP_POWERSAFE_OVERWRITE |
        SQLITE_IOCAP_SAFE_APPEND |
        SQLITE_IOCAP_SEQUENTIAL
    }

    fn shm_map(&mut self, i_pg: i32, pgsz: i32, arg2: i32, arg3: *mut *mut c_void) -> Result<()> {
        Err(Error::new(ErrorKind::DefineVfs(SQLITE_IOERR_SHMMAP)))
    }

    fn shm_lock(&mut self, offset: i32, n: i32, flags: i32) -> Result<()> {
        // SQLITE_IOERR_SHMLOCK is deprecated?
        Err(Error::new(ErrorKind::DefineVfs(SQLITE_IOERR_SHMLOCK)))
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

/// Usage: "ATTACH memvfs_from_file('test.db') AS inmem;"
fn vfs_from_file(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let path = api::value_text(&values[0]).map_err(|_| Error::new_message("can't determine path arg"))?;

    let text_output = format!("file:{}?vfs={}", path, EXTENSION_NAME);

    api::result_text(context, text_output);

    Ok(())
}

#[sqlite_entrypoint_permanent]
pub fn sqlite3_memvfs_init(db: *mut sqlite3) -> Result<()> {
    let name = CString::new(EXTENSION_NAME).expect("should be fine");
    let mem_vfs = MemVfs {
        default_vfs: unsafe {
            // pass thru
            DefaultVfs::from_ptr(sqlite3ext_vfs_find(ptr::null()))
        },
        name: name
    };
    let name_ptr = mem_vfs.name.as_ptr();

    let vfs: sqlite3_vfs = create_vfs(mem_vfs, name_ptr, 1024, None);
    register_vfs(vfs, true)?;

    let flags = FunctionFlags::UTF8 | FunctionFlags::DETERMINISTIC;
    define_scalar_function(db, "memvfs_from_file", 1, vfs_from_file, flags)?;

    Ok(())
}
