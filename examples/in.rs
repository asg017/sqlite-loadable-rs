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
}
impl InCursor {
    fn new() -> InCursor {
        let base: sqlite3_vtab_cursor = unsafe { mem::zeroed() };
        InCursor { base, rowid: 0 }
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
                    for v in InValues::new(values[idx]) {
                        let v = v.unwrap();
                        println!("{}", api::value_text(&v).unwrap());
                    }
                    /*let result = in_values(values[idx]).unwrap();
                    println!("Y handling {}", result.len());
                    for v in result {
                        println!("{}", api::value_text(&v).unwrap());
                    }*/
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
                api::result_int64(context, 1000 + self.rowid);
            }
            Some(Columns::B) => {
                api::result_int64(context, 2000 + self.rowid);
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
