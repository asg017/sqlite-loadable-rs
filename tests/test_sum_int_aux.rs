use libsqlite3_sys::sqlite3_int64;
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

pub fn x_inverse(context: *mut sqlite3_context, values: &[*mut sqlite3_value], i: &mut i64) -> Result<()> {
    assert!(values.len() == 1);
    let new_value = api::value_int64(values.get(0).expect("should be one"));
    api::set_aggregate_context_value::<sqlite3_int64>(context, *i - new_value)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    use rusqlite::{ffi::sqlite3_auto_extension, Connection};

    #[test]
    fn test_rusqlite_auto_extension() {
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_sumintaux_init as *const ())));
        }

        let conn = Connection::open_in_memory().unwrap();

        let _ = conn
            .execute("CREATE TABLE t3(x TEXT, y INTEGER)", ());

        let _ = conn
            .execute("INSERT INTO t3 VALUES ('a', 4), ('b', 5), ('c', 3), ('d', 8), ('e', 1)", ());

        let result: sqlite3_int64 = conn.query_row("SELECT x, sumint_aux(y) OVER (
            ORDER BY x ROWS BETWEEN 1 PRECEDING AND 1 FOLLOWING
          ) AS sum_y
          FROM t3 ORDER BY x", (), |x| x.get(1)).unwrap();


        assert_eq!(result, 9);
    }
}
