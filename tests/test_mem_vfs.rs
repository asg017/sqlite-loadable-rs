#![allow(unused)]

include!("../examples/mem_vfs.in.rs");

#[cfg(test)]
mod tests {
    use super::*;

    use rusqlite::{ffi::sqlite3_auto_extension, Connection, OpenFlags};

    #[test]
    fn test_rusqlite_auto_extension() {
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite3_memvfs_init as *const (),
            )));
        }

        let flags = OpenFlags::SQLITE_OPEN_URI | OpenFlags::SQLITE_OPEN_READ_WRITE;

        let conn = Connection::open_in_memory_with_flags(flags).unwrap();

        conn.execute("ATTACH DATABASE memvfs_from_file('dummy.db') AS inmem;", ());

        conn.execute("CREATE TABLE t3(x, y)", ());
        conn.execute("INSERT INTO t3 VALUES('a', 4),('b', 5),('c', 3),('d', 8),('e', 1)", ());

        let result: String = conn
        .query_row("select x from t3 where y = 4", (), |x| x.get(0))
        .unwrap();

        assert_eq!(result, "a");
    }
}