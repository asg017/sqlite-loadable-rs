//! Define window functions on sqlite3 database connections.

#![allow(clippy::not_unsafe_ptr_arg_deref)]
use std::{
    ffi::CString,
    os::raw::{c_int, c_void},
    slice,
};

use crate::{
    api,
    constants::{SQLITE_INTERNAL, SQLITE_OKAY},
    errors::{Error, ErrorKind, Result},
    ext::sqlite3ext_create_window_function, FunctionFlags,
};
use sqlite3ext_sys::{sqlite3, sqlite3_context, sqlite3_user_data, sqlite3_value};

// TODO typedef repeating parameter types, across multiple files

fn create_window_function(
    db: *mut sqlite3,
    name: &str,
    num_args: c_int,
    func_flags: FunctionFlags,
    p_app: *mut c_void,
    x_step: Option<unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value)>,
    x_final: Option<unsafe extern "C" fn(*mut sqlite3_context)>,
    x_value: Option<unsafe extern "C" fn(*mut sqlite3_context)>,
    x_inverse: Option<unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value)>,
    destroy: Option<unsafe extern "C" fn(*mut c_void)>,
) -> Result<()> {

    let cname = CString::new(name)?;
    let result = unsafe {
        sqlite3ext_create_window_function(
            db,
            cname.as_ptr(),
            num_args,
            func_flags.bits(),
            p_app,
            x_step,
            x_final,
            x_value,
            x_inverse,
            destroy,
        )
    };

    if result != SQLITE_OKAY {
        Err(Error::new(ErrorKind::DefineWindowFunction(result)))
    } else {
        Ok(())
    }
}

