//! Define scalar functions on sqlite3 database connections.

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
    ext::{
        sqlite3, sqlite3_context, sqlite3_value,
        sqlite3ext_user_data,
        sqlite3ext_create_function_v2,
    }, FunctionFlags,
};

fn create_function_v2(
    db: *mut sqlite3,
    name: &str,
    num_args: c_int,
    func_flags: FunctionFlags,
    p_app: *mut c_void,
    x_func: Option<unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value)>,
    x_step: Option<unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value)>,
    x_final: Option<unsafe extern "C" fn(*mut sqlite3_context)>,
    destroy: Option<unsafe extern "C" fn(*mut c_void)>,
) -> Result<()>
{
    let cname = CString::new(name)?;
    let result = unsafe {
        sqlite3ext_create_function_v2(
            db,
            cname.as_ptr(),
            num_args,
            func_flags.bits(),
            p_app,
            x_func,
            x_step,
            x_final,
            destroy,
        )
    };

    if result != SQLITE_OKAY {
        Err(Error::new(ErrorKind::DefineScalarFunction(result)))
    } else {
        Ok(())
    }
}

/// Defines a new scalar function on the given database connection.
///
/// # Example
/// ```rust
/// fn xyz_version(context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
///   context_result_text(context, &format!("v{}", env!("CARGO_PKG_VERSION")))?;
///   Ok(())
/// }
///
/// define_scalar_function(db, "xyz_version", 0, xyz_version)?;
/// ```
pub fn define_scalar_function<F>(
    db: *mut sqlite3,
    name: &str,
    num_args: c_int,
    x_func: F,
    func_flags: FunctionFlags,
) -> Result<()>
where
    // TODO - can we wrap the context arg with a safe/ergonomic struct?
    // calling `context_result_text(context, "foo")` is long, but maybe
    // `context.result_text("foo")` with a special wrapper struct can be
    // as fast
    F: Fn(*mut sqlite3_context, &[*mut sqlite3_value]) -> Result<()>,
{
    let function_pointer: *mut F = Box::into_raw(Box::new(x_func));

    unsafe extern "C" fn x_func_wrapper<F>(
        context: *mut sqlite3_context,
        argc: c_int,
        argv: *mut *mut sqlite3_value,
    ) where
        F: Fn(*mut sqlite3_context, &[*mut sqlite3_value]) -> Result<()>,
    {
        let boxed_function: *mut F = sqlite3ext_user_data(context).cast::<F>();
        // .collect slows things waaaay down, so stick with slice for now
        let args = slice::from_raw_parts(argv, argc as usize);
        match (*boxed_function)(context, args) {
            Ok(()) => (),
            Err(e) => {
                if api::result_error(context, &e.result_error_message()).is_err() {
                    api::result_error_code(context, SQLITE_INTERNAL);
                }
            }
        }
    }
    create_function_v2(
        db,
        name,
        num_args,
        func_flags,
        function_pointer.cast::<c_void>(),
        Some(x_func_wrapper::<F>),
        None,
        None,
        None,
    )
}

