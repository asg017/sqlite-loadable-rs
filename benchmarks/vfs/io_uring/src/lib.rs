#![allow(unused)]
pub mod lock;
pub mod open;
pub mod ops;

use io_uring::IoUring;
use libc::name_t;

use sqlite_loadable::ext::{
    sqlite3_file, sqlite3_io_methods, sqlite3_syscall_ptr, sqlite3_vfs,
    sqlite3ext_context_db_handle, sqlite3ext_database_file_object, sqlite3ext_file_control,
    sqlite3ext_vfs_find, sqlite3ext_vfs_register,
};

use sqlite_loadable::vfs::shim::{ShimFile, ShimVfs};
use sqlite_loadable::vfs::vfs::create_vfs;

use sqlite_loadable::vfs::file::{create_io_methods_boxed, prepare_file_ptr, FileWithAux};
use sqlite_loadable::{
    api, define_scalar_function, prelude::*, register_boxed_vfs, vfs::traits::SqliteVfs,
    SqliteIoMethods,
};

use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::mem::MaybeUninit;
use std::os::raw::{c_char, c_void};
use std::rc::Rc;
use std::sync::Arc;
use std::{mem, ptr};

// use sqlite3ext_sys::{sqlite3_file, sqlite3_io_methods, sqlite3_syscall_ptr, sqlite3_vfs};
use sqlite3ext_sys::{SQLITE_CANTOPEN, SQLITE_IOERR_DELETE, SQLITE_OPEN_MAIN_DB, SQLITE_OPEN_WAL};

use std::io::{Error, ErrorKind, Result};

use crate::ops::OpsFd;

pub const EXTENSION_NAME: &str = "iouring";
pub const RING_SIZE: u32 = 32;

// TODO alternate implementation: write to mmap

struct IoUringVfs {
    default_vfs: ShimVfs,
    vfs_name: CString,
    ring: Rc<RefCell<IoUring>>,
}

impl SqliteVfs for IoUringVfs {
    fn open(
        &mut self,
        z_name: *const c_char,
        p_file: *mut sqlite3_file,
        flags: i32,
        p_res_out: *mut i32,
    ) -> Result<()> {
        let mut uring_ops = OpsFd::from_rc_refcell_ring(z_name as *mut _, self.ring.clone());

        uring_ops.open_file()?;

        unsafe { prepare_file_ptr(p_file, uring_ops) };

        Ok(())
    }

    fn delete(&mut self, z_name: *const c_char, sync_dir: i32) -> Result<()> {
        log::trace!("delete");

        let f = unsafe { CStr::from_ptr(z_name) };

        let file_path_str = f.to_str().expect("invalid UTF-8 string");

        if let Ok(metadata) = fs::metadata(std::path::Path::new(file_path_str)) {
            if metadata.is_file() {
                self.default_vfs.delete(z_name, sync_dir)?;
            } else {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    "pointer did not refer to valid file",
                ));
            }
        } else {
            return Err(Error::new(
                ErrorKind::NotFound,
                "failed to fetch metadata on file",
            ));
        }

        Ok(())
    }

    fn access(&mut self, z_name: *const c_char, flags: i32, p_res_out: *mut i32) -> Result<()> {
        log::trace!("access, flags {}", flags);

        self.default_vfs.access(z_name, flags, p_res_out)
    }

    fn full_pathname(
        &mut self,
        z_name: *const c_char,
        n_out: i32,
        z_out: *mut c_char,
    ) -> Result<()> {
        log::trace!("full_pathname");

        let name = unsafe { CStr::from_ptr(z_name) };
        let src_ptr = name.as_ptr();
        let dst_ptr = z_out;
        let len = name.to_bytes_with_nul().len();
        unsafe { ptr::copy_nonoverlapping(src_ptr, dst_ptr.cast(), len) };

        Ok(())
    }

    /// From here onwards, all calls are redirected to the default vfs, e.g. default unix vfs
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
        log::trace!("randomness");
        self.default_vfs.randomness(n_byte, z_out)
    }

    fn sleep(&mut self, microseconds: i32) -> i32 {
        log::trace!("sleep");
        self.default_vfs.sleep(microseconds)
    }

    fn current_time(&mut self, arg2: *mut f64) -> i32 {
        log::trace!("current_time");
        self.default_vfs.current_time(arg2)
    }

    fn get_last_error(&mut self, arg2: i32, arg3: *mut c_char) -> Result<()> {
        if !arg3.is_null() {
            let cstr = unsafe { CStr::from_ptr(arg3) };
            let err_str = cstr.to_string_lossy();
            log::trace!("get_last_error: {}", err_str);
        } else {
            log::trace!("get_last_error");
        }
        self.default_vfs.get_last_error(arg2, arg3)
    }

    fn current_time_int64(&mut self, arg2: *mut i64) -> i32 {
        log::trace!("current_time_int64");
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
#[sqlite_entrypoint]
pub fn sqlite3_iouringvfs_init(db: *mut sqlite3) -> sqlite_loadable::Result<()> {
    let vfs_name = CString::new(EXTENSION_NAME).expect("should be fine");

    let shimmed_name = CString::new("unix").expect("cannot find the default linux vfs");
    let shimmed_vfs_char = shimmed_name.as_ptr() as *const c_char;
    let shimmed_vfs = unsafe { sqlite3ext_vfs_find(shimmed_vfs_char) };

    let mut ring = Rc::new(RefCell::new(
        IoUring::new(RING_SIZE).expect("unable to create a ring"),
    ));

    let ring_vfs = IoUringVfs {
        default_vfs: unsafe {
            // pass thru
            ShimVfs::from_ptr(shimmed_vfs)
        },
        vfs_name,
        ring,
    };

    // allocation is bound to lifetime of struct
    let name_ptr = ring_vfs.vfs_name.as_ptr();

    // vfs_file_size == 0, fixes the stack smash, when Box does the clean up
    let vfs: sqlite3_vfs = create_vfs(
        ring_vfs,
        name_ptr,
        1024,
        // sqlite3 has ownership and thus manages the memory
        std::mem::size_of::<FileWithAux<OpsFd>>() as i32,
    );

    register_boxed_vfs(vfs, false)?;

    let flags = FunctionFlags::UTF8 | FunctionFlags::DETERMINISTIC;
    define_scalar_function(db, "io_uring_vfs_from_file", 1, vfs_from_file, flags)?;

    Ok(())
}
