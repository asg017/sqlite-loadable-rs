//! End-to-end test of `sqlite_loadable::source`: an in-memory producer and a
//! file producer, plus SQL-callable consumer functions that resolve them
//! through `resolve_source_api`, all loaded into one rusqlite connection.
#![cfg(feature = "source")]

use sqlite_loadable::prelude::*;
use sqlite_loadable::source::{
    self, resolve_source_api, SourceApi, SourceError, SourceMeta, SourceResult,
};
use sqlite_loadable::{api, define_scalar_function, Result};
use std::collections::HashMap;
use std::io::Read;
use std::sync::atomic::{AtomicUsize, Ordering};

static MEM_API_DROPS: AtomicUsize = AtomicUsize::new(0);

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
}
impl SourceApi for MemApi {
    fn head(&self, url: &str) -> SourceResult<SourceMeta> {
        let o = self.object(url)?;
        Ok(SourceMeta {
            size: Some(o.len() as u64),
            last_modified_ms: Some(1_700_000_000_000),
            etag: Some(format!("\"etag-{}\"", o.len())),
            content_type: Some("text/plain".into()),
        })
    }
    fn get(&self, url: &str) -> SourceResult<Box<dyn Read + Send>> {
        if url == "mem://panic" {
            panic!("boom");
        }
        Ok(Box::new(std::io::Cursor::new(self.object(url)?.clone())))
    }
    fn get_range(&self, url: &str, start: u64, len: u64) -> SourceResult<Vec<u8>> {
        let o = self.object(url)?;
        let start = start as usize;
        let end = (start + len as usize).min(o.len());
        if start > o.len() {
            return Err("range start past end".into());
        }
        Ok(o[start..end].to_vec())
    }
}

struct FileApi;
impl SourceApi for FileApi {
    fn head(&self, url: &str) -> SourceResult<SourceMeta> {
        let md = std::fs::metadata(url.trim_start_matches("file://"))?;
        Ok(SourceMeta { size: Some(md.len()), ..Default::default() })
    }
    fn get(&self, url: &str) -> SourceResult<Box<dyn Read + Send>> {
        Ok(Box::new(std::fs::File::open(url.trim_start_matches("file://"))?))
    }
    fn get_range(&self, url: &str, start: u64, len: u64) -> SourceResult<Vec<u8>> {
        use std::io::{Seek, SeekFrom};
        let mut f = std::fs::File::open(url.trim_start_matches("file://"))?;
        f.seek(SeekFrom::Start(start))?;
        let mut out = Vec::new();
        f.take(len).read_to_end(&mut out)?;
        Ok(out)
    }
}

fn mem_api(context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
    let mut objects = HashMap::new();
    objects.insert("mem://hello.txt".to_string(), b"hello, world\n".to_vec());
    objects.insert("mem://big.bin".to_string(), (0..200_000u32).map(|i| (i % 251) as u8).collect());
    source::result_source_api(context, MemApi { objects });
    Ok(())
}
fn file_api(context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
    source::result_source_api(context, FileApi);
    Ok(())
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
    api::result_json(
        context,
        serde_json::json!({
            "size": meta.size, "last_modified_ms": meta.last_modified_ms,
            "etag": meta.etag, "content_type": meta.content_type,
        }),
    )?;
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
    let bytes = h.get_range(
        api::value_text(&values[1])?,
        api::value_int64(&values[2]) as u64,
        api::value_int64(&values[3]) as u64,
    )?;
    api::result_blob(context, &bytes);
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
fn t_for_source(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let db = api::context_db_handle(context);
    match source::resolve_for_source(db, api::value_text(&values[0])?) {
        Ok(None) => api::result_text(context, "local")?,
        Ok(Some((scheme, _h))) => api::result_text(context, scheme.function)?,
        Err(e) => api::result_text(context, format!("error: {}", e))?,
    }
    Ok(())
}

#[sqlite_entrypoint]
pub fn sqlite3_sourcetest_init(db: *mut sqlite3) -> Result<()> {
    let flags = FunctionFlags::UTF8;
    define_scalar_function(db, "_mem_api", 0, mem_api, flags)?;
    define_scalar_function(db, "_file_api", 0, file_api, flags)?;
    define_scalar_function(db, "_other_pointer", 0, other_pointer, flags)?;
    define_scalar_function(db, "t_head", 2, t_head, flags)?;
    define_scalar_function(db, "t_get", 2, t_get, flags)?;
    define_scalar_function(db, "t_range", 4, t_range, flags)?;
    define_scalar_function(db, "t_refcount", 1, t_refcount, flags)?;
    define_scalar_function(db, "t_for_source", 1, t_for_source, flags)?;
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
        let e = err(&db, "select t_get('_other_pointer', 'mem://x')");
        assert!(e.contains("_other_pointer() did not return a sqlite-source-api-v1 pointer"), "{}", e);
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
    fn dispatch() {
        let (db, _g) = conn();
        assert_eq!(q::<String>(&db, "select t_for_source('data/x.csv')"), "local");
        assert_eq!(
            q::<String>(&db, "select t_for_source('https://example.com/x.csv')"),
            "error: no source extension loaded for \"https://\" URLs (expected a _http_api() function)"
        );
        db.execute_batch("").unwrap();
    }
}
