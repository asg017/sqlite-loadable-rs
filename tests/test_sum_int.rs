use libsqlite3_sys::sqlite3_int64;
use sqlite_loadable::prelude::*;
use sqlite_loadable::window::define_window_function;
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
    define_window_function(
        db, "sumint", -1, flags,
        x_step, x_final, Some(x_value), Some(x_inverse),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use rusqlite::{ffi::sqlite3_auto_extension, Connection};

    #[test]
    fn test_rusqlite_auto_extension() {
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_sumint_init as *const ())));
        }

        let conn = Connection::open_in_memory().unwrap();

        let _ = conn
            .execute("CREATE TABLE t3(x TEXT, y INTEGER)", ());

        let _ = conn
            .execute("INSERT INTO t3 VALUES ('a', 4), ('b', 5), ('c', 3), ('d', 8), ('e', 1)", ());

        let result: sqlite3_int64 = conn.query_row("SELECT x, sumint(y) OVER (
            ORDER BY x ROWS BETWEEN 1 PRECEDING AND 1 FOLLOWING
          ) AS sum_y
          FROM t3 ORDER BY x", (), |x| x.get(1)).unwrap();


        assert_eq!(result, 9);
    }
}
