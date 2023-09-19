//! cargo build --example load_permanent
//! sqlite3 :memory: '.read examples/test.sql'

use sqlite_loadable::prelude::*;
use sqlite_loadable::{api, define_scalar_function, Result};
use std::os::raw::c_int;

// This function will be registered as a scalar function named "hello", and will be called on
// every invocation. It's goal is to return a string of "hello, NAME!" where NAME is the
// text value of the 1st argument.
pub fn hello(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let name = api::value_text(values.get(0).expect("1st argument as name"))?;

    api::result_text(context, format!("hello, {}!", name))?;
    Ok(())
}

// Exposes a extern C function named "sqlite3_hello_init" in the compiled dynamic library,
// the "entrypoint" that SQLite will use to load the extension.
// Notice the naming sequence - "sqlite3_" followed by "hello" then "_init". Since the
// compiled file is named "libhello.dylib" (or .so/.dll depending on your operating system),
// SQLite by default will look for an entrypoint called "sqlite3_hello_init".
// See "Loading an Extension" for more details <https://www.sqlite.org/loadext.html#loading_an_extension>
#[sqlite_entrypoint_permanent]
pub fn sqlite3_hello_init(db: *mut sqlite3) -> Result<()> {
    let flags = FunctionFlags::UTF8 | FunctionFlags::DETERMINISTIC;
    define_scalar_function(db, "hello", 1, hello, flags)?;
    Ok(())
}
