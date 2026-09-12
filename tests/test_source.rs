//! End-to-end test of `sqlite_loadable::source`: an in-memory producer and a
//! file producer, plus SQL-callable consumer functions that resolve them
//! through `resolve_source_api`, all loaded into one rusqlite connection.
#![cfg(feature = "source")]

use sqlite_loadable::prelude::*;
use sqlite_loadable::source::{
    self, resolve_source_api, RemoteGlob, SourceApi, SourceEntry, SourceError, SourceMeta,
    SourceResult,
};
use sqlite_loadable::{api, define_scalar_function, Result};
use std::collections::HashMap;
use std::io::Read;
use std::sync::atomic::{AtomicUsize, Ordering};

static MEM_API_DROPS: AtomicUsize = AtomicUsize::new(0);
static MEM_API_HEADS: AtomicUsize = AtomicUsize::new(0);

struct MemApi {
    objects: HashMap<String, Vec<u8>>,
}
impl Drop for MemApi {
    fn drop(&mut self) {
        MEM_API_DROPS.fetch_add(1, Ordering::SeqCst);
    }
}
impl MemApi {
    fn object(&self, url: &str) -> SourceResult<&Vec<u8>> {
        self.objects
            .get(url)
            .ok_or_else(|| SourceError::Message(format!("mem: {} not found", url)))
    }
    fn etag(o: &[u8]) -> String {
        format!("\"etag-{}\"", o.len())
    }
    fn check(&self, url: &str, if_match: Option<&str>) -> SourceResult<&Vec<u8>> {
        let o = self.object(url)?;
        match if_match {
            Some(expected) if expected != Self::etag(o) => Err(SourceError::Changed {
                url: url.to_owned(),
                expected: expected.to_owned(),
            }),
            _ => Ok(o),
        }
    }
}
impl SourceApi for MemApi {
    fn head(&self, url: &str) -> SourceResult<SourceMeta> {
        MEM_API_HEADS.fetch_add(1, Ordering::SeqCst);
        let o = self.object(url)?;
        Ok(SourceMeta {
            size: Some(o.len() as u64),
            last_modified_ms: Some(1_700_000_000_000),
            etag: Some(Self::etag(o)),
            content_type: Some("text/plain".into()),
        })
    }
    fn get(&self, url: &str, if_match: Option<&str>) -> SourceResult<Box<dyn Read + Send>> {
        if url == "mem://panic" {
            panic!("boom");
        }
        Ok(Box::new(std::io::Cursor::new(self.check(url, if_match)?.clone())))
    }
    fn get_range(&self, url: &str, start: u64, len: u64, if_match: Option<&str>) -> SourceResult<Vec<u8>> {
        let o = self.check(url, if_match)?;
        let start = start as usize;
        let end = (start + len as usize).min(o.len());
        if start > o.len() {
            return Err("range start past end".into());
        }
        Ok(o[start..end].to_vec())
    }
    fn supports_list(&self) -> bool {
        true
    }
    fn list(&self, prefix: &str) -> SourceResult<Vec<SourceEntry>> {
        if prefix == "mem://boom/" {
            return Err("mem: list exploded".into());
        }
        let mut urls: Vec<&String> = self.objects.keys().filter(|k| k.starts_with(prefix)).collect();
        // deliberately not sorted: consumers sort
        urls.sort_by(|a, b| b.cmp(a));
        Ok(urls
            .into_iter()
            .map(|url| {
                let o = &self.objects[url];
                SourceEntry {
                    url: url.clone(),
                    size: Some(o.len() as u64),
                    // objects under mem://nometa/ come back without etag/size
                    etag: (!url.starts_with("mem://nometa/")).then(|| Self::etag(o)),
                    last_modified_ms: Some(1_700_000_000_000),
                }
            })
            .map(|mut e| {
                if e.url.starts_with("mem://nometa/") {
                    e.size = None;
                }
                e
            })
            .collect())
    }
}

