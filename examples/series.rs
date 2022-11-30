//! cargo build --example series
//! sqlite3 :memory: '.read examples/test.sql'

use sqlite_loadable::prelude::*;
use sqlite_loadable::{
    api, define_table_function,
    table::{BestIndexError, ConstraintOperator, IndexInfo, VTab, VTabArguments, VTabCursor},
    Result,
};

use std::{mem, os::raw::c_int};

static CREATE_SQL: &str = "CREATE TABLE x(value, start hidden, stop hidden, step hidden)";
enum Columns {
    Value,
    Start,
    Stop,
    Step,
}
fn column(index: i32) -> Option<Columns> {
    match index {
        0 => Some(Columns::Value),
        1 => Some(Columns::Start),
        2 => Some(Columns::Stop),
        3 => Some(Columns::Step),
        _ => None,
    }
}

#[repr(C)]
pub struct GenerateSeriesTable {
    /// must be first
    base: sqlite3_vtab,
}

impl<'vtab> VTab<'vtab> for GenerateSeriesTable {
    type Aux = ();
    type Cursor = GenerateSeriesCursor;

    fn connect(
        _db: *mut sqlite3,
        _aux: Option<&Self::Aux>,
        _args: VTabArguments,
    ) -> Result<(String, GenerateSeriesTable)> {
        let base: sqlite3_vtab = unsafe { mem::zeroed() };
        let vtab = GenerateSeriesTable { base };
        // TODO db.config(VTabConfig::Innocuous)?;
        Ok((CREATE_SQL.to_owned(), vtab))
    }
    fn destroy(&self) -> Result<()> {
        Ok(())
    }

    fn best_index(&self, mut info: IndexInfo) -> core::result::Result<(), BestIndexError> {
        let mut has_start = false;
        let mut has_stop = false;
        for mut constraint in info.constraints() {
            match column(constraint.column_idx()) {
                Some(Columns::Start) => {
                    if constraint.usable() && constraint.op() == Some(ConstraintOperator::EQ) {
                        constraint.set_omit(true);
                        constraint.set_argv_index(1);
                        has_start = true;
                    } else {
                        return Err(BestIndexError::Constraint);
                    }
                }
                Some(Columns::Stop) => {
                    if constraint.usable() && constraint.op() == Some(ConstraintOperator::EQ) {
                        constraint.set_omit(true);
                        constraint.set_argv_index(2);
                        has_stop = true;
                    } else {
                        return Err(BestIndexError::Constraint);
                    }
                }
                _ => todo!(),
            }
        }
        if !has_start || !has_stop {
            return Err(BestIndexError::Error);
        }
        info.set_estimated_cost(100000.0);
        info.set_estimated_rows(100000);
        info.set_idxnum(1);

        Ok(())
    }

    fn open(&mut self) -> Result<GenerateSeriesCursor> {
        Ok(GenerateSeriesCursor::new())
    }
}

#[repr(C)]
pub struct GenerateSeriesCursor {
    /// Base class. Must be first
    base: sqlite3_vtab_cursor,
    rowid: i64,
    value: i64,
    min: i64,
    max: i64,
    step: i64,
}
impl GenerateSeriesCursor {
    fn new() -> GenerateSeriesCursor {
        let base: sqlite3_vtab_cursor = unsafe { mem::zeroed() };
        GenerateSeriesCursor {
            base,
            rowid: 0,
            value: 0,
            min: 0,
            max: 0,
            step: 0,
        }
    }
}

impl VTabCursor for GenerateSeriesCursor {
    fn filter(
        &mut self,
        _idx_num: c_int,
        _idx_str: Option<&str>,
        values: &[*mut sqlite3_value],
    ) -> Result<()> {
        //let pattern = values.get(0).unwrap().text()?;
        //let contents = values.get(1).unwrap().text()?;
        self.min = api::value_int64(values.get(0).expect("1st min constraint is required"));
        self.max = api::value_int64(values.get(1).expect("2nd max constraint is required"));
        self.value = self.min;
        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        self.value += 1;
        Ok(())
    }

    fn eof(&self) -> bool {
        self.value > self.max
    }

    fn column(&self, context: *mut sqlite3_context, i: c_int) -> Result<()> {
        match column(i) {
            Some(Columns::Value) => {
                api::result_int64(context, self.value);
            }
            Some(Columns::Start) => {
                api::result_int64(context, self.min);
            }
            Some(Columns::Stop) => {
                api::result_int64(context, self.max);
            }
            Some(Columns::Step) => {
                //context_result_int(0);
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
pub fn sqlite3_seriesrs_init(db: *mut sqlite3) -> Result<()> {
    define_table_function::<GenerateSeriesTable>(db, "generate_series_rs", None)?;
    Ok(())
}
