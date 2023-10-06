#[cfg(test)]
mod tests {
    use _iouringvfs::sqlite3_iouringvfs_init;
    use rusqlite::{ffi::sqlite3_auto_extension, Connection, self};

    #[test]
    fn test_io_uring_ext() -> rusqlite::Result<()> {
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite3_iouringvfs_init as *const (),
            )));
        }

        let conn = Connection::open_in_memory()?;

        conn.execute("ATTACH io_uring_vfs_from_file('from.db') AS inring;", ())?;

        conn.execute("CREATE TABLE t3(x, y)", ())?;
        conn.execute("INSERT INTO t3 VALUES('a', 4),('b', 5),('c', 3),('d', 8),('e', 1)", ())?;

        let result: String = conn
            .query_row("select x from t3 where y = 4", (), |x| x.get(0))
            .unwrap();

        assert_eq!(result, "a");

        Ok(())
    }

    #[test]
    fn test_io_uring_ext_with_wal() -> rusqlite::Result<()> {
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite3_iouringvfs_init as *const (),
            )));
        }

        let conn = Connection::open_in_memory()?;

        conn.execute("ATTACH io_uring_vfs_from_file('from.db') AS inring;", ())?;

        conn.pragma_update(None, "journal_mode", "wal")?;

        conn.execute("CREATE TABLE t3(x, y)", ())?;
        conn.execute("INSERT INTO t3 VALUES('a', 4),('b', 5),('c', 3),('d', 8),('e', 1)", ())?;

        let result: String = conn
            .query_row("select x from t3 where y = 1", (), |x| x.get(0))
            .unwrap();

        assert_eq!(result, "e");

        Ok(())
    }
}