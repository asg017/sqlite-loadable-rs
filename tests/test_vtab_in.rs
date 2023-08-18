//! cargo build --example series
//! sqlite3 :memory: '.read examples/test.sql'

use sqlite_loadable::prelude::*;
use sqlite_loadable::table::InValues;
use sqlite_loadable::{
    api, define_table_function,
    table::{BestIndexError, ConstraintOperator, IndexInfo, VTab, VTabArguments, VTabCursor},
    Result,
};

use std::{mem, os::raw::c_int};

static CREATE_SQL: &str = "CREATE TABLE x(a, b, x hidden, y hidden)";
enum Columns {
    A,
    B,
    X,
    Y,
}
fn column(index: i32) -> Option<Columns> {
    match index {
        0 => Some(Columns::A),
        1 => Some(Columns::B),
        2 => Some(Columns::X),
        3 => Some(Columns::Y),
        _ => None,
    }
}

#[repr(C)]
pub struct InTable {
    /// must be first
    base: sqlite3_vtab,
}

impl<'vtab> VTab<'vtab> for InTable {
    type Aux = ();
    type Cursor = InCursor;

    fn connect(
        _db: *mut sqlite3,
        _aux: Option<&Self::Aux>,
        _args: VTabArguments,
    ) -> Result<(String, InTable)> {
        let base: sqlite3_vtab = unsafe { mem::zeroed() };
        let vtab = InTable { base };
        // TODO db.config(VTabConfig::Innocuous)?;
        Ok((CREATE_SQL.to_owned(), vtab))
    }
    fn destroy(&self) -> Result<()> {
        Ok(())
    }

    fn best_index(&self, mut info: IndexInfo) -> core::result::Result<(), BestIndexError> {
        let mut idx_str = String::new();
        let mut argv_index = 1;
        for mut constraint in info.constraints() {
            if !constraint.usable() {
                continue;
            }
            match column(constraint.column_idx()) {
                Some(Columns::X) => {
                    if constraint.op() == Some(ConstraintOperator::EQ) {
                        constraint.set_omit(true);
                        constraint.set_argv_index(argv_index);
                        argv_index += 1;
                        idx_str.push('x');
                    } else {
                        return Err(BestIndexError::Constraint);
                    }
                }
                Some(Columns::Y) => {
                    if constraint.op() == Some(ConstraintOperator::EQ) {
                        constraint.set_omit(true);
                        constraint.set_argv_index(argv_index);
                        argv_index += 1;
                        if constraint.can_process_all_in() {
                            idx_str.push('Y');
                            constraint.enable_process_all_in();
                        } else {
                            idx_str.push('y');
                        }
                    } else {
                        return Err(BestIndexError::Constraint);
                    }
                }
                _ => todo!(),
            }
        }
        info.set_estimated_cost(100000.0);
        info.set_estimated_rows(100000);
        info.set_idxnum(1);
        info.set_idxstr(idx_str.as_str()).unwrap();

        Ok(())
    }

    fn open(&mut self) -> Result<InCursor> {
        Ok(InCursor::new())
    }
}

#[repr(C)]
pub struct InCursor {
    /// Base class. Must be first
    base: sqlite3_vtab_cursor,
    rowid: i64,
    value: Option<String>,
}
impl InCursor {
    fn new() -> InCursor {
        let base: sqlite3_vtab_cursor = unsafe { mem::zeroed() };
        InCursor {
            base,
            rowid: 0,
            value: None,
        }
    }
}

impl VTabCursor for InCursor {
    fn filter(
        &mut self,
        _idx_num: c_int,
        idx_str: Option<&str>,
        values: &[*mut sqlite3_value],
    ) -> Result<()> {
        let idx_str = idx_str.unwrap();
        for (idx, constraint) in idx_str.chars().enumerate() {
            match constraint {
                'Y' => {
                    let mut value = String::new();
                    for v in InValues::new(values[idx]) {
                        let v = v.unwrap();
                        value.push_str(api::value_text(&v).unwrap());
                    }
                    self.value = Some(value)
                }
                'y' => (),
                _ => (),
            }
        }
        self.rowid = 1;
        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        self.rowid += 1;
        Ok(())
    }

    fn eof(&self) -> bool {
        self.rowid >= 10
    }

    fn column(&self, context: *mut sqlite3_context, i: c_int) -> Result<()> {
        match column(i) {
            Some(Columns::A) => {
                if let Some(value) = &self.value {
                    api::result_text(context, value)?;
                } else {
                    api::result_text(context, "")?;
                }
            }
            _ => (),
        }
        Ok(())
    }

    fn rowid(&self) -> Result<i64> {
        Ok(self.rowid)
    }
}

#[sqlite_entrypoint]
pub fn sqlite3_in_init(db: *mut sqlite3) -> Result<()> {
    define_table_function::<InTable>(db, "vtab_in", None)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use rusqlite::{ffi::sqlite3_auto_extension, Connection};

    fn query_plan(db: &Connection, sql: &str) -> String {
        db.query_row(format!("explain query plan {}", sql).as_str(), [], |row| {
            row.get("detail")
        })
        .unwrap()
    }

    #[test]
    fn test_rusqlite_auto_extension() {
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_in_init as *const ())));
        }

        let db = Connection::open_in_memory().unwrap();

        assert_eq!(
            query_plan(&db, "select * from vtab_in"),
            "SCAN vtab_in VIRTUAL TABLE INDEX 1:"
        );
        assert_eq!(
            query_plan(&db, "select * from vtab_in(1)"),
            "SCAN vtab_in VIRTUAL TABLE INDEX 1:x"
        );
        assert_eq!(
            query_plan(&db, "select * from vtab_in where y in (1,2,3)"),
            "SCAN vtab_in VIRTUAL TABLE INDEX 1:Y"
        );
        assert_eq!(
            query_plan(&db, "select * from vtab_in where y = 1"),
            "SCAN vtab_in VIRTUAL TABLE INDEX 1:y"
        );
        // TODO test when sqlite version is 3.37 or less

        let a: String = db
            .query_row("select a from vtab_in where y = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(a, "");

        let a: String = db
            .query_row("select a from vtab_in where y in (1,2,3)", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(a, "123");
    }
}
