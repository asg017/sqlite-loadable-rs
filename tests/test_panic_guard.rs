//! A panic inside a scalar function or a virtual-table callback becomes an
//! SQLite error instead of unwinding across the C boundary.

use sqlite_loadable::prelude::*;
use sqlite_loadable::table::{ConstraintOperator, IndexInfo, VTab, VTabArguments, VTabCursor};
use sqlite_loadable::{define_scalar_function, define_table_function, BestIndexError, Result};
use std::os::raw::c_int;

fn t_panic(_context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
    panic!("scalar boom");
}

#[repr(C)]
struct PanicTable {
    base: sqlite3_vtab,
}
impl<'vtab> VTab<'vtab> for PanicTable {
    type Aux = ();
    type Cursor = PanicCursor;
    fn connect(
        _db: *mut sqlite3,
        _aux: Option<&Self::Aux>,
        _args: VTabArguments,
    ) -> Result<(String, Self)> {
        Ok((
            "create table x(value)".to_owned(),
            PanicTable { base: unsafe { std::mem::zeroed() } },
        ))
    }
    fn best_index(&self, info: IndexInfo) -> core::result::Result<(), BestIndexError> {
        for mut c in info.constraints() {
            if c.usable() && c.op() == Some(ConstraintOperator::EQ) {
                c.set_omit(true);
                c.set_argv_index(1);
            }
        }
        Ok(())
    }
    fn open(&mut self) -> Result<Self::Cursor> {
        Ok(PanicCursor { base: unsafe { std::mem::zeroed() } })
    }
}
#[repr(C)]
struct PanicCursor {
    base: sqlite3_vtab_cursor,
}
impl VTabCursor for PanicCursor {
    fn filter(&mut self, _idx_num: c_int, _idx_str: Option<&str>, _args: &[*mut sqlite3_value]) -> Result<()> {
        panic!("filter boom");
    }
    fn next(&mut self) -> Result<()> {
        Ok(())
    }
    fn eof(&self) -> bool {
        true
    }
    fn column(&self, _context: *mut sqlite3_context, _i: c_int) -> Result<()> {
        Ok(())
    }
    fn rowid(&self) -> Result<i64> {
        Ok(0)
    }
}

#[sqlite_entrypoint]
pub fn sqlite3_panictest_init(db: *mut sqlite3) -> Result<()> {
    define_scalar_function(db, "t_panic", 0, t_panic, FunctionFlags::UTF8)?;
    define_table_function::<PanicTable>(db, "t_panic_table", None)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::{ffi::sqlite3_auto_extension, Connection};

    fn conn() -> Connection {
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite3_panictest_init as *const (),
            )));
        }
        Connection::open_in_memory().unwrap()
    }
    fn err(db: &Connection, sql: &str) -> String {
        db.query_row(sql, [], |r| r.get::<_, i64>(0)).unwrap_err().to_string()
    }

    #[test]
    fn scalar_panic_is_an_error() {
        let db = conn();
        let e = err(&db, "select t_panic()");
        assert!(e.contains("panic: scalar boom"), "{}", e);
        // the connection is still usable afterwards
        assert_eq!(db.query_row("select 1", [], |r| r.get::<_, i64>(0)).unwrap(), 1);
    }

    #[test]
    fn vtab_panic_is_an_error() {
        let db = conn();
        let e = err(&db, "select value from t_panic_table");
        assert!(e.contains("panic: filter boom"), "{}", e);
        assert_eq!(db.query_row("select 1", [], |r| r.get::<_, i64>(0)).unwrap(), 1);
    }
}
