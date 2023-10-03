#[cfg(test)]
mod tests {
    use _iouringvfs::sqlite3_iouringvfs_init;
    use rusqlite::{ffi::sqlite3_auto_extension, Connection};

    #[test]
    fn test_rusqlite_auto_extension() {
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite3_iouringvfs_init as *const (),
            )));
        }

        let conn = Connection::open_in_memory().unwrap();

        let _ = conn.execute("ATTACH io_uring_vfs_from_file('from.db') AS inring;", ());

        let _ = conn.execute("CREATE TABLE t3(x, y)", ());
        let _ = conn.execute("INSERT INTO t3 VALUES('a', 4),('b', 5),('c', 3),('d', 8),('e', 1)", ());

        let result: String = conn
        .query_row("select x from t3 where y = 4", (), |x| x.get(0))
        .unwrap();

        assert_eq!(result, "a");
    }
}