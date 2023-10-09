#[cfg(feature = "exec")]
use sqlite_loadable::prelude::*;
#[cfg(feature = "exec")]
use sqlite_loadable::{api, define_scalar_function, Result};

#[cfg(feature = "exec")]
use sqlite_loadable::exec;

#[cfg(feature = "exec")]
pub fn t_values(context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
    let mut stmt =
        exec::Statement::prepare(api::context_db_handle(context), "select value from t").unwrap();
    let mut values: Vec<i64> = vec![];
    for row in stmt.execute() {
        let x = row.unwrap().get::<i64>(0);
        values.push(x.unwrap());
    }
    api::result_json(context, serde_json::json!(values))?;
    Ok(())
}

#[cfg(feature = "exec")]
#[sqlite_entrypoint]
pub fn sqlite3_exec_init(db: *mut sqlite3) -> Result<()> {
    let flags = FunctionFlags::UTF8 | FunctionFlags::DETERMINISTIC;
    define_scalar_function(db, "t_values", 0, t_values, flags)?;
    Ok(())
}

#[cfg(feature = "exec")]
#[cfg(test)]
mod tests {
    use super::*;

    use rusqlite::{ffi::sqlite3_auto_extension, Connection};

    #[test]
    fn test_rusqlite_auto_extension() {
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_exec_init as *const ())));
        }

        let db = Connection::open_in_memory().unwrap();

        db.execute(
            "create table t as select value from json_each('[7, 8, 9]')",
            [],
        )
        .unwrap();

        let result: String = db
            .query_row("SELECT t_values()", [], |row| row.get(0))
            .unwrap();
        assert_eq!(result, "[7,8,9]");
    }
}
