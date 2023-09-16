use crate::errors::Result;

use std::os::raw::{c_int, c_void, c_char};

use sqlite3ext_sys::{sqlite3_int64, sqlite3_syscall_ptr, sqlite3_file, sqlite3_vfs};

// TODO compare performance of dynamic (indirection via trait) vs static dispatch (just callbacks)
/// See https://www.sqlite.org/c3ref/io_methods.html for hints on how to implement
pub trait SqliteIoMethods {
    fn close(&mut self) -> Result<()>;
    fn read(
        &mut self,
        buf: *mut c_void,
        i_amt: i32,
        i_ofst: i64,
    ) -> Result<()>;
    fn write(
        &mut self,
        buf: *const c_void,
        i_amt: i32,
        i_ofst: i64,
    ) -> Result<()>;
    fn truncate(&mut self, size: i64) -> Result<()>;
    fn sync(&mut self, flags: c_int) -> Result<()>;
    fn file_size(&mut self, p_size: *mut sqlite3_int64) -> Result<()>;
    fn lock(&mut self, arg2: c_int) -> Result<()>;
    fn unlock(&mut self, arg2: c_int) -> Result<()>;
    fn check_reserved_lock(
        &mut self,
        p_res_out: *mut c_int,
    ) -> Result<()>;
    fn file_control(
        &mut self,
        op: c_int,
        p_arg: *mut c_void,
    ) -> Result<()>;
    fn sector_size(&mut self) -> c_int;
    fn device_characteristics(&mut self) -> c_int;
    fn shm_map(
        &mut self,
        i_pg: c_int,
        pgsz: c_int,
        arg2: c_int,
        arg3: *mut *mut c_void,
    ) -> Result<()>;
    fn shm_lock(
        &mut self,
        offset: c_int,
        n: c_int,
        flags: c_int,
    ) -> Result<()>;
    fn shm_barrier(&mut self) -> Result<()>;
    fn shm_unmap(
        &mut self,
        delete_flag: c_int,
    ) -> Result<()>;
    fn fetch(
        &mut self,
        i_ofst: sqlite3_int64,
        i_amt: c_int,
        pp: *mut *mut c_void,
    ) -> Result<()>;
    fn unfetch(
        &mut self,
        i_ofst: sqlite3_int64,
        p: *mut c_void,
    ) -> Result<()>;
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

    fn delete(
        &mut self,
        z_name: *const c_char,
        sync_dir: c_int,
    ) -> Result<()>;

    fn access(
        &mut self,
        z_name: *const c_char,
        flags: c_int,
        p_res_out: *mut c_int,
    ) -> Result<()>;

    fn full_pathname(
        &mut self,
        z_name: *const c_char,
        n_out: c_int,
        z_out: *mut c_char,
    ) -> Result<()>;

    fn dl_open(
        &mut self,
        z_filename: *const c_char,
    ) -> *mut c_void;

    fn dl_error(
        &mut self,
        n_byte: c_int,
        z_err_msg: *mut c_char,
    );

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
    >;

    fn dl_close(&mut self, arg2: *mut c_void);

    fn randomness(
        &mut self,
        n_byte: c_int,
        z_out: *mut c_char,
    ) -> c_int;

    fn sleep(
        &mut self,
        microseconds: c_int,
    ) -> c_int;

    fn current_time(&mut self, arg2: *mut f64) -> c_int;

    fn get_last_error(
        &mut self,
        arg2: c_int,
        arg3: *mut c_char,
    ) -> Result<()>;

    fn current_time_int64(
        &mut self,
        arg2: *mut sqlite3_int64,
    ) -> c_int;

    fn set_system_call(
        &mut self,
        z_name: *const c_char,
        arg2: sqlite3_syscall_ptr,
    ) -> c_int;

    fn get_system_call(
        &mut self,
        z_name: *const c_char,
    ) -> sqlite3_syscall_ptr;

    fn next_system_call(
        &mut self,
        z_name: *const c_char,
    ) -> *const c_char;
}
