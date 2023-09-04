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

fn create_window_function(
    db: *mut sqlite3,
    name: &str,
    num_args: c_int,
    func_flags: FunctionFlags,
    p_app: *mut c_void,
    x_step: unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value),
    x_final: unsafe extern "C" fn(*mut sqlite3_context),
    x_value: Option<unsafe extern "C" fn(*mut sqlite3_context)>,
    x_inverse: Option<unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value)>,
    destroy: unsafe extern "C" fn(*mut c_void),
) -> Result<()> {

    let cname = CString::new(name)?;
    let result = unsafe {
        sqlite3ext_create_window_function(
            db,
            cname.as_ptr(),
            num_args,
            func_flags.bits(),
            p_app,
            Some(x_step),
            Some(x_final),
            x_value,
            x_inverse,
            Some(destroy),
        )
    };

    if result != SQLITE_OKAY {
        Err(Error::new(ErrorKind::DefineWindowFunction(result)))
    } else {
        Ok(())
    }
}

type ValueCallback = fn(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()>;
type ContextCallback = fn(context: *mut sqlite3_context) -> Result<()>;

struct WindowFunctionCallbacks
{
    x_step: ValueCallback,
    x_final: ContextCallback,
    x_value: Option<ContextCallback>,
    x_inverse: Option<ValueCallback>,
}

impl WindowFunctionCallbacks {
    fn new(
        x_step: ValueCallback,
        x_final: ContextCallback,
        x_value: Option<ContextCallback>,
        x_inverse: Option<ValueCallback>
    ) -> Self {
        Self {
            x_step,
            x_final,
            x_value,
            x_inverse,
        }
    }
}

pub fn define_window_function(
    db: *mut sqlite3,
    name: &str,
    num_args: c_int,
    func_flags: FunctionFlags,
    x_step: ValueCallback,
    x_final: ContextCallback,
    x_value: Option<ContextCallback>,
    x_inverse: Option<ValueCallback>
) -> Result<()>
{
    let app_pointer = Box::into_raw(
        Box::new(
            WindowFunctionCallbacks::new(x_step, x_final, x_value, x_inverse)
        )
    );
    
    unsafe extern "C" fn x_step_wrapper(
        context: *mut sqlite3_context,
        argc: c_int,
        argv: *mut *mut sqlite3_value,
    )
    {
        let x = sqlite3_user_data(context).cast::<WindowFunctionCallbacks>();
        let args = slice::from_raw_parts(argv, argc as usize);
        match ((*x).x_step)(context, args) {
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
        let x = sqlite3_user_data(context).cast::<WindowFunctionCallbacks>();
        if let Some(x_inverse) = (*x).x_inverse {
            let args = slice::from_raw_parts(argv, argc as usize);
            match x_inverse(context, args) {
                Ok(()) => (),
                Err(e) => {
                    if api::result_error(context, &e.result_error_message()).is_err() {
                        api::result_error_code(context, SQLITE_INTERNAL);
                    }
                }
            }    
        }
    }

    unsafe extern "C" fn x_final_wrapper(
        context: *mut sqlite3_context,
    )
    {
        let x = sqlite3_user_data(context).cast::<WindowFunctionCallbacks>();
        match ((*x).x_final)(context) {
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
        let x = sqlite3_user_data(context).cast::<WindowFunctionCallbacks>();
        if let Some(x_value) = (*x).x_value {
            match x_value(context) {
                Ok(()) => (),
                Err(e) => {
                    if api::result_error(context, &e.result_error_message()).is_err() {
                        api::result_error_code(context, SQLITE_INTERNAL);
                    }
                }
            }    
        }
    }

    unsafe extern "C" fn destroy(
        p_app: *mut c_void,
    )
    {
        let callbacks = p_app.cast::<WindowFunctionCallbacks>();
        let _ = Box::from_raw(callbacks); // drop
    }

    create_window_function(
        db,
        name,
        num_args,
        func_flags,
        app_pointer.cast::<c_void>(),
        x_step_wrapper,
        x_final_wrapper,
        Some(x_value_wrapper),
        Some(x_inverse_wrapper),
        destroy,
    )


}