//! Test for VTabWriteable INSERT operations, including explicit rowid
//!
//! This is a regression test for the bug where INSERT with explicit rowid
//! would panic because determine_update_operation incorrectly checked argv[1]
//! instead of argv[0] to detect INSERT operations.

use sqlite_loadable::prelude::*;
use sqlite_loadable::{
    api, define_virtual_table_writeable,
    table::{BestIndexError, IndexInfo, VTab, VTabArguments, VTabCursor, VTabWriteable, UpdateOperation},
    Result,
};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::{mem, os::raw::c_int};

static CREATE_SQL: &str = "CREATE TABLE x(value TEXT)";

/// Shared storage for the test virtual table
type Storage = Arc<Mutex<HashMap<i64, String>>>;

#[repr(C)]
pub struct TestWriteableTable {
    base: sqlite3_vtab,
    storage: Storage,
}

impl<'vtab> VTab<'vtab> for TestWriteableTable {
    type Aux = Storage;
    type Cursor = TestWriteableCursor;

    fn connect(
        _db: *mut sqlite3,
        aux: Option<&Self::Aux>,
        _args: VTabArguments,
    ) -> Result<(String, TestWriteableTable)> {
        let base: sqlite3_vtab = unsafe { mem::zeroed() };
        let storage = aux.cloned().unwrap_or_else(|| Arc::new(Mutex::new(HashMap::new())));
        let vtab = TestWriteableTable { base, storage };
        Ok((CREATE_SQL.to_owned(), vtab))
    }

    fn destroy(&self) -> Result<()> {
        Ok(())
    }

    fn best_index(&self, mut info: IndexInfo) -> core::result::Result<(), BestIndexError> {
        info.set_estimated_cost(100.0);
        info.set_estimated_rows(100);
        Ok(())
    }

    fn open(&mut self) -> Result<TestWriteableCursor> {
        let rows: Vec<(i64, String)> = self.storage.lock().unwrap()
            .iter()
            .map(|(k, v)| (*k, v.clone()))
            .collect();
        Ok(TestWriteableCursor::new(rows))
    }
}

impl<'vtab> VTabWriteable<'vtab> for TestWriteableTable {
    fn update(&'vtab mut self, operation: UpdateOperation, p_rowid: *mut i64) -> Result<()> {
        match operation {
            UpdateOperation::Insert { rowid, values } => {
                let mut storage = self.storage.lock().unwrap();

                // Get the rowid - either explicit or generate one
                let actual_rowid = if let Some(rowid_ptr) = rowid {
                    api::value_int64(rowid_ptr)
                } else {
                    // Generate a new rowid
                    storage.keys().max().map(|m| m + 1).unwrap_or(1)
                };

                // Get the value from the first column
                let value = if let Some(val_ptr) = values.first() {
                    api::value_text(val_ptr).unwrap_or_default().to_string()
                } else {
                    String::new()
                };

                storage.insert(actual_rowid, value);

                // Return the rowid to SQLite
                if !p_rowid.is_null() {
                    unsafe { *p_rowid = actual_rowid; }
                }
                Ok(())
            }
            UpdateOperation::Delete(rowid) => {
                let mut storage = self.storage.lock().unwrap();
                let rowid_val = api::value_int64(rowid);
                storage.remove(&rowid_val);
                Ok(())
            }
            UpdateOperation::Update { .. } => {
                // Not implemented for this test
                Ok(())
            }
        }
    }
}

#[repr(C)]
pub struct TestWriteableCursor {
    base: sqlite3_vtab_cursor,
    rows: Vec<(i64, String)>,
    index: usize,
}

impl TestWriteableCursor {
    fn new(rows: Vec<(i64, String)>) -> Self {
        let base: sqlite3_vtab_cursor = unsafe { mem::zeroed() };
        TestWriteableCursor { base, rows, index: 0 }
    }
}

impl VTabCursor for TestWriteableCursor {
    fn filter(
        &mut self,
        _idx_num: c_int,
        _idx_str: Option<&str>,
        _values: &[*mut sqlite3_value],
    ) -> Result<()> {
        self.index = 0;
        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        self.index += 1;
        Ok(())
    }

    fn eof(&self) -> bool {
        self.index >= self.rows.len()
    }

    fn column(&self, context: *mut sqlite3_context, i: c_int) -> Result<()> {
        if i == 0 {
            if let Some((_, value)) = self.rows.get(self.index) {
                api::result_text(context, value)?;
            }
        }
        Ok(())
    }

    fn rowid(&self) -> Result<i64> {
        Ok(self.rows.get(self.index).map(|(id, _)| *id).unwrap_or(0))
    }
}

#[sqlite_entrypoint]
pub fn sqlite3_testwriteable_init(db: *mut sqlite3) -> Result<()> {
    let storage: Storage = Arc::new(Mutex::new(HashMap::new()));
    define_virtual_table_writeable::<TestWriteableTable>(db, "test_writeable", Some(storage))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::{ffi::sqlite3_auto_extension, Connection};

    fn setup_connection() -> Connection {
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite3_testwriteable_init as *const (),
            )));
        }
        Connection::open_in_memory().unwrap()
    }

    #[test]
    fn test_insert_without_rowid() {
        let conn = setup_connection();

        conn.execute("CREATE VIRTUAL TABLE t USING test_writeable()", []).unwrap();
        conn.execute("INSERT INTO t(value) VALUES ('hello')", []).unwrap();

        let result: String = conn
            .query_row("SELECT value FROM t", [], |r| r.get(0))
            .unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_insert_with_explicit_rowid() {
        // This is the regression test for the bug where INSERT with explicit
        // rowid would panic because determine_update_operation incorrectly
        // checked argv[1] instead of argv[0] to detect INSERT operations.
        let conn = setup_connection();

        conn.execute("CREATE VIRTUAL TABLE t USING test_writeable()", []).unwrap();

        // INSERT with explicit rowid - this used to panic with todo!()
        conn.execute("INSERT INTO t(rowid, value) VALUES (42, 'explicit')", []).unwrap();

        let (rowid, value): (i64, String) = conn
            .query_row("SELECT rowid, value FROM t", [], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap();

        assert_eq!(rowid, 42);
        assert_eq!(value, "explicit");
    }

    #[test]
    fn test_insert_multiple_with_explicit_rowids() {
        let conn = setup_connection();

        conn.execute("CREATE VIRTUAL TABLE t USING test_writeable()", []).unwrap();
        conn.execute("INSERT INTO t(rowid, value) VALUES (10, 'ten')", []).unwrap();
        conn.execute("INSERT INTO t(rowid, value) VALUES (20, 'twenty')", []).unwrap();
        conn.execute("INSERT INTO t(rowid, value) VALUES (30, 'thirty')", []).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM t", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_delete() {
        let conn = setup_connection();

        conn.execute("CREATE VIRTUAL TABLE t USING test_writeable()", []).unwrap();
        conn.execute("INSERT INTO t(rowid, value) VALUES (1, 'one')", []).unwrap();
        conn.execute("INSERT INTO t(rowid, value) VALUES (2, 'two')", []).unwrap();

        conn.execute("DELETE FROM t WHERE rowid = 1", []).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM t", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }
}