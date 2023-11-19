#![allow(unused)]

include!("../include/mem_vfs.in.rs");

#[derive(Debug)]
struct Person {
    id: i32,
    name: String,
    data: Option<Vec<u8>>,
}

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
            "db/100-bytes.db",
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
            EXTENSION_NAME,
        )?;

        conn.execute(
            "CREATE TABLE person (
                id    INTEGER PRIMARY KEY,
                name  TEXT NOT NULL,
                data  BLOB
            )",
            (), // empty list of parameters.
        )?;
        let me = Person {
            id: 0,
            name: "Batman".to_string(),
            data: None,
        };
        conn.execute(
            "INSERT INTO person (name, data) VALUES (?1, ?2)",
            (&me.name, &me.data),
        )?;

        let mut stmt = conn.prepare("SELECT id, name, data FROM person")?;
        let person_iter = stmt.query_map([], |row| {
            Ok(Person {
                id: row.get(0)?,
                name: row.get(1)?,
                data: row.get(2)?,
            })
        })?;

        for person in person_iter {
            println!("Found person {:?}", person.unwrap());
        }

        Ok(())
    }
}
