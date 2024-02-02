TODO

- aggregate functions
  - `scalar.rs` -> `functions.rs`
  - `Aggregate` and `Window` structs?
  - should it use `sqlite3_aggregate_context` or something else?
  - `create_aggregate_function`
  - `create_window_function`
- queries

  - `exec::prepare(db: *mut sqlite3, sql: &str) -> Result<Statement>`
  - `Statement.bind_int32(column_idx: i32, value: i32)`
  - `Statement.bind_text(column_idx: i32, value: &str)`
  - `Statement.bind_blob(column_idx: i32, value: &[u8])`
  - `Statement.execute() -> Iterator<Result<Row>>`
  - `Statement.execute_to_completion()`
  - `exec::execute(db, sql)`

- tests
  - charcters table func
  - some vtab to test argc/argv/argparse handling
  - more exec tests

## static feature

GOAL

1. For `crate-type=staticlib`, statically link sqlite3 for better static support
2. WASM requires static linking, since `-DSLQITE_OMIT_EXTENSIONS` is defined
3. Want to be able to do `sqlite3_hello_init(db)` manually instead of always relying on auto_extension

Problem: We don't want to always statically link sqlite3, since cdylib builds don't need it/it overwrites other linked sqlite builds sometimes.

Solution: the `ext.rs` functions need to export different functions (either `pApi->xFunc` for cdylib or `xFunc()` for static) depending on staticlib vs dylib.

Though there's no `crate_type="cdylib"` macro we can use, so gotta introduce a new feature flag. Also means builds will be weird.


```rs
#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_user_data2(context: *mut sqlite3_context) -> *mut c_void {
    libsqlite3_sys::sqlite3_user_data(context)
}
#[cfg(all(not(feature = "static"), feature = "dynamic"))]
pub unsafe fn sqlite3ext_user_data2(context: *mut sqlite3_context) -> *mut c_void {
    ((*SQLITE3_API).user_data.expect(EXPECT_MESSAGE))(context)
}

pub unsafe fn sqlite3ext_mprintf(s: *const c_char) -> *const c_char {
    ((*SQLITE3_API).mprintf.expect(EXPECT_MESSAGE))(s)
}

#[macro_export]
macro_rules! export_sqlite_function {
    ($func:ident) => {
        #[cfg(feature = "static")]
        pub unsafe fn $func(context: *mut sqlite3_context) -> *mut c_void {
          concat_idents!(libsqlite3_sys::, sqlite3_, $func)(context)
        }

        #[cfg(all(not(feature = "static"), feature = "dynamic"))]
        pub unsafe fn $func(context: *mut sqlite3_context) -> *mut c_void {
            ((*SQLITE3_API).$func.expect(EXPECT_MESSAGE))(context)
        }
    };
}

export_sqlite_function!(user_data);

#[cfg(feature = "static")]
use libsqlite3_sys;

#[cfg(feature = "static")]
pub unsafe fn sqlite3ext_auto_extension(f: unsafe extern "C" fn()) -> i32 {
    libsqlite3_sys::sqlite3_auto_extension(Some(f))
}
```

## WASM

```
RUSTFLAGS="-Clink-args=-sERROR_ON_UNDEFINED_SYMBOLS=0 -Clink-args=--no-entry" cargo build --example hello --target wasm32-unknown-emscripten --features=static


wget https://www.sqlite.org/2023/sqlite-src-3430100.zip
unzip sqlite-src-3430100.zip
cd sqlite-src-3430100/
./configure --enable-all
make sqlite3.c

cd ext/wasm
make sqlite3_wasm_extra_init.c=../../../../target/wasm32-unknown-emscripten/debug/examples/libhello.a "emcc.flags=-s EXTRA_EXPORTED_RUNTIME_METHODS=['ENV'] -s FETCH"
```

```rs
#[no_mangle]
/// in hello.rs


//#[cfg(target_os = "emscripten")]
pub extern "C" fn sqlite3_wasm_extra_init(_unused: *const std::ffi::c_char) -> std::ffi::c_int {
    use sqlite_loadable::SQLITE_OK;
    println!("sqlite3_wasm_extra_init");
    unsafe {
        sqlite_loadable::ext::sqlite3ext_auto_extension(std::mem::transmute(
            sqlite3_hello_init as *const (),
        ));
    }

    SQLITE_OK
}

```
