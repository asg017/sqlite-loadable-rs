//! `sqlite_loadable::scan_config::register`: silent no-op without the host's
//! registry table, idempotent inserts with lowercased patterns when it exists.
#![cfg(feature = "exec")]

use sqlite_loadable::prelude::*;
use sqlite_loadable::{api, define_scalar_function, scan_config, Result};

/// Registers scan-config patterns the way an extension entrypoint would.
fn t_register_scans(context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
    let db = api::context_db_handle(context);
    scan_config::register(
        db,
        "memcsv",
        &["*.csv", "*.CSV.GZ"],
        "(flexible=true)",
        scan_config::Remote::RemoteOk,
    )?;
    api::result_text(context, "ok")?;
    Ok(())
}

#[sqlite_entrypoint]
pub fn sqlite3_scanconfigtest_init(db: *mut sqlite3) -> Result<()> {
    define_scalar_function(db, "t_register_scans", 0, t_register_scans, FunctionFlags::UTF8)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::{ffi::sqlite3_auto_extension, Connection};

    fn conn() -> Connection {
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite3_scanconfigtest_init as *const (),
            )));
        }
        Connection::open_in_memory().unwrap()
    }
    fn q<T: rusqlite::types::FromSql>(db: &Connection, sql: &str) -> T {
        db.query_row(sql, [], |r| r.get(0)).unwrap()
    }

    #[test]
    fn scan_config_register() {
        let db = conn();
        // no registry table: silent no-op
        assert_eq!(q::<String>(&db, "select t_register_scans()"), "ok");
        db.execute_batch(
            "create table temp._solite_replacement_scan_config (
               vtab_module  text not null,
               file_pattern text not null,
               module_args  text not null default '',
               remote       integer not null default 0,
               primary key (vtab_module, file_pattern) on conflict replace
             );",
        )
        .unwrap();
        assert_eq!(q::<String>(&db, "select t_register_scans()"), "ok");
        // registering twice is idempotent thanks to the conflict-replace key
        assert_eq!(q::<String>(&db, "select t_register_scans()"), "ok");
        let rows: Vec<(String, String, String, i64)> = db
            .prepare(
                "select vtab_module, file_pattern, module_args, remote \
                 from temp._solite_replacement_scan_config order by file_pattern",
            )
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        assert_eq!(
            rows,
            vec![
                // patterns are lowercased on the way in
                ("memcsv".into(), "*.csv".into(), "(flexible=true)".into(), 1),
                ("memcsv".into(), "*.csv.gz".into(), "(flexible=true)".into(), 1),
            ]
        );
    }
}
