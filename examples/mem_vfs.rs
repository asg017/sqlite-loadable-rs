//! cargo build --example mem_vfs
//! sqlite3 :memory: '.read examples/test.sql'
#![allow(unused)]

/// Inspired by https://www.sqlite.org/src/file/ext/misc/memvfs.c

use sqlite_loadable::{prelude::*, SqliteIoMethods, declare_file, declare_vfs};
use sqlite_loadable::{Result, vfs::traits::SqliteVfs};

use std::os::raw::{c_int, c_void, c_char};
use sqlite3ext_sys::{sqlite3_int64, sqlite3_syscall_ptr, sqlite3_file, sqlite3_vfs, sqlite3_vfs_register, sqlite3_io_methods};

struct MemVfs;

impl SqliteVfs for MemVfs {
    fn open(&mut self, z_name: *const c_char, flags: c_int) -> Result<()> {
        Ok(())
    }

    fn delete(&mut self, z_name: *const c_char, sync_dir: c_int) -> Result<()> {
        Ok(())
    }

    fn access(&mut self, z_name: *const c_char, flags: c_int, p_res_out: *mut c_int) -> Result<()> {
        Ok(())
    }

    fn full_pathname(&mut self, z_name: *const c_char, n_out: c_int, z_out: *mut c_char) -> Result<()> {
        Ok(())
    }

    fn dl_open(&mut self, z_filename: *const c_char) -> *mut c_void {
        std::ptr::null_mut()
    }

    fn dl_error(&mut self, n_byte: c_int, z_err_msg: *mut c_char) {
    }

    fn dl_sym(&mut self, arg2: *mut c_void, z_symbol: *const c_char) -> Option<unsafe extern "C" fn(arg1: *mut sqlite3_vfs, arg2: *mut c_void, z_symbol: *const c_char)> {
        None
    }

    fn dl_close(&mut self, arg2: *mut c_void) {
    }

    fn randomness(&mut self, n_byte: c_int, z_out: *mut c_char) -> Result<()> {
        Ok(())
    }

    fn sleep(&mut self, microseconds: c_int) -> Result<()> {
        Ok(())
    }

    fn current_time(&mut self, arg2: *mut f64) -> Result<()> {
        Ok(())
    }

    fn get_last_error(&mut self, arg2: c_int, arg3: *mut c_char) -> Result<()> {
        Ok(())
    }

    fn current_time_int64(&mut self, arg2: *mut sqlite3_int64) -> Result<()> {
        Ok(())
    }

    fn set_system_call(&mut self, z_name: *const c_char, arg2: sqlite3_syscall_ptr) -> Result<()> {
        Ok(())
    }

    fn get_system_call(&mut self, z_name: *const c_char) -> sqlite3_syscall_ptr {
        unsafe extern "C"
        fn meh() {}

        Some(meh)
    }

    fn next_system_call(&mut self, z_name: *const c_char) -> *const c_char {
        std::ptr::null()
    }

    fn init (&self) -> (sqlite3_file, Option<c_int>) {
        (declare_file::<MemIoMethods>(), None)
    }
}

struct MemIoMethods;

impl SqliteIoMethods for MemIoMethods {
    fn close(&mut self) -> Result<()> {
        Ok(())
    }

    fn read(&mut self, buf: *mut c_void, i_amt: c_int, i_ofst: sqlite3_int64) -> Result<()> {
        Ok(())
    }

    fn write(&mut self, buf: *const c_void, i_amt: c_int, i_ofst: sqlite3_int64) -> Result<()> {
        Ok(())
    }

    fn truncate(&mut self, size: sqlite3_int64) -> Result<()> {
        Ok(())
    }

    fn sync(&mut self, flags: c_int) -> Result<()> {
        Ok(())
    }

    fn file_size(&mut self, p_size: *mut sqlite3_int64) -> Result<()> {
        Ok(())
    }

    fn lock(&mut self, arg2: c_int) -> Result<()> {
        Ok(())
    }

    fn unlock(&mut self, arg2: c_int) -> Result<()> {
        Ok(())
    }

    fn check_reserved_lock(&mut self, p_res_out: *mut c_int) -> Result<()> {
        Ok(())
    }

    fn file_control(&mut self, op: c_int, p_arg: *mut c_void) -> Result<()> {
        Ok(())
    }

    fn sector_size(&mut self) -> Result<()> {
        Ok(())
    }

    fn device_characteristics(&mut self) -> Result<()> {
        Ok(())
    }

    fn shm_map(&mut self, i_pg: c_int, pgsz: c_int, arg2: c_int, arg3: *mut *mut c_void) -> Result<()> {
        Ok(())
    }

    fn shm_lock(&mut self, offset: c_int, n: c_int, flags: c_int) -> Result<()> {
        Ok(())
    }

    fn shm_barrier(&mut self) -> Result<()> {
        Ok(())
    }

    fn shm_unmap(&mut self, delete_flag: c_int) -> Result<()> {
        Ok(())
    }

    fn fetch(&mut self, i_ofst: sqlite3_int64, i_amt: c_int, pp: *mut *mut c_void) -> Result<()> {
        Ok(())
    }

    fn unfetch(&mut self, i_ofst: sqlite3_int64, p: *mut c_void) -> Result<()> {
        Ok(())
    }
}


#[sqlite_entrypoint_permanent]
pub fn sqlite3_memvfs_init(db: *mut sqlite3) -> Result<()> {
    // Why is this necessary in the original example?
    // mem_vfs.pAppData = sqlite3_vfs_find(0);

    let vfs: sqlite3_vfs = declare_vfs::<MemVfs>(MemVfs {}, "memvfs", 1024);
    let vfs_ptr = Box::into_raw(Box::new(vfs));

    unsafe { sqlite3_vfs_register(vfs_ptr, 1); } // TODO wrap, to use pretty syntax: api::register_vfs(vfs_ptr, true)?;

    Ok(())
}