//! Replacement-scan hints: tell a hosting shell which files this extension
//! can read.
//!
//! Hosts like solite create a per-connection temp table *before* extensions
//! are loaded:
//!
//! ```sql
//! create table temp._solite_replacement_scan_config (
//!   vtab_module  text not null,              -- bare module name
//!   file_pattern text not null,              -- lowercase GLOB, e.g. '*.csv'
//!   module_args  text not null default '',   -- appended to the USING clause
//!   remote       integer not null default 0, -- 1 = module accepts URLs
//!   primary key (vtab_module, file_pattern) on conflict replace
//! );
//! ```
//!
//! When a query then references an unknown table whose name matches a
//! registered pattern (`select * from "data/report.xlsx"`), the host creates
//! `create virtual table temp."<name>" using <vtab_module><module_args>` and
//! retries. An extension declares its patterns by calling [`register`] from
//! its entrypoint; under a host that didn't create the table this is a
//! silent no-op, so the same compiled extension works everywhere.
//!
//! C extensions implement the same convention with a single ignored-error
//! `sqlite3_exec` of the INSERT (see sqlite-source.h's sibling docs):
//!
//! ```c
//! /* replacement-scan hints for hosts like solite; no-op elsewhere */
//! sqlite3_exec(db,
//!   "insert or replace into temp._solite_replacement_scan_config"
//!   "(vtab_module, file_pattern, module_args, remote)"
//!   " values ('xml0','*.xml','',0),('xml0','*.rss','',0)", 0, 0, 0);
//! ```

use crate::exec::Statement;
use crate::ext::sqlite3;
use crate::{Error, Result};

/// The registry's table name; the host owns creation, extensions only insert.
pub const SCAN_CONFIG_TABLE: &str = "temp._solite_replacement_scan_config";

/// Whether the module can be pointed at remote URLs (source-API consumers)
/// or only at local files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Remote {
    LocalOnly,
    RemoteOk,
}

/// Insert replacement-scan patterns for `module`, all sharing `module_args`
/// (`""` for none, otherwise a parenthesized suffix like `"(flexible=true)"`).
/// Silent no-op when the host didn't create the registry table. Call from the
/// extension entrypoint, after the module is registered.
pub fn register(
    db: *mut sqlite3,
    module: &str,
    patterns: &[&str],
    module_args: &str,
    remote: Remote,
) -> Result<()> {
    if !module
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        || module.is_empty()
    {
        return Err(Error::new_message(format!(
            "invalid vtab module name: {:?}",
            module
        )));
    }
    // Preparing the INSERT doubles as the probe: it fails with
    // "no such table" on hosts without the registry.
    let sql = format!(
        "insert or replace into {} (vtab_module, file_pattern, module_args, remote) \
         values (?1, ?2, ?3, ?4)",
        SCAN_CONFIG_TABLE
    );
    let remote = matches!(remote, Remote::RemoteOk) as i32;
    for pattern in patterns {
        let mut stmt = match Statement::prepare(db, &sql) {
            Ok(stmt) => stmt,
            Err(_) => return Ok(()), // host has no registry table
        };
        stmt.bind_text(1, module)
            .and_then(|_| stmt.bind_text(2, &pattern.to_ascii_lowercase()))
            .and_then(|_| stmt.bind_text(3, module_args))
            .and_then(|_| stmt.bind_i32(4, remote))
            .map_err(|e| Error::new_message(e.to_string()))?;
        let rc = unsafe { crate::ext::sqlite3ext_step(stmt.as_ptr()) };
        if rc != crate::constants::SQLITE_DONE {
            return Err(Error::new_message(format!(
                "registering replacement-scan pattern {:?} failed (rc={})",
                pattern, rc
            )));
        }
    }
    Ok(())
}
