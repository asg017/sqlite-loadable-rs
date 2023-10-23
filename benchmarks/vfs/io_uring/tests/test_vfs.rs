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

        let tmp_file = tempfile::NamedTempFile::new().unwrap();
        let out_path = tmp_file.path().to_str().unwrap();

        let flags = OpenFlags::SQLITE_OPEN_URI | OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE;
        let conn = Connection::open_with_flags_and_vfs(out_path, flags, "unix")?;

        let stmt = format!("ATTACH DATABASE io_uring_vfs_from_file('{}') AS inring", out_path);
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

        let tmp_file = tempfile::NamedTempFile::new().unwrap();
        let out_path = tmp_file.path().to_str().unwrap();

        let flags = OpenFlags::SQLITE_OPEN_URI | OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE;
        let conn = Connection::open_with_flags_and_vfs(out_path, flags, "unix")?;
        
        let stmt = format!("ATTACH io_uring_vfs_from_file('{}') AS inring", out_path);
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