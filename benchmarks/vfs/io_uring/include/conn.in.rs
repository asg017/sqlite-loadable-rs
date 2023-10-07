use _iouringvfs::sqlite3_iouringvfs_init;
use rusqlite::{ffi::sqlite3_auto_extension, Connection};

/// Tests were derived from: https://www.sqlite.org/speed.html
fn create_test_database(args: Vec<String>) -> rusqlite::Result<Connection> {
    assert!(args.len() <= 2);

    unsafe {
        sqlite3_auto_extension(Some(std::mem::transmute(
            sqlite3_iouringvfs_init as *const (),
        )));
    }

    let conn = if args.len() == 2 {
        let file_path = args[1].as_str();
        let conn = Connection::open(file_path).expect("Failed to create in-file database");
        if file_path.contains("ring") {
            let stmt = format!("ATTACH io_uring_vfs_from_file('{}') AS inring", file_path);
            let stmt_str = stmt.as_str();
            conn.execute(stmt_str, ()).expect("Failed to execute");
        }
        conn
    }else {
        Connection::open_in_memory().expect("Failed to create in-memory database")
    };

    conn.execute_batch(
        "CREATE TABLE t1(a integer, b varchar(100));
        CREATE TABLE t2(a integer, b integer, c varchar(100));
        CREATE TABLE t3(a integer, b integer, c varchar(100));
        CREATE INDEX i3 ON t3(c);
        CREATE TABLE t4(a integer, b integer, c varchar(100));
        CREATE TABLE t5(a integer, b integer, c varchar(100));
        CREATE TABLE t6(a integer, b integer);
        CREATE TABLE t7(a integer, b integer);
        CREATE INDEX i7 ON t7(b);
        CREATE TABLE t8(a integer, b integer);
        CREATE TABLE t9(a integer, b integer);
        CREATE TABLE t10(a integer, b integer, c varchar(100));
        CREATE INDEX i10 ON t10(a);"
    )?;
    
    Ok(conn)
}