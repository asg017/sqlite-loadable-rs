#[cfg(test)]
mod tests {
    use _iouringvfs::sqlite3_iouringvfs_init;
    use rusqlite::{ffi::sqlite3_auto_extension, Connection, self, OpenFlags};

    #[test]
    fn test_io_uring_ext() -> rusqlite::Result<()> {
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite3_iouringvfs_init as *const (),
            )));
        }

        let file_path = "test_iouring.db";

        let flags = OpenFlags::SQLITE_OPEN_URI | OpenFlags::SQLITE_OPEN_READ_WRITE;
        let conn = Connection::open_in_memory_with_flags(flags).unwrap();

        let stmt = format!("ATTACH DATABASE io_uring_vfs_from_file('{}') AS inring", file_path);
        let stmt_str = stmt.as_str();
        conn.execute(stmt_str, ())?;

        conn.execute("CREATE TABLE t3(x varchar(10), y integer)", ())?;
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

        let file_path = "test_iouring.wal.db";

        let flags = OpenFlags::SQLITE_OPEN_URI | OpenFlags::SQLITE_OPEN_READ_WRITE;
        let conn = Connection::open_in_memory_with_flags(flags).unwrap();
        
        let stmt = format!("ATTACH io_uring_vfs_from_file('{}') AS inring", file_path);
        let stmt_str = stmt.as_str();
        conn.execute(stmt_str, ())?;

        conn.execute("CREATE TABLE t3(x, y)", ())?;
        conn.execute("INSERT INTO t3 VALUES('a', 4),('b', 5),('c', 3),('d', 8),('e', 1)", ())?;

        let result: String = conn
            .query_row("select x from t3 where y = 1", (), |x| x.get(0))
            .unwrap();

        assert_eq!(result, "e");

        Ok(())
    }
}