//! cargo build --example sum_int_aux

use sqlite_loadable::prelude::*;
use sqlite_loadable::window::define_window_function_with_aux;
use sqlite_loadable::{api, Result};

/// Example inspired by sqlite3's sumint
/// https://www.sqlite.org/windowfunctions.html#user_defined_aggregate_window_functions
pub fn x_step(_context: *mut sqlite3_context, values: &[*mut sqlite3_value], i: &mut i64) -> Result<()> {
    assert!(values.len() == 1);
    let new_value = api::value_int64(values.get(0).expect("should be one"));
    *i = *i + new_value;
    Ok(())
}


pub fn x_final(context: *mut sqlite3_context, i: &mut i64) -> Result<()> {
    api::result_int64(context, *i);
    Ok(())
}


pub fn x_value(context: *mut sqlite3_context, i: &mut i64) -> Result<()> {
    api::result_int64(context, *i);
    Ok(())
}

pub fn x_inverse(_context: *mut sqlite3_context, values: &[*mut sqlite3_value], i: &mut i64) -> Result<()> {
    assert!(values.len() == 1);
    let new_value = api::value_int64(values.get(0).expect("should be one"));
    *i = *i - new_value;
    Ok(())
}

#[sqlite_entrypoint]
pub fn sqlite3_sumintaux_init(db: *mut sqlite3) -> Result<()> {
    let flags = FunctionFlags::UTF8 | FunctionFlags::DETERMINISTIC;
    define_window_function_with_aux::<i64>(
        db, "sumint_aux", -1, flags,
        x_step, x_final, Some(x_value), Some(x_inverse),
        0,
    )?;
    Ok(())
}