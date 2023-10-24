#![allow(unused)]
pub mod ops;

pub(crate) mod connection;
pub(crate) mod lock;

use ops::Ops;

use sqlite_loadable::ext::{
    sqlite3ext_context_db_handle, sqlite3ext_database_file_object, sqlite3ext_file_control,
    sqlite3ext_vfs_find, sqlite3ext_vfs_register,
};
use sqlite_loadable::vfs::default::{DefaultFile, DefaultVfs};
use sqlite_loadable::vfs::vfs::create_vfs;

use sqlite_loadable::vfs::file::{FileWithAux, MethodsWithAux};
use sqlite_loadable::{
    api, create_file_pointer, define_scalar_function, prelude::*, register_boxed_vfs,
    vfs::traits::SqliteVfs, SqliteIoMethods,
};
use url::Url;

use std::ffi::{CStr, CString};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::os::raw::{c_char, c_void};
use std::{mem, ptr};

use sqlite3ext_sys::{sqlite3_file, sqlite3_io_methods, sqlite3_syscall_ptr, sqlite3_vfs};
use sqlite3ext_sys::{SQLITE_CANTOPEN, SQLITE_IOERR_DELETE, SQLITE_OPEN_MAIN_DB, SQLITE_OPEN_WAL};

use std::io::{Error, ErrorKind, Result};

/// Inspired by https://www.sqlite.org/src/file/ext/misc/memvfs.c

// Based on the following article for default vfs, mem vfs and io uring vfs
// source: https://voidstar.tech/sqlite_insert_speed
// source: https://www.sqlite.org/speed.html

const EXTENSION_NAME: &str = "iouring";

struct IoUringVfs {
    default_vfs: DefaultVfs,
    vfs_name: CString,
}

impl SqliteVfs for IoUringVfs {
    fn open(
        &mut self,
        z_name: *const c_char,
        p_file: *mut sqlite3_file,
        flags: i32,
        p_res_out: *mut i32,
    ) -> Result<()> {
        let file_path = unsafe { CStr::from_ptr(z_name) };

        let db_file_obj = unsafe { sqlite3ext_database_file_object(file_path.as_ptr()) };
        let mut file = Ops::new(file_path.to_owned(), 32);

        file.open_file()
            .map_err(|_| Error::new(ErrorKind::Other, "can't open file"))?;

        unsafe {
            *p_file = *create_file_pointer(file);
        }

        Ok(())
    }

    fn delete(&mut self, z_name: *const c_char, sync_dir: i32) -> Result<()> {
        let file_path_cstr = unsafe { CStr::from_ptr(z_name) };
        let file_path = file_path_cstr.to_str().unwrap();
        if let Ok(metadata) = fs::metadata(file_path) {
            if metadata.is_file() {
                self.default_vfs.delete(z_name, sync_dir);
            }
        }
        Ok(())
    }

    fn access(&mut self, z_name: *const c_char, flags: i32, p_res_out: *mut i32) -> Result<()> {
        unsafe {
            // *p_res_out = if self.wal { 1 } else { 0 };
            *p_res_out = 0;
        }
        Ok(())
    }

    fn full_pathname(
        &mut self,
        z_name: *const c_char,
        n_out: i32,
        z_out: *mut c_char,
    ) -> Result<()> {
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

/// Usage: "ATTACH io_uring_vfs_from_file('test.db') AS inring;"
fn vfs_from_file(
    context: *mut sqlite3_context,
    values: &[*mut sqlite3_value],
) -> sqlite_loadable::Result<()> {
    let path = api::value_text(&values[0])?;

    let text_output = format!("file:{}?vfs={}", path, EXTENSION_NAME);

    api::result_text(context, text_output);

    Ok(())
}

// See Cargo.toml "[[lib]] name = ..." matches this function name
#[sqlite_entrypoint_permanent]
pub fn sqlite3_iouringvfs_init(db: *mut sqlite3) -> sqlite_loadable::Result<()> {
    let vfs_name = CString::new(EXTENSION_NAME).expect("should be fine");

    let shimmed_name = CString::new("unix-dotfile").unwrap();
    let shimmed_vfs_char = shimmed_name.as_ptr() as *const c_char;
    let shimmed_vfs = unsafe { sqlite3ext_vfs_find(shimmed_vfs_char) };

    let ring_vfs = IoUringVfs {
        default_vfs: unsafe {
            // pass thru
            DefaultVfs::from_ptr(shimmed_vfs)
        },
        vfs_name,
    };

    // allocation is bound to lifetime of struct
    let name_ptr = ring_vfs.vfs_name.as_ptr();

    // let file_size = std::mem::size_of::<FileWithAux<Ops>>();
    // let vfs: sqlite3_vfs = create_vfs(ring_vfs, name_ptr, 1024, file_size.try_into().unwrap());

    // vfs_file_size == 0, fixes the stack smash, when Box does the clean up
    let vfs: sqlite3_vfs = create_vfs(ring_vfs, name_ptr, 1024, 0);

    register_boxed_vfs(vfs, false)?;

    let flags = FunctionFlags::UTF8 | FunctionFlags::DETERMINISTIC;
    define_scalar_function(db, "io_uring_vfs_from_file", 1, vfs_from_file, flags)?;

    Ok(())
}
