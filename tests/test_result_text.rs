//! api::result_text passes a borrowed buffer with SQLITE_TRANSIENT, so SQLite must copy it
//! before the call returns. These cover the edge cases of that: empty strings, embedded NULs,
//! a buffer freed right after the call, and result buffers reused across rows of varying size.
use sqlite_loadable::prelude::*;
use sqlite_loadable::{api, define_scalar_function, Result};

pub fn empty(context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
    api::result_text(context, "")?;
    Ok(())
}

pub fn with_nul(context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
    api::result_text(context, "a\0b")?;
    Ok(())
}

/// Returns `'x'` repeated N times from a String that is dropped (and then overwritten by a
/// fresh allocation) right after result_text returns.
pub fn repeat_x(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let n = api::value_int64(values.first().expect("1st argument as n")) as usize;
    let s = "x".repeat(n);
    api::result_text(context, &s)?;
    drop(s);
    let clobber = "y".repeat(n);
    std::hint::black_box(clobber);
    Ok(())
}

/// Sets the result twice; the second value must win.
pub fn twice(context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
    api::result_text(context, String::from("first, a longer value that needs a bigger buffer"))?;
    api::result_text(context, String::from("second"))?;
    Ok(())
}

#[sqlite_entrypoint]
pub fn sqlite3_resulttext_init(db: *mut sqlite3) -> Result<()> {
    let flags = FunctionFlags::UTF8 | FunctionFlags::DETERMINISTIC;
    define_scalar_function(db, "empty", 0, empty, flags)?;
    define_scalar_function(db, "with_nul", 0, with_nul, flags)?;
    define_scalar_function(db, "repeat_x", 1, repeat_x, flags)?;
    define_scalar_function(db, "twice", 0, twice, flags)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use rusqlite::{ffi::sqlite3_auto_extension, Connection};

    fn conn() -> Connection {
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite3_resulttext_init as *const (),
            )));
        }
        Connection::open_in_memory().unwrap()
    }

    #[test]
    fn empty_string_is_text_not_null() {
        let conn = conn();
        let (value, typ): (String, String) = conn
            .query_row("select empty(), typeof(empty())", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .unwrap();
        assert_eq!(value, "");
        assert_eq!(typ, "text");
    }

    #[test]
    fn embedded_nul_is_kept() {
        let conn = conn();
        let (hex, octets): (String, i64) = conn
            .query_row(
                "select hex(with_nul()), length(cast(with_nul() as blob))",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(hex, "610062");
        assert_eq!(octets, 3);
    }

    #[test]
    fn varying_lengths_across_rows() {
        // Lengths go up and down around SQLite's 32-byte minimum Mem allocation, so the
        // result buffer is both reused and grown from row to row.
        let conn = conn();
        let mut stmt = conn
            .prepare(
                "with recursive r(i) as (select 0 union all select i + 1 from r where i < 200)
                 select i, repeat_x(case when i % 2 then i * 37 % 5000 else i % 40 end) from r",
            )
            .unwrap();
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
            .unwrap();
        let mut count = 0;
        for row in rows {
            let (i, s) = row.unwrap();
            let expected = if i % 2 == 1 { i * 37 % 5000 } else { i % 40 } as usize;
            assert_eq!(s, "x".repeat(expected), "row {i}");
            count += 1;
        }
        assert_eq!(count, 201);
    }

    #[test]
    fn large_string() {
        let conn = conn();
        let n = 10 * 1024 * 1024;
        let len: i64 = conn
            .query_row(
                "select length(repeat_x(?)) where repeat_x(?) = ?",
                rusqlite::params![n, n, "x".repeat(n)],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(len, n as i64);
    }

    #[test]
    fn last_result_wins() {
        let conn = conn();
        let s: String = conn.query_row("select twice()", [], |r| r.get(0)).unwrap();
        assert_eq!(s, "second");
    }
}
