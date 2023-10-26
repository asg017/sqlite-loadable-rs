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
        let conn = Connection::open(file_path)?;
        if file_path.contains("ring") {
            let stmt = format!("ATTACH io_uring_vfs_from_file('{}') AS inring", file_path);
            let stmt_str = stmt.as_str();
            conn.execute(stmt_str, ())?;
        }
        conn
    }else {
        Connection::open_in_memory()?
    };

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS t1(a integer, b varchar(100));
        CREATE TABLE IF NOT EXISTS t2(a integer, b integer, c varchar(100));
        CREATE TABLE IF NOT EXISTS t3(a integer, b integer, c varchar(100));
        CREATE INDEX IF NOT EXISTS i3 ON t3(c);
        CREATE TABLE IF NOT EXISTS t4(a integer, b integer, c varchar(100));
        CREATE TABLE IF NOT EXISTS t5(a integer, b integer, c varchar(100));
        CREATE TABLE IF NOT EXISTS t6(a integer, b integer);
        CREATE TABLE IF NOT EXISTS t7(a integer, b integer);
        CREATE INDEX IF NOT EXISTS i7 ON t7(b);
        CREATE TABLE IF NOT EXISTS t8(a integer, b integer);
        CREATE TABLE IF NOT EXISTS t9(a integer, b integer);
        CREATE TABLE IF NOT EXISTS t10(a integer, b integer, c varchar(100));
        CREATE INDEX IF NOT EXISTS i10 ON t10(a);"
    )?;
    
    Ok(conn)
}