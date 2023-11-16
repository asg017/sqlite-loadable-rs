include!("../include/conn.in.rs");

#[cfg(test)]
mod tests {
    use _iouringvfs::sqlite3_iouringvfs_init;
    use rusqlite::{self, ffi::sqlite3_auto_extension};

    use crate::open_io_uring_connection;

    #[test]
    fn test_io_uring_ext() -> rusqlite::Result<()> {
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite3_iouringvfs_init as *const (),
            )));
        }

        // let tmp_file = tempfile::NamedTempFile::new().unwrap();
        // let out_path = tmp_file.path().to_str().unwrap();
        let out_path = "main.db";

        let conn = open_io_uring_connection(out_path)?;

        conn.execute("CREATE TABLE t3(x varchar(10), y integer)", ())?;
        conn.execute(
            "INSERT INTO t3 VALUES('a', 4),('b', 5),('c', 3),('d', 8),('e', 1)",
            (),
        )?;

        let result: String = conn
            .query_row("select x from t3 where y = 4", (), |x| x.get(0))?;

        assert_eq!(result, "a");

        Ok(())
    }
}
 