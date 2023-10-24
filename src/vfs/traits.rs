use std::io::Result;

use std::os::raw::{c_char, c_int, c_void};

use sqlite3ext_sys::sqlite3_file;

#[cfg(feature = "vfs_loadext")]
use sqlite3ext_sys::sqlite3_vfs;

#[cfg(feature = "vfs_syscall")]
use sqlite3ext_sys::{sqlite3_syscall_ptr, sqlite3_vfs};

// TODO compare performance of dynamic (indirection via trait) vs static dispatch (just callbacks)
/// See https://www.sqlite.org/c3ref/io_methods.html for hints on how to implement
pub trait SqliteIoMethods {
    fn close(&mut self, file: *mut sqlite3_file) -> Result<()>;
    fn read(
        &mut self,
        file: *mut sqlite3_file,
        buf: *mut c_void,
        i_amt: i32,
        i_ofst: i64,
    ) -> Result<()>;
    fn write(
        &mut self,
        file: *mut sqlite3_file,
        buf: *const c_void,
        i_amt: i32,
        i_ofst: i64,
    ) -> Result<()>;
    fn truncate(&mut self, file: *mut sqlite3_file, size: i64) -> Result<()>;
    fn sync(&mut self, file: *mut sqlite3_file, flags: c_int) -> Result<()>;
    fn file_size(&mut self, file: *mut sqlite3_file, p_size: *mut i64) -> Result<()>;
    fn lock(&mut self, file: *mut sqlite3_file, arg2: c_int) -> Result<c_int>;
    fn unlock(&mut self, file: *mut sqlite3_file, arg2: c_int) -> Result<c_int>;
    fn check_reserved_lock(
        &mut self,
        file: *mut sqlite3_file,
        p_res_out: *mut c_int,
    ) -> Result<bool>;
    fn file_control(
        &mut self,
        file: *mut sqlite3_file,
        op: c_int,
        p_arg: *mut c_void,
    ) -> Result<()>;
    fn sector_size(&mut self, file: *mut sqlite3_file) -> Result<c_int>;
    fn device_characteristics(&mut self, file: *mut sqlite3_file) -> Result<c_int>;
    fn shm_map(
        &mut self,
        file: *mut sqlite3_file,
        i_pg: c_int,
        pgsz: c_int,
        arg2: c_int,
        arg3: *mut *mut c_void,
    ) -> Result<()>;
    fn shm_lock(
        &mut self,
        file: *mut sqlite3_file,
        offset: c_int,
        n: c_int,
        flags: c_int,
    ) -> Result<()>;
    fn shm_barrier(&mut self, file: *mut sqlite3_file) -> Result<()>;
    fn shm_unmap(&mut self, file: *mut sqlite3_file, delete_flag: c_int) -> Result<()>;
    fn fetch(
        &mut self,
        file: *mut sqlite3_file,
        i_ofst: i64,
        i_amt: c_int,
        pp: *mut *mut c_void,
    ) -> Result<()>;
    fn unfetch(&mut self, file: *mut sqlite3_file, i_ofst: i64, p: *mut c_void) -> Result<()>;
}

// TODO compare dynamic (indirection via trait) vs static dispatch (just callbacks)
pub trait SqliteVfs {
    fn open(
        &mut self,
        z_name: *const c_char,
        p_file: *mut sqlite3_file,
        flags: c_int,
        p_out_flags: *mut c_int,
    ) -> Result<()>;

    fn delete(&mut self, z_name: *const c_char, sync_dir: c_int) -> Result<()>;

    fn access(&mut self, z_name: *const c_char, flags: c_int, p_res_out: *mut c_int) -> Result<()>;

    fn full_pathname(
        &mut self,
        z_name: *const c_char,
        n_out: c_int,
        z_out: *mut c_char,
    ) -> Result<()>;

    #[cfg(feature = "vfs_loadext")]
    fn dl_open(&mut self, z_filename: *const c_char) -> *mut c_void;

    #[cfg(feature = "vfs_loadext")]
    fn dl_error(&mut self, n_byte: c_int, z_err_msg: *mut c_char);

    #[cfg(feature = "vfs_loadext")]
    fn dl_sym(
        &mut self,
        arg2: *mut c_void,
        z_symbol: *const c_char,
    ) -> Option<
        unsafe extern "C" fn(arg1: *mut sqlite3_vfs, arg2: *mut c_void, z_symbol: *const c_char),
    >;

    #[cfg(feature = "vfs_loadext")]
    fn dl_close(&mut self, arg2: *mut c_void);

    fn randomness(&mut self, n_byte: c_int, z_out: *mut c_char) -> c_int;

    fn sleep(&mut self, microseconds: c_int) -> c_int;

    fn current_time(&mut self, arg2: *mut f64) -> c_int;

    fn get_last_error(&mut self, arg2: c_int, arg3: *mut c_char) -> Result<()>;

    fn current_time_int64(&mut self, arg2: *mut i64) -> c_int;

    #[cfg(feature = "vfs_syscall")]
    fn set_system_call(&mut self, z_name: *const c_char, arg2: sqlite3_syscall_ptr) -> c_int;

    #[cfg(feature = "vfs_syscall")]
    fn get_system_call(&mut self, z_name: *const c_char) -> sqlite3_syscall_ptr;

    #[cfg(feature = "vfs_syscall")]
    fn next_system_call(&mut self, z_name: *const c_char) -> *const c_char;
}
