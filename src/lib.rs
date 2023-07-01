#![doc = include_str!("../README.md")]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod api;
pub mod collation;
mod constants;
pub mod entrypoints;
pub mod errors;
pub mod exec;
pub mod ext; // TODO dont expose
pub mod prelude;
pub mod scalar;
pub mod table;
pub mod vtab_argparse;

#[doc(inline)]
pub use errors::{Error, ErrorKind, Result};

#[doc(inline)]
pub use scalar::{define_scalar_function, define_scalar_function_with_aux, FunctionFlags};

#[doc(inline)]
pub use collation::define_collation;

#[doc(inline)]
pub use table::{
    define_table_function, define_virtual_table, define_virtual_table_writeable,
    define_virtual_table_writeablex, BestIndexError,
};

#[cfg(test)]
mod tests {
    use super::prelude::*;
    use super::*;
    use rusqlite::{ffi::sqlite3_auto_extension, Connection};

    pub fn hello(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
        let name = api::value_text(values.get(0).expect("1st argument as name"))?;

        api::result_text(context, format!("hello, {}!", name))?;
        Ok(())
    }
    pub fn t_values(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
        let mut stmt =
            exec::Statement::prepare(api::context_db_handle(context), "select value from t")
                .unwrap();
        let mut values: Vec<i64> = vec![];
        for row in stmt.execute() {
            let x = row.unwrap().get::<i64>(0);
            values.push(x.unwrap());
        }
        api::result_json(context, serde_json::json!(values))?;
        Ok(())
    }

    #[sqlite_entrypoint]
    pub fn sqlite3_hello_init(db: *mut sqlite3) -> Result<()> {
        let flags = FunctionFlags::UTF8 | FunctionFlags::DETERMINISTIC;
        define_scalar_function(db, "hello", 1, hello, flags)?;
        define_scalar_function(db, "t_values", 0, t_values, flags)?;
        Ok(())
    }

    #[test]
    fn hello_extension() {
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_hello_init as *const ())));
        }
        let db = Connection::open_in_memory().unwrap();

        db.execute(
            "create table t as select value from json_each('[7, 8, 9]')",
            [],
        )
        .unwrap();

        let name: String = db
            .query_row("SELECT hello(?)", ["alex"], |row| row.get(0))
            .unwrap();
        assert_eq!(name, "hello, alex!");
        let name: String = db
            .query_row("SELECT t_values()", [], |row| row.get(0))
            .unwrap();
        assert_eq!(name, "[7,8,9]");
    }
}
