use _iouringvfs::sqlite3_iouringvfs_init;
use rusqlite::{ffi::sqlite3_auto_extension, Connection};

pub const IOURING_DB_ALIAS: &str = "ring";

fn open_io_uring_connection(db: &str) -> rusqlite::Result<Connection> {
    use rusqlite::OpenFlags;
    use _iouringvfs::EXTENSION_NAME;

    let conn = Connection::open_with_flags_and_vfs(
        db,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE,
        EXTENSION_NAME,
    )?;

    Ok(conn)
}

#[allow(dead_code)]
/// Tests were derived from: https://www.sqlite.org/speed.html
fn create_test_database(args: Vec<String>) -> rusqlite::Result<Connection> {
    assert!(args.len() <= 2);

    unsafe {
        sqlite3_auto_extension(Some(std::mem::transmute(
            sqlite3_iouringvfs_init as *const (),
        )));
    }

    // Necessary to load the custom vfs
    let _conn = Connection::open_in_memory()?;
    _conn.close().expect("error occurred while closing");

    let conn = if args.len() == 2 {
        let file_path = args[1].as_str();

        let conn = if file_path.contains("ring") {
            open_io_uring_connection(file_path)?
        }else {
            Connection::open(file_path)?
        };

        if file_path.contains("wal") {
            conn.execute_batch("PRAGMA journal_mode = WAL")?;
        }

        conn    
    }else {
        Connection::open_in_memory()?
    };

    conn.execute_batch(
        r"CREATE TABLE IF NOT EXISTS t1(a integer, b varchar(100));
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