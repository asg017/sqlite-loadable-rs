//! A tiny *consumer* of the source API, for testing producers from SQL:
//!
//!   select source_head('_http_api', 'https://…');            -- JSON meta
//!   select source_get('_http_api', 'https://…');             -- body blob
//!   select source_range('_http_api', 'https://…', 10, 5);    -- range blob
//!   select source_get_len('_http_api', 'https://…');         -- streamed byte count
//!
//! Build: cargo build --example source_consumer --features source
use sqlite_loadable::prelude::*;
use sqlite_loadable::source::{resolve_source_api, SourceHandle};
use sqlite_loadable::{api, define_scalar_function, Result};
use std::io::Read;

fn resolve(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<SourceHandle> {
    let function = api::value_text(&values[0])?;
    Ok(resolve_source_api(api::context_db_handle(context), function)?)
}

fn source_head(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let meta = resolve(context, values)?.head(api::value_text(&values[1])?)?;
    let json = serde_json::json!({
        "size": meta.size, "last_modified_ms": meta.last_modified_ms,
        "etag": meta.etag, "content_type": meta.content_type,
    });
    // result_text rather than result_json: the latter sets a subtype, which
    // recent SQLite CLIs reject unless the function declares RESULT_SUBTYPE.
    api::result_text(context, json.to_string())?;
    Ok(())
}
fn source_get(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let mut stream = resolve(context, values)?.get(api::value_text(&values[1])?)?;
    let mut out = Vec::new();
    stream
        .read_to_end(&mut out)
        .map_err(|e| sqlite_loadable::Error::new_message(e.to_string()))?;
    api::result_blob(context, &out);
    Ok(())
}
/// Streams the body in 64 KiB chunks and returns only the byte count — proves
/// large bodies stream without being materialized by the consumer.
fn source_get_len(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let mut stream = resolve(context, values)?.get(api::value_text(&values[1])?)?;
    let mut buf = vec![0u8; 64 * 1024];
    let mut total: i64 = 0;
    loop {
        let n = stream
            .read(&mut buf)
            .map_err(|e| sqlite_loadable::Error::new_message(e.to_string()))?;
        if n == 0 {
            break;
        }
        total += n as i64;
    }
    api::result_int64(context, total);
    Ok(())
}
fn source_range(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let bytes = resolve(context, values)?.get_range(
        api::value_text(&values[1])?,
        api::value_int64(&values[2]) as u64,
        api::value_int64(&values[3]) as u64,
    )?;
    api::result_blob(context, &bytes);
    Ok(())
}

#[sqlite_entrypoint]
pub fn sqlite3_sourceconsumer_init(db: *mut sqlite3) -> Result<()> {
    let flags = FunctionFlags::UTF8;
    define_scalar_function(db, "source_head", 2, source_head, flags)?;
    define_scalar_function(db, "source_get", 2, source_get, flags)?;
    define_scalar_function(db, "source_get_len", 2, source_get_len, flags)?;
    define_scalar_function(db, "source_range", 4, source_range, flags)?;
    Ok(())
}
