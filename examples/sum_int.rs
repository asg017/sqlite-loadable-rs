//! cargo build --example sum_int
//! sqlite3 :memory: '.read examples/test.sql'

use libsqlite3_sys::sqlite3_int64;
use sqlite_loadable::prelude::*;
use sqlite_loadable::window::{WindowFunctionCallbacks, define_window_function};
use sqlite_loadable::{api, Result};

/// Example inspired by sqlite3's sumint
/// https://www.sqlite.org/windowfunctions.html#user_defined_aggregate_window_functions
pub fn x_step(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    assert!(values.len() == 1);
    let new_value = api::value_int64(values.get(0).expect("should be one"));
    let previous_value = api::get_aggregate_context_value::<sqlite3_int64>(context)?;
    api::set_aggregate_context_value::<sqlite3_int64>(context, previous_value + new_value)?;
    Ok(())
}


pub fn x_final(context: *mut sqlite3_context) -> Result<()> {
    let value = api::get_aggregate_context_value::<sqlite3_int64>(context)?;
    api::result_int64(context, value);
    Ok(())
}


pub fn x_value(context: *mut sqlite3_context) -> Result<()> {
    let value = api::get_aggregate_context_value::<sqlite3_int64>(context)?;
    api::result_int64(context, value);
    Ok(())
}

pub fn x_inverse(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    assert!(values.len() == 1);
    let new_value = api::value_int64(values.get(0).expect("should be one"));
    let previous_value = api::get_aggregate_context_value::<sqlite3_int64>(context)?;
    api::set_aggregate_context_value::<sqlite3_int64>(context, previous_value - new_value)?;
    Ok(())
}

#[sqlite_entrypoint]
pub fn sqlite3_sumint_init(db: *mut sqlite3) -> Result<()> {
    let flags = FunctionFlags::UTF8 | FunctionFlags::DETERMINISTIC;
    define_window_function(db, "sumint", -1, flags,
    WindowFunctionCallbacks::new(x_step, x_final, x_value, x_inverse))?;
    Ok(())
}