struct FileApi;
impl SourceApi for FileApi {
    fn head(&self, url: &str) -> SourceResult<SourceMeta> {
        let md = std::fs::metadata(url.trim_start_matches("file://"))?;
        Ok(SourceMeta { size: Some(md.len()), ..Default::default() })
    }
    fn get(&self, url: &str, _if_match: Option<&str>) -> SourceResult<Box<dyn Read + Send>> {
        Ok(Box::new(std::fs::File::open(url.trim_start_matches("file://"))?))
    }
    fn get_range(&self, url: &str, start: u64, len: u64, _if_match: Option<&str>) -> SourceResult<Vec<u8>> {
        use std::io::{Seek, SeekFrom};
        let mut f = std::fs::File::open(url.trim_start_matches("file://"))?;
        f.seek(SeekFrom::Start(start))?;
        let mut out = Vec::new();
        f.take(len).read_to_end(&mut out)?;
        Ok(out)
    }
}

/// A producer whose function name (`_claimy_api`) matches no scheme: it is
/// only reachable through the claim pass, and claims `t3://` URLs the way a
/// configured objectstore would.
struct ClaimyApi;
impl SourceApi for ClaimyApi {
    fn head(&self, _url: &str) -> SourceResult<SourceMeta> {
        Ok(SourceMeta { size: Some(3), ..Default::default() })
    }
    fn get(&self, _url: &str, _if_match: Option<&str>) -> SourceResult<Box<dyn Read + Send>> {
        Ok(Box::new(std::io::Cursor::new(b"t3!".to_vec())))
    }
    fn get_range(&self, _url: &str, _s: u64, _l: u64, _m: Option<&str>) -> SourceResult<Vec<u8>> {
        Ok(b"t3!".to_vec())
    }
    fn supports_claims(&self) -> bool {
        true
    }
    fn claims(&self, url: &str) -> bool {
        if url == "t3://panic" {
            panic!("claims boom");
        }
        url.starts_with("t3://")
    }
}
fn claimy_api(context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
    source::result_source_api(context, ClaimyApi);
    Ok(())
}

fn mem_api(context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
    let mut objects = HashMap::new();
    objects.insert("mem://hello.txt".to_string(), b"hello, world\n".to_vec());
    objects.insert("mem://big.bin".to_string(), (0..200_000u32).map(|i| (i % 251) as u8).collect());
    objects.insert("mem://data/2024/01/part-1.csv".to_string(), b"a,b\n1,2\n".to_vec());
    objects.insert("mem://data/2024/01/part-2.csv".to_string(), b"a,b\n3,4\n".to_vec());
    objects.insert("mem://data/2024/02/part-1.csv".to_string(), b"a,b\n5,6\n".to_vec());
    objects.insert("mem://data/2024/02/notes.txt".to_string(), b"x".to_vec());
    objects.insert("mem://data/2024/summary.csv".to_string(), b"a,b\n".to_vec());
    objects.insert("mem://nometa/x.csv".to_string(), b"a\n".to_vec());
    source::result_source_api(context, MemApi { objects });
    Ok(())
}
fn file_api(context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
    source::result_source_api(context, FileApi);
    Ok(())
}
/// A provider whose resolver function itself fails (e.g. bad config).
fn err_api(_context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
    Err(sqlite_loadable::Error::new_message("bad config row"))
}
/// A function that returns *some other* pointer type, to test NotASourceApi.
fn other_pointer(context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
    api::result_pointer(context, b"something-else\0", 42i32);
    Ok(())
}

// ---- consumer side, exposed to SQL for testing ----

