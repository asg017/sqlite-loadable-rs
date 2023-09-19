use sqlite_loadable::prelude::*;
use sqlite_loadable::{define_collation, Result};
use std::cmp::Ordering;

fn compare(a: &[u8], b: &[u8]) -> i32 {
    let a: Vec<u8> = a.iter().rev().cloned().collect();
    let b: Vec<u8> = b.iter().rev().cloned().collect();
    match a.cmp(&b) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}
#[sqlite_entrypoint]
pub fn sqlite3_test_collation_init(db: *mut sqlite3) -> Result<()> {
    define_collation(db, "test_collation", compare)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use rusqlite::{ffi::sqlite3_auto_extension, Connection};

    #[test]
    fn test_rusqlite_auto_extension() {
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite3_test_collation_init as *const (),
            )));
        }

        let conn = Connection::open_in_memory().unwrap();

        let result: String = conn
            .query_row(
                "
            with ordered as (
              select value
              from json_each(?)
              order by 1 collate test_collation
            )
            select json_group_array(value) from ordered
            ",
                [r#"[
                  "xxxc", "yyyb", "zzza"
                ]"#],
                |x| x.get(0),
            )
            .unwrap();

        assert_eq!(result, "[\"zzza\",\"yyyb\",\"xxxc\"]");
    }
}