/// Defines a new scalar function, but with the added ability to pass in an arbritary
/// application "pointer" as any rust type. Can be accessed in the callback
/// function as the 3rd argument, as a reference.
/// <https://www.sqlite.org/c3ref/create_function.html#:~:text=The%20fifth%20parameter%20is%20an%20arbitrary%20pointer.>
pub fn define_scalar_function_with_aux<F, T>(
    db: *mut sqlite3,
    name: &str,
    num_args: c_int,
    x_func: F,
    func_flags: FunctionFlags,
    aux: T,
) -> Result<()>
where
    F: Fn(*mut sqlite3_context, &[*mut sqlite3_value], &T) -> Result<()>,
{
    let function_pointer: *mut F = Box::into_raw(Box::new(x_func));
    let aux_pointer: *mut T = Box::into_raw(Box::new(aux));
    let app_pointer = Box::into_raw(Box::new((function_pointer, aux_pointer)));

    unsafe extern "C" fn x_func_wrapper<F, T>(
        context: *mut sqlite3_context,
        argc: c_int,
        argv: *mut *mut sqlite3_value,
    ) where
        F: Fn(*mut sqlite3_context, &[*mut sqlite3_value], &T) -> Result<()>,
    {
        let x = sqlite3ext_user_data(context).cast::<(*mut F, *mut T)>();
        let boxed_function = (*x).0;
        let aux = (*x).1;
        // .collect slows things waaaay down, so stick with slice for now
        let args = slice::from_raw_parts(argv, argc as usize);
        let b = Box::from_raw(aux);
        match (*boxed_function)(context, args, &*b) {
            Ok(()) => (),
            Err(e) => {
                if api::result_error(context, &e.result_error_message()).is_err() {
                    api::result_error_code(context, SQLITE_INTERNAL);
                }
            }
        }
        Box::into_raw(b);
    }
    create_function_v2(
        db,
        name,
        num_args,
        func_flags,
        app_pointer.cast::<c_void>(),
        Some(x_func_wrapper::<F, T>),
        None,
        None,
        None,
    )
}

pub fn delete_scalar_function(
    db: *mut sqlite3,
    name: &str,
    num_args: c_int,
    func_flags: FunctionFlags,
) -> Result<()> {
    create_function_v2(
        db,
        name,
        num_args,
        func_flags,
        std::ptr::null_mut(),
        None,
        None,
        None,
        None,
    )
}

// TODO only used for find_function, probably can combine with that return type?
pub fn scalar_function_raw<F>(
    x_func: F,
) -> unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value)
where
    F: Fn(*mut sqlite3_context, &[*mut sqlite3_value]) -> Result<()>,
{
    // TODO: how does x_func even get called here???
    let _function_pointer: *mut F = Box::into_raw(Box::new(x_func));

    unsafe extern "C" fn x_func_wrapper<F>(
        context: *mut sqlite3_context,
        argc: c_int,
        argv: *mut *mut sqlite3_value,
    ) where
        F: Fn(*mut sqlite3_context, &[*mut sqlite3_value]) -> Result<()>,
    {
        let boxed_function: *mut F = sqlite3ext_user_data(context).cast::<F>();
        let args = slice::from_raw_parts(argv, argc as usize);
        match (*boxed_function)(context, args) {
            Ok(()) => (),
            Err(e) => {
                if api::result_error(context, &e.result_error_message()).is_err() {
                    api::result_error_code(context, SQLITE_INTERNAL);
                }
            }
        }
    }

    x_func_wrapper::<F>
}

pub fn scalar_function_raw_with_aux<F, T>(
    x_func: F,
    aux: T,
) -> (
    unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value),
    *mut c_void,
)
where
    F: Fn(*mut sqlite3_context, &[*mut sqlite3_value], &T) -> Result<()>,
{
    // TODO: how does x_func even get called here???
    let function_pointer: *mut F = Box::into_raw(Box::new(x_func));
    let aux_pointer: *mut T = Box::into_raw(Box::new(aux));
    let app_pointer = Box::into_raw(Box::new((function_pointer, aux_pointer)));

    unsafe extern "C" fn x_func_wrapper<F, T>(
        context: *mut sqlite3_context,
        argc: c_int,
        argv: *mut *mut sqlite3_value,
    ) where
        F: Fn(*mut sqlite3_context, &[*mut sqlite3_value], &T) -> Result<()>,
    {
        let x = sqlite3ext_user_data(context).cast::<(*mut F, *mut T)>();
        let boxed_function = (*x).0;
        let aux = (*x).1;

        let args = slice::from_raw_parts(argv, argc as usize);
        let b = Box::from_raw(aux);
        match (*boxed_function)(context, args, &*b) {
            Ok(()) => (),
            Err(e) => {
                if api::result_error(context, &e.result_error_message()).is_err() {
                    api::result_error_code(context, SQLITE_INTERNAL);
                }
            }
        }
        Box::into_raw(b);
    }

    (x_func_wrapper::<F, T>, app_pointer.cast())
}
