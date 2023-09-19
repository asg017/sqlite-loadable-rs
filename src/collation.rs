//! Define custom collations on sqlite3 database connections.

#![allow(clippy::not_unsafe_ptr_arg_deref)]
use crate::{
    errors::{Error, ErrorKind, Result},
    ext::sqlite3ext_collation_v2,
};
use sqlite3ext_sys::{sqlite3, SQLITE_OK};
use std::{ffi::CString, os::raw::c_void};

use sqlite3ext_sys::SQLITE_UTF8;

pub fn define_collation<F>(db: *mut sqlite3, name: &str, x_func: F) -> Result<()>
where
    F: Fn(&[u8], &[u8]) -> i32,
{
    let function_pointer: *mut F = Box::into_raw(Box::new(x_func));

    unsafe extern "C" fn compare_function_wrapper<F>(
        func: *mut std::os::raw::c_void,
        a_size: std::os::raw::c_int,
        a_pointer: *const std::os::raw::c_void,
        b_size: std::os::raw::c_int,
        b_pointer: *const ::std::os::raw::c_void,
    ) -> i32
    where
        F: Fn(&[u8], &[u8]) -> i32,
    {
        let boxed_function: *mut F = func.cast::<F>();
        // TODO: don't unwrap here. Maybe collation function should use &[u8] ?
        let a = std::slice::from_raw_parts(a_pointer as *const u8, a_size as usize);
        let b = std::slice::from_raw_parts(b_pointer as *const u8, b_size as usize);
        (*boxed_function)(a, b)
    }
    let cname = CString::new(name)?;
    let result = unsafe {
        sqlite3ext_collation_v2(
            db,
            cname.as_ptr(),
            SQLITE_UTF8 as i32,
            function_pointer.cast::<c_void>(),
            Some(compare_function_wrapper::<F>),
            None,
        )
    };

    if result != SQLITE_OK {
        Err(Error::new(ErrorKind::DefineScalarFunction(result)))
    } else {
        Ok(())
    }
}
