#![allow(unused)]

include!("../include/mem_vfs.in.rs");

#[cfg(test)]
mod tests {
    use super::*;

    use rusqlite::{ffi::sqlite3_auto_extension, Connection, OpenFlags, Result};

    #[test]
    fn test_rusqlite_auto_extension() -> Result<()> {
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_memvfs_init as *const ())));
        }

        // let conn = Connection::open_in_memory()?;
        // conn.execute("ATTACH DATABASE mem_vfs_uri() AS inmem;", ());

        // open in memory first to run faux_sqlite_extension_init2,
        // to register the new vfs, in the auto extension callback
        // in the following open_with_flags_and_vfs connection
        // this workaround would be unnecessary if a Option<sqlite3_api_routines> would
        // be passed down to the callback function besides the sqlite3 ptr
        // also large parts of ext.rs would be unnecessary
        // if the end user could decide whether to use rusqlite or libsqlite3
        // by placing a conditional cfg there instead
        // in any case, this should be documented properly -- TODO
        let _conn = Connection::open_in_memory()?;

        _conn.close();

        let conn = Connection::open_with_flags_and_vfs(
            "db/not_really_opened.db",
                OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE,
            // | OpenFlags::SQLITE_OPEN_MEMORY, // skips File creation altogether
            // | OpenFlags::SQLITE_OPEN_EXCLUSIVE, // it is a no-op, used internally
            EXTENSION_NAME,
        )?;

        conn.execute_batch(r#"
            PRAGMA locking_mode = EXCLUSIVE;
            PRAGMA journal_mode = TRUNCATE;"#)?;

        conn.execute("CREATE TABLE t3(x, y)", ())?;

        conn.execute(
            "INSERT INTO t3 VALUES('a', 4),('b', 5),('c', 3),('d', 8),('e', 1)",
            (),
        )?;

        let result: String = conn
            .query_row("SELECT x FROM t3 WHERE y = 4", (), |x| x.get(0))?;

        assert_eq!(result, "a");

        Ok(())
    }
}
