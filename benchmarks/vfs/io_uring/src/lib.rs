#![allow(unused)]

pub mod ops;

use ops::Ops;

use sqlite_loadable::ext::{sqlite3ext_vfs_find, sqlite3ext_context_db_handle, sqlite3ext_file_control};
use sqlite_loadable::vfs::default::DefaultVfs;
use sqlite_loadable::vfs::file::FilePolymorph;
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

/// Inspired by https://www.sqlite.org/src/file/ext/misc/memvfs.c

const EXTENSION_NAME: &str = "iouring";

struct IoUringVfs {
    default_vfs: DefaultVfs,
    vfs_name: CString,
}

impl SqliteVfs for IoUringVfs {
    fn open(&mut self, z_name: *const c_char, p_file: *mut sqlite3_file, flags: i32, p_res_out: *mut i32) -> Result<()> {        

        let file_path = unsafe { CString::from_raw(z_name.cast_mut()) };

        let mut file = Ops::new(file_path.clone(), 32);

        file.open_file().map_err(|_| Error::new_message("can't open file"))?;

        unsafe { *p_file = *create_file_pointer( file ); }
    
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

impl SqliteIoMethods for Ops {
    fn close(&mut self) -> Result<()> {
        unsafe { self.o_close() }
    }

    fn read(&mut self, buf: *mut c_void, s: i32, ofst: i64) -> Result<()> {
        unsafe { self.o_read(ofst as u64, s as u32, buf) }
    }

    fn write(&mut self, buf: *const c_void, s: i32, ofst: i64) -> Result<()> {
        unsafe { self.o_write(buf, ofst as u64, s as u32) }
    }

    fn truncate(&mut self, size: i64) -> Result<()> {
        unsafe { self.o_truncate(size) }
    }

    fn sync(&mut self, flags: i32) -> Result<()> {
        Ok(())
    }

    fn file_size(&mut self, p_size: *mut i64) -> Result<()> {
        unsafe { self.o_file_size(p_size as *mut u64) }
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

    fn file_control(&mut self, op: i32, p_arg: *mut c_void) -> Result<()> {
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
        unsafe { self.o_fetch(ofst as u64, size as u32, pp) }
    }

    fn unfetch(&mut self, i_ofst: i64, p: *mut c_void) -> Result<()> {
        Ok(())
    }
}

/// Usage: "ATTACH iouring_vfs_from_file('test.db') AS inring;"
fn vfs_from_file(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let path = api::value_text(&values[0]).map_err(|_| Error::new_message("can't determine path arg"))?;

    let text_output = format!("file:{}?vfs={}", path, EXTENSION_NAME);

    api::result_text(context, text_output);

    Ok(())
}

// See Cargo.toml "[[lib]] name = ..." matches this function name
#[sqlite_entrypoint_permanent]
pub fn sqlite3_iouringvfs_init(db: *mut sqlite3) -> Result<()> {
    let vfs_name = CString::new(EXTENSION_NAME).expect("should be fine");
    let ring_vfs = IoUringVfs {
        default_vfs: unsafe {
            // pass thru
            DefaultVfs::from_ptr(sqlite3ext_vfs_find(ptr::null()))
        },
        vfs_name
    };

    let name_ptr = ring_vfs.vfs_name.as_ptr(); // allocation is bound to lifetime of struct

    let vfs: sqlite3_vfs = create_vfs(ring_vfs, name_ptr, 1024, None);

    register_vfs(vfs, true)?;

    let flags = FunctionFlags::UTF8 | FunctionFlags::DETERMINISTIC;
    define_scalar_function(db, "iouring_vfs_from_file", 1, vfs_from_file, flags)?;

    Ok(())
}

// TODO single parameter: n for workers = files, CPU-bounded?
// TODO parameter: one worker per db = vfs

// TODO Mutex lock on path

// TODO write a unit test for each operation that either vfs or file has to support
// put them in tests/, each op must be non-copy

// TODO write unit test that also benchmarks, use rusql and
// pre-generate random values, from non-io_uring to io_uring, and vice versa

// single process vs concurrent interleaving
// CRUD
// aggregates functions etc.

// table to table, 0 = no io uring, 1 = io uring supported
// 0 on 0
// 1 on 0
// 0 on 1
// 1 on 1

// TODO compare standard file vfs, mem vfs, io_uring+mmap vfs, mmap vfs

