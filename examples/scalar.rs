//! cargo build --example scalar
//! sqlite3 :memory: '.read examples/test.sql'

use sqlite_loadable::prelude::*;
use sqlite_loadable::{api, define_scalar_function, Result};
use std::os::raw::c_int;

// yo()
fn yo(context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
    api::result_text(context, "yo")?;
    Ok(())
}

// surround_rs(name)
fn surround(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let value = api::value_text(values.get(0).expect("1st argument as name"))?;
    api::result_text(context, format!("x{}x", value))?;
    Ok(())
}

// add_rs(a, b)
fn add(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let a = api::value_int(values.get(0).expect("1st argument"));
    let b = api::value_int(values.get(1).expect("2nd argument"));
    api::result_int(context, a + b);
    Ok(())
}

// connect(seperator, string1, ...)
fn connect(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let seperator = api::value_text(values.get(0).expect("1st argument"))?;
    let strings: Vec<&str> = values
        .get(1..)
        .expect("more than 1 argument to be given")
        .iter()
        .filter_map(|v| api::value_text(v).ok())
        .collect();
    api::result_text(context, strings.join(seperator))?;
    Ok(())
}

#[sqlite_entrypoint]
pub fn sqlite3_scalarrs_init(db: *mut sqlite3) -> Result<()> {
    let flags = FunctionFlags::UTF8 | FunctionFlags::DETERMINISTIC;
    define_scalar_function(db, "surround_rs", 1, surround, flags)?;
    define_scalar_function(db, "connect", -1, connect, flags)?;
    define_scalar_function(db, "yo_rs", 0, yo, flags)?;
    define_scalar_function(db, "add_rs", 2, add, flags)?;
    Ok(())
}