fn resolve(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<source::SourceHandle> {
    let function = api::value_text(&values[0])?;
    Ok(resolve_source_api(api::context_db_handle(context), function)?)
}

fn t_head(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let h = resolve(context, values)?;
    let meta = h.head(api::value_text(&values[1])?)?;
    let json = serde_json::json!({
        "size": meta.size, "last_modified_ms": meta.last_modified_ms,
        "etag": meta.etag, "content_type": meta.content_type,
    });
    api::result_text(context, json.to_string())?;
    Ok(())
}
fn t_get(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let h = resolve(context, values)?;
    let mut stream = h.get(api::value_text(&values[1])?)?;
    let mut out = Vec::new();
    stream.read_to_end(&mut out).map_err(|e| sqlite_loadable::Error::new_message(e.to_string()))?;
    api::result_blob(context, &out);
    Ok(())
}
fn t_range(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let h = resolve(context, values)?;
    let if_match = values.get(4).map(|v| api::value_text(v)).transpose()?;
    let bytes = h.get_range_if(
        api::value_text(&values[1])?,
        api::value_int64(&values[2]) as u64,
        api::value_int64(&values[3]) as u64,
        if_match,
    )?;
    api::result_blob(context, &bytes);
    Ok(())
}
fn t_get_if(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let h = resolve(context, values)?;
    let mut stream = h.get_if(api::value_text(&values[1])?, Some(api::value_text(&values[2])?))?;
    let mut out = Vec::new();
    stream.read_to_end(&mut out).map_err(|e| sqlite_loadable::Error::new_message(e.to_string()))?;
    api::result_blob(context, &out);
    Ok(())
}
/// Resolve, clone twice, drop everything; returns the drop count delta for MemApi.
fn t_refcount(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let before = MEM_API_DROPS.load(Ordering::SeqCst);
    {
        let h = resolve(context, values)?;
        let h2 = h.clone();
        let h3 = h2.clone();
        // The producing statement is already finalized inside resolve();
        // the handle must still work.
        let _ = h3.head("mem://hello.txt")?;
        drop(h);
        drop(h2);
        let _ = h3.head("mem://hello.txt")?;
    }
    let after = MEM_API_DROPS.load(Ordering::SeqCst);
    api::result_int64(context, (after - before) as i64);
    Ok(())
}
fn entries_json(entries: &[SourceEntry]) -> String {
    let items: Vec<serde_json::Value> = entries
        .iter()
        .map(|e| {
            serde_json::json!({
                "url": e.url, "size": e.size, "etag": e.etag,
                "last_modified_ms": e.last_modified_ms,
            })
        })
        .collect();
    serde_json::Value::Array(items).to_string()
}
/// `t_list(fn, prefix)` -> JSON array of entries, in the producer's order.
fn t_list(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let h = resolve(context, values)?;
    let entries = h.list(api::value_text(&values[1])?)?;
    api::result_text(context, entries_json(&entries))?;
    Ok(())
}
/// `t_glob(fn, url)` -> JSON array of the entries matching a remote glob
/// (sorted), or the string "not a glob".
fn t_glob(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let h = resolve(context, values)?;
    match RemoteGlob::parse(api::value_text(&values[1])?)? {
        Some(g) => api::result_text(context, entries_json(&g.expand(&h)?))?,
        None => api::result_text(context, "not a glob")?,
    }
    Ok(())
}
fn t_supports_list(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let h = resolve(context, values)?;
    api::result_bool(context, h.supports_list());
    Ok(())
}
fn t_for_source(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let db = api::context_db_handle(context);
    match source::resolve_for_source(db, api::value_text(&values[0])?) {
        Ok(None) => api::result_text(context, "local")?,
        Ok(Some((scheme, _h))) => api::result_text(context, scheme.function)?,
        Err(e) => api::result_text(context, format!("error: {}", e))?,
    }
    Ok(())
}
/// `t_claims(fn, url)` -> whether that producer claims the URL.
fn t_claims(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let h = resolve(context, values)?;
    api::result_bool(context, h.claims(api::value_text(&values[1])?));
    Ok(())
}
#[sqlite_entrypoint]
pub fn sqlite3_sourcetest_init(db: *mut sqlite3) -> Result<()> {
    let flags = FunctionFlags::UTF8;
    define_scalar_function(db, "_mem_api", 0, mem_api, flags)?;
    define_scalar_function(db, "_file_api", 0, file_api, flags)?;
    define_scalar_function(db, "_claimy_api", 0, claimy_api, flags)?;
    define_scalar_function(db, "_other_pointer", 0, other_pointer, flags)?;
    define_scalar_function(db, "_err_api", 0, err_api, flags)?;
    define_scalar_function(db, "t_head", 2, t_head, flags)?;
    define_scalar_function(db, "t_get", 2, t_get, flags)?;
    define_scalar_function(db, "t_range", 4, t_range, flags)?;
    define_scalar_function(db, "t_range", 5, t_range, flags)?;
    define_scalar_function(db, "t_get_if", 3, t_get_if, flags)?;
    define_scalar_function(db, "t_refcount", 1, t_refcount, flags)?;
    define_scalar_function(db, "t_for_source", 1, t_for_source, flags)?;
    define_scalar_function(db, "t_list", 2, t_list, flags)?;
    define_scalar_function(db, "t_glob", 2, t_glob, flags)?;
    define_scalar_function(db, "t_supports_list", 1, t_supports_list, flags)?;
    define_scalar_function(db, "t_claims", 2, t_claims, flags)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::{ffi::sqlite3_auto_extension, Connection};

    // MEM_API_DROPS is global, so tests must not interleave.
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn conn() -> (Connection, std::sync::MutexGuard<'static, ()>) {
        let guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite3_sourcetest_init as *const (),
            )));
        }
        (Connection::open_in_memory().unwrap(), guard)
    }
    fn q<T: rusqlite::types::FromSql>(db: &Connection, sql: &str) -> T {
        db.query_row(sql, [], |r| r.get(0)).unwrap()
    }
    fn err(db: &Connection, sql: &str) -> String {
        db.query_row(sql, [], |r| r.get::<_, String>(0)).unwrap_err().to_string()
    }

    #[test]
    fn pointer_is_null_typed() {
        let (db, _g) = conn();
        assert_eq!(q::<String>(&db, "select typeof(_mem_api())"), "null");
    }

    #[test]
    fn head_get_range() {
        let (db, _g) = conn();
        let meta: String = q(&db, "select t_head('_mem_api', 'mem://hello.txt')");
        assert_eq!(
            meta,
            r#"{"content_type":"text/plain","etag":"\"etag-13\"","last_modified_ms":1700000000000,"size":13}"#
        );
        let body: Vec<u8> = q(&db, "select t_get('_mem_api', 'mem://hello.txt')");
        assert_eq!(body, b"hello, world\n");
        let big: Vec<u8> = q(&db, "select t_get('_mem_api', 'mem://big.bin')");
        assert_eq!(big.len(), 200_000);
        assert_eq!(big[1000], (1000 % 251) as u8);
        let r: Vec<u8> = q(&db, "select t_range('_mem_api', 'mem://hello.txt', 7, 5)");
        assert_eq!(r, b"world");
        let r: Vec<u8> = q(&db, "select t_range('_mem_api', 'mem://hello.txt', 7, 500)");
        assert_eq!(r, b"world\n");
        let r: Vec<u8> = q(&db, "select t_range('_mem_api', 'mem://hello.txt', 13, 5)");
        assert_eq!(r, b"");
    }

    #[test]
    fn file_provider() {
        let (db, _g) = conn();
        let path = std::env::current_dir().unwrap().join("Cargo.toml");
        let expected = std::fs::read(&path).unwrap();
        let url = format!("file://{}", path.display());
        let body: Vec<u8> = q(&db, &format!("select t_get('_file_api', '{}')", url));
        assert_eq!(body, expected);
        let meta: String = q(&db, &format!("select t_head('_file_api', '{}')", url));
        assert!(meta.contains(&format!("\"size\":{}", expected.len())), "{}", meta);
        assert!(meta.contains("\"etag\":null"), "{}", meta);
        let r: Vec<u8> = q(&db, &format!("select t_range('_file_api', '{}', 1, 8)", url));
        assert_eq!(r, &expected[1..9]);
    }

    #[test]
    fn errors() {
        let (db, _g) = conn();
        let e = err(&db, "select t_get('_nope_api', 'mem://x')");
        assert!(e.contains("no source extension loaded (expected a _nope_api() function)"), "{}", e);
        let e = err(&db, "select t_get('_err_api', 'mem://x')");
        assert!(e.contains("could not resolve source API _err_api(): bad config row"), "{}", e);
        let e = err(&db, "select t_get('_other_pointer', 'mem://x')");
        assert!(e.contains("_other_pointer() did not return a sqlite-source-api-v2 pointer"), "{}", e);
        let e = err(&db, "select t_get('_mem_api', 'mem://missing')");
        assert!(e.contains("mem: mem://missing not found"), "{}", e);
        let e = err(&db, "select t_range('_mem_api', 'mem://hello.txt', 99, 1)");
        assert!(e.contains("range start past end"), "{}", e);
        let e = err(&db, "select t_get('_file_api', 'file:///definitely/not/here')");
        assert!(e.contains("No such file"), "{}", e);
        // panics inside the producer are caught at the FFI boundary
        let e = err(&db, "select t_get('_mem_api', 'mem://panic')");
        assert!(e.contains("panic in source provider: boom"), "{}", e);
        let e = err(&db, "select t_get('bad name; drop', 'mem://x')");
        assert!(e.contains("invalid function name"), "{}", e);
    }

    #[test]
    fn if_match() {
        let (db, _g) = conn();
        let ok: Vec<u8> = q(&db, "select t_range('_mem_api', 'mem://hello.txt', 0, 5, '\"etag-13\"')");
        assert_eq!(ok, b"hello");
        let e = err(&db, "select t_range('_mem_api', 'mem://hello.txt', 0, 5, '\"etag-old\"')");
        assert!(
            e.contains("mem://hello.txt changed since it was opened (ETag \"etag-old\" no longer matches)"),
            "{}", e
        );
        let ok: Vec<u8> = q(&db, "select t_get_if('_mem_api', 'mem://hello.txt', '\"etag-13\"')");
        assert_eq!(ok, b"hello, world\n");
        let e = err(&db, "select t_get_if('_mem_api', 'mem://hello.txt', 'nope')");
        assert!(e.contains("changed since it was opened"), "{}", e);
    }

    #[test]
    fn refcount() {
        let (db, _g) = conn();
        // handle outlives the resolving statement; exactly one drop once all clones are gone
        assert_eq!(q::<i64>(&db, "select t_refcount('_mem_api')"), 1);
        // and the pointer value produced directly in SQL is freed with the statement
        let before = MEM_API_DROPS.load(Ordering::SeqCst);
        q::<String>(&db, "select typeof(_mem_api())");
        assert_eq!(MEM_API_DROPS.load(Ordering::SeqCst), before + 1);
    }

    #[test]
    fn list() {
        let (db, _g) = conn();
        assert_eq!(q::<bool>(&db, "select t_supports_list('_mem_api')"), true);
        assert_eq!(q::<bool>(&db, "select t_supports_list('_file_api')"), false);

        let out: String = q(&db, "select t_list('_mem_api', 'mem://data/2024/01/')");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        let urls: Vec<&str> = v.as_array().unwrap().iter().map(|e| e["url"].as_str().unwrap()).collect();
        // producer order is preserved by list() itself
        assert_eq!(urls, ["mem://data/2024/01/part-2.csv", "mem://data/2024/01/part-1.csv"]);
        assert_eq!(v[0]["size"], 8);
        assert_eq!(v[0]["etag"], "\"etag-8\"");
        assert_eq!(v[0]["last_modified_ms"], 1_700_000_000_000i64);
        // unknown metadata comes back as null
        let out: String = q(&db, "select t_list('_mem_api', 'mem://nometa/')");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v[0]["size"], serde_json::Value::Null);
        assert_eq!(v[0]["etag"], serde_json::Value::Null);
        // empty listing is fine
        assert_eq!(q::<String>(&db, "select t_list('_mem_api', 'mem://nothing/')"), "[]");

        // NULL slot: the file provider does not list
        let e = err(&db, "select t_list('_file_api', 'file:///tmp/')");
        assert!(e.contains("listing is not supported by this source"), "{}", e);
        let e = err(&db, "select t_glob('_file_api', 'file:///tmp/*.csv')");
        assert!(e.contains("listing is not supported by this source"), "{}", e);
        // producer errors pass through
        let e = err(&db, "select t_list('_mem_api', 'mem://boom/')");
        assert!(e.contains("mem: list exploded"), "{}", e);
    }

    #[test]
    fn remote_glob() {
        let (db, _g) = conn();
        let heads = MEM_API_HEADS.load(Ordering::SeqCst);
        let urls = |sql: &str| -> Vec<String> {
            let out: String = q(&db, sql);
            let v: serde_json::Value = serde_json::from_str(&out).unwrap();
            v.as_array().unwrap().iter().map(|e| e["url"].as_str().unwrap().to_owned()).collect()
        };
        assert_eq!(
            urls("select t_glob('_mem_api', 'mem://data/2024/*/part-*.csv')"),
            ["mem://data/2024/01/part-1.csv", "mem://data/2024/01/part-2.csv", "mem://data/2024/02/part-1.csv"]
        );
        assert_eq!(
            urls("select t_glob('_mem_api', 'mem://data/2024/*.csv')"),
            ["mem://data/2024/summary.csv"]
        );
        assert_eq!(
            urls("select t_glob('_mem_api', 'mem://data/**/*.csv')"),
            [
                "mem://data/2024/01/part-1.csv",
                "mem://data/2024/01/part-2.csv",
                "mem://data/2024/02/part-1.csv",
                "mem://data/2024/summary.csv",
            ]
        );
        assert_eq!(urls("select t_glob('_mem_api', 'mem://data/2024/0[2]/*')"), ["mem://data/2024/02/notes.txt", "mem://data/2024/02/part-1.csv"]);
        assert_eq!(q::<String>(&db, "select t_glob('_mem_api', 'mem://hello.txt')"), "not a glob");
        let e = err(&db, "select t_glob('_mem_api', 'mem://data/2024/*.parquet')");
        assert!(e.contains("no files matched 'mem://data/2024/*.parquet'"), "{}", e);
        // expanding a glob is one list() call, never a head()
        assert_eq!(MEM_API_HEADS.load(Ordering::SeqCst), heads);
    }

    #[test]
    fn dispatch() {
        let (db, _g) = conn();
        assert_eq!(q::<String>(&db, "select t_for_source('data/x.csv')"), "local");
        // const map: a mapped scheme with no producer errors immediately, no fallthrough
        assert_eq!(
            q::<String>(&db, "select t_for_source('https://example.com/x.csv')"),
            "error: no source extension loaded for \"https://\" URLs (expected a _http_api() function)"
        );
        // derived probe: mem:// finds _mem_api by naming convention
        assert_eq!(q::<String>(&db, "select t_for_source('mem://hello.txt')"), "_mem_api");
        assert_eq!(q::<String>(&db, "select t_for_source('MEM://hello.txt')"), "_mem_api");
        // claim pass: no _t3_api exists, but _claimy_api claims t3:// URLs
        assert_eq!(q::<String>(&db, "select t_for_source('t3://bucket/k.csv')"), "_claimy_api");
        // a panicking claims() is caught and treated as "does not claim"
        assert_eq!(q::<String>(&db, "select t_for_source('t3://panic')"), "local");
        // nothing serves or claims zzz:// (the broken _err_api is skipped, not fatal)
        assert_eq!(q::<String>(&db, "select t_for_source('zzz://x/y.csv')"), "local");
        db.execute_batch("").unwrap();
    }

    #[test]
    fn claims() {
        let (db, _g) = conn();
        assert_eq!(q::<bool>(&db, "select t_claims('_claimy_api', 't3://b/k')"), true);
        assert_eq!(q::<bool>(&db, "select t_claims('_claimy_api', 's3://b/k')"), false);
        // NULL claims slot on producers that don't opt in
        assert_eq!(q::<bool>(&db, "select t_claims('_mem_api', 't3://b/k')"), false);
    }
}