pub struct WindowFunctionCallbacks
{
    x_step: fn(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()>,
    x_final: fn(context: *mut sqlite3_context) -> Result<()>,
    x_value: fn(context: *mut sqlite3_context) -> Result<()>,
    x_inverse: fn(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()>,
}

impl WindowFunctionCallbacks {
    pub fn new(
        x_step: fn(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()>,
        x_final: fn(context: *mut sqlite3_context) -> Result<()>,
        x_value: fn(context: *mut sqlite3_context) -> Result<()>,
        x_inverse: fn(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()>
    ) -> Self {
        WindowFunctionCallbacks {
            x_step,
            x_final,
            x_value,
            x_inverse,
        }
    }
}

pub struct WindowFunctionCallbacksWithAux<T>
{
    x_step: fn(context: *mut sqlite3_context, values: &[*mut sqlite3_value], aux: &T) -> Result<()>,
    x_final: fn(context: *mut sqlite3_context, aux: &T) -> Result<()>,
    x_value: fn(context: *mut sqlite3_context, aux: &T) -> Result<()>,
    x_inverse: fn(context: *mut sqlite3_context, values: &[*mut sqlite3_value], aux: &T) -> Result<()>,
}

impl<T> WindowFunctionCallbacksWithAux<T> {
    pub fn new(
        x_step: fn(context: *mut sqlite3_context, values: &[*mut sqlite3_value], aux: &T) -> Result<()>,
        x_final: fn(context: *mut sqlite3_context, aux: &T) -> Result<()>,
        x_value: fn(context: *mut sqlite3_context, aux: &T) -> Result<()>,
        x_inverse: fn(context: *mut sqlite3_context, values: &[*mut sqlite3_value], aux: &T) -> Result<()>
    ) -> Self {
        WindowFunctionCallbacksWithAux {
            x_step,
            x_final,
            x_value,
            x_inverse,
        }
    }
}

// TODO add documentation
// TODO add new test with aux object
// TODO parentheses matching
/// The aux parameter can be used to pass another context object altogether
pub fn define_window_function_with_aux<T>(
    db: *mut sqlite3,
    name: &str,
    num_args: c_int,
    func_flags: FunctionFlags,
    callbacks: WindowFunctionCallbacksWithAux<T>,
    aux: T,
) -> Result<()>
{
    let callbacks_pointer = Box::into_raw(Box::new(callbacks));
    let aux_pointer: *mut T = Box::into_raw(Box::new(aux));
    let app_pointer = Box::into_raw(Box::new((callbacks_pointer, aux_pointer)));
    
    unsafe extern "C" fn x_step_wrapper<T>(
        context: *mut sqlite3_context,
        argc: c_int,
        argv: *mut *mut sqlite3_value,
    )
    {
        let x = sqlite3_user_data(context).cast::<(*mut WindowFunctionCallbacksWithAux<T>, *mut T)>();
        let boxed_function = Box::from_raw((*x).0).as_ref().x_step;
        let aux = (*x).1;
        // .collect slows things waaaay down, so stick with slice for now
        let args = slice::from_raw_parts(argv, argc as usize);
        let b = Box::from_raw(aux);
        match boxed_function(context, args, &*b) {
            Ok(()) => (),
            Err(e) => {
                if api::result_error(context, &e.result_error_message()).is_err() {
                    api::result_error_code(context, SQLITE_INTERNAL);
                }
            }
        }
        Box::into_raw(b);
    }

    unsafe extern "C" fn x_inverse_wrapper<T>(
        context: *mut sqlite3_context,
        argc: c_int,
        argv: *mut *mut sqlite3_value,
    )
    {
        let x = sqlite3_user_data(context).cast::<(*mut WindowFunctionCallbacksWithAux<T>, *mut T)>();
        let boxed_function = Box::from_raw((*x).0).as_ref().x_inverse;
        let aux = (*x).1;
        // .collect slows things waaaay down, so stick with slice for now
        let args = slice::from_raw_parts(argv, argc as usize);
        let b = Box::from_raw(aux);
        match boxed_function(context, args, &*b) {
            Ok(()) => (),
            Err(e) => {
                if api::result_error(context, &e.result_error_message()).is_err() {
                    api::result_error_code(context, SQLITE_INTERNAL);
                }
            }
        }
        Box::into_raw(b);
    }

    unsafe extern "C" fn x_final_wrapper<T>(
        context: *mut sqlite3_context,
    )
    {
        let x = sqlite3_user_data(context).cast::<(*mut WindowFunctionCallbacksWithAux<T>, *mut T)>();
        let boxed_function = Box::from_raw((*x).0).as_ref().x_final;
        let aux = (*x).1;
        let b = Box::from_raw(aux);
        match boxed_function(context, &*b) {
            Ok(()) => (),
            Err(e) => {
                if api::result_error(context, &e.result_error_message()).is_err() {
                    api::result_error_code(context, SQLITE_INTERNAL);
                }
            }
        }
    }

    unsafe extern "C" fn x_value_wrapper<T>(
        context: *mut sqlite3_context,
    )
    {
        let x = sqlite3_user_data(context).cast::<(*mut WindowFunctionCallbacksWithAux<T>, *mut T)>();
        let boxed_function = Box::from_raw((*x).0).as_ref().x_value;
        let aux = (*x).1;
        let b = Box::from_raw(aux);
        match boxed_function(context, &*b) {
            Ok(()) => (),
            Err(e) => {
                if api::result_error(context, &e.result_error_message()).is_err() {
                    api::result_error_code(context, SQLITE_INTERNAL);
                }
            }
        }
    }

    create_window_function(
        db,
        name,
        num_args,
        func_flags,
        // app_pointer,
        app_pointer.cast::<c_void>(),
        Some(x_step_wrapper::<T>),
        Some(x_final_wrapper::<T>),
        Some(x_value_wrapper::<T>),
        Some(x_inverse_wrapper::<T>),
        None, // Note: release resources in x_final if necessary
        )


}

// TODO add documentation
// TODO parentheses matching
/// The aux parameter can be used to pass another context object altogether
pub fn define_window_function(
    db: *mut sqlite3,
    name: &str,
    num_args: c_int,
    func_flags: FunctionFlags,
    callbacks: WindowFunctionCallbacks,
) -> Result<()>
{
    let callbacks_pointer = Box::into_raw(Box::new(callbacks));
    let app_pointer = Box::into_raw(Box::new(callbacks_pointer));
    
    unsafe extern "C" fn x_step_wrapper(
        context: *mut sqlite3_context,
        argc: c_int,
        argv: *mut *mut sqlite3_value,
    )
    {
        let x = sqlite3_user_data(context).cast::<*mut WindowFunctionCallbacks>();
        let boxed_function = Box::from_raw(*x).as_ref().x_step;
        // .collect slows things waaaay down, so stick with slice for now
        let args = slice::from_raw_parts(argv, argc as usize);
        match boxed_function(context, args) {
            Ok(()) => (),
            Err(e) => {
                if api::result_error(context, &e.result_error_message()).is_err() {
                    api::result_error_code(context, SQLITE_INTERNAL);
                }
            }
        }
    }

    unsafe extern "C" fn x_inverse_wrapper(
        context: *mut sqlite3_context,
        argc: c_int,
        argv: *mut *mut sqlite3_value,
    )
    {
        let x = sqlite3_user_data(context).cast::<*mut WindowFunctionCallbacks>();
        let boxed_function = Box::from_raw(*x).as_ref().x_inverse;
        // .collect slows things waaaay down, so stick with slice for now
        let args = slice::from_raw_parts(argv, argc as usize);
        match boxed_function(context, args) {
            Ok(()) => (),
            Err(e) => {
                if api::result_error(context, &e.result_error_message()).is_err() {
                    api::result_error_code(context, SQLITE_INTERNAL);
                }
            }
        }
    }

    unsafe extern "C" fn x_final_wrapper(
        context: *mut sqlite3_context,
    )
    {
        let x = sqlite3_user_data(context).cast::<*mut WindowFunctionCallbacks>();
        let boxed_function = Box::from_raw(*x).as_ref().x_final;
        match boxed_function(context) {
            Ok(()) => (),
            Err(e) => {
                if api::result_error(context, &e.result_error_message()).is_err() {
                    api::result_error_code(context, SQLITE_INTERNAL);
                }
            }
        }
    }

    unsafe extern "C" fn x_value_wrapper(
        context: *mut sqlite3_context,
    )
    {
        let x = sqlite3_user_data(context).cast::<*mut WindowFunctionCallbacks>();
        let boxed_function = Box::from_raw(*x).as_ref().x_value;
        match boxed_function(context) {
            Ok(()) => (),
            Err(e) => {
                if api::result_error(context, &e.result_error_message()).is_err() {
                    api::result_error_code(context, SQLITE_INTERNAL);
                }
            }
        }
    }

    create_window_function(
        db,
        name,
        num_args,
        func_flags,
        // app_pointer,
        app_pointer.cast::<c_void>(),
        Some(x_step_wrapper),
        Some(x_final_wrapper),
        Some(x_value_wrapper),
        Some(x_inverse_wrapper),
        None, // Note: release resources in x_final if necessary
    )


}