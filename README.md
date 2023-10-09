# sqlite-loadable-rs

[![Latest Version](https://img.shields.io/crates/v/sqlite-loadable.svg)](https://crates.io/crates/sqlite-loadable)
[![Documentation](https://docs.rs/sqlite-loadable/badge.svg)](https://docs.rs/sqlite-loadable)

A framework for building loadable SQLite extensions in Rust. Inspired by [rusqlite](https://github.com/rusqlite/rusqlite), [pgx](https://github.com/tcdi/pgx), and Riyaz Ali's similar [SQLite Go library](https://github.com/riyaz-ali/sqlite). See [_Introducing sqlite-loadable-rs: A framework for building SQLite Extensions in Rust_](https://observablehq.com/@asg017/introducing-sqlite-loadable-rs) (Dec 2022) for more details!

If your company or organization finds this library useful, consider [supporting my work](#supporting)!

---

> **Warning**
> Still in beta, very unstable and unsafe code! Watch the repo for new releases, or [follow my newsletter/RSS feed](https://buttondown.email/alexgarcia) for future updates.

---

## Background

SQLite's [runtime loadable extensions](https://www.sqlite.org/loadext.html) allows one to add new scalar functions, table functions, virtual tables, virtual filesystems, and more to a SQLite database connection. These compiled [dynamically-linked libraries](https://en.wikipedia.org/wiki/Dynamic-link_library) can be loaded in any SQLite context, including the [SQLite CLI](https://sqlite.org/cli.html#loading_extensions), [Python](https://docs.python.org/3/library/sqlite3.html#sqlite3.Connection.load_extension), [Node.js](https://github.com/WiseLibs/better-sqlite3/blob/master/docs/api.md#loadextensionpath-entrypoint---this), [Rust](https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html#method.load_extension), [Go](https://pkg.go.dev/github.com/mattn/go-sqlite3#SQLiteConn.LoadExtension), and many other languages.

> **Note**
> Notice the word _loadable_. Loadable extensions are these compiled dynamically-linked libraries, with a suffix of `.dylib` or `.so` or `.dll` (depending on your operating system). These are different than [application-defined functions](https://www.sqlite.org/appfunc.html) that many language clients support (such as Python's [`.create_function()`](https://docs.python.org/3/library/sqlite3.html#sqlite3.Connection.create_function) or Node.js's [`.function()`](https://github.com/WiseLibs/better-sqlite3/blob/master/docs/api.md#functionname-options-function---this)).

Historically, the main way one could create these _loadable_ SQLite extensions were with C/C++, such as [spatilite](https://www.gaia-gis.it/fossil/libspatialite/index), the wonderful [sqlean project](https://github.com/nalgeon/sqlean), or SQLite's [official miscellaneous extensions](https://www.sqlite.org/src/file/ext/misc).

But C is difficult to use safely, and integrating 3rd party libraries can be a nightmare. Riyaz Ali wrote a [Go library](https://github.com/riyaz-ali/sqlite) that allows one to easily write loadable extensions in Go, but it comes with a large performance cost and binary size. For Rust, [rusqlite](https://github.com/rusqlite/rusqlite) has had a few different PRs that attempted to add loadable extension support in that library, but none have been merged.

So, `sqlite-loadable-rs` is the first and most involved framework for writing loadable SQLite extensions in Rust!

## Features

### Scalar functions

Scalar functions are the simplest functions one can add to SQLite - take in values as inputs, and return a value as output. To implement one in `sqlite-loadable-rs`, you just need to call `define_scalar_function` on a "callback" Rust function decorated with `#[sqlite_entrypoint]`, and you'll be able to call it from SQL!

```rust
// add(a, b)
fn add(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let a = api::value_int(values.get(0).expect("1st argument"));
    let b = api::value_int(values.get(1).expect("2nd argument"));
    api::result_int(context, a + b);
    Ok(())
}

// connect(seperator, string1, string2, ...)
fn connect(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let seperator = api::value_text(values.get(0).expect("1st argument"))?;
    let strings:Vec<&str> = values
        .get(1..)
        .expect("more than 1 argument to be given")
        .iter()
        .filter_map(|v| api::value_text(v).ok())
        .collect();
    api::result_text(context, &strings.join(seperator))?;
    Ok(())
}
#[sqlite_entrypoint]
pub fn sqlite3_extension_init(db: *mut sqlite3) -> Result<()> {
    define_scalar_function(db, "add", 2, add, FunctionFlags::DETERMINISTIC)?;
    define_scalar_function(db, "connect", -1, connect, FunctionFlags::DETERMINISTIC)?;
    Ok(())
}
```

```sql
sqlite> select add(1, 2);
3
sqlite> select connect('-', 'alex', 'brian', 'craig');
alex-brian-craig
```

See [`define_scalar_function`](https://docs.rs/sqlite-loadable/latest/sqlite_loadable/fn.define_scalar_function.html) for more info.

### Table functions

Table functions, (aka "[Eponymous-only virtual tables](https://www.sqlite.org/vtab.html#eponymous_only_virtual_tables)"), can be added to your extension with [`define_table_function`](https://docs.rs/sqlite-loadable/latest/sqlite_loadable/fn.define_table_function.html).

```rust
define_table_function::<CharactersTable>(db, "characters", None)?;
```

Defining a table function is complicated and requires a lot of code - see the [`characters.rs`](./examples/characters.rs) example for a full solution.

Once compiled, you can invoke a table function like querying any other table, with any arguments that the table function supports.

```sql
sqlite> .load target/debug/examples/libcharacters
sqlite> select rowid, * from characters('alex garcia');
┌───────┬───────┐
│ rowid │ value │
├───────┼───────┤
│ 0     │ a     │
│ 1     │ l     │
│ 2     │ e     │
│ 3     │ x     │
│ 4     │       │
│ 5     │ g     │
│ 6     │ a     │
│ 7     │ r     │
│ 8     │ c     │
│ 9     │ i     │
│ 10    │ a     │
└───────┴───────┘
```

Some real-world non-Rust examples of table functions in SQLite:

- [json_each](https://www.sqlite.org/json1.html#jeach) / [json_tree](https://www.sqlite.org/json1.html#jtree)
- [generate_series](https://www.sqlite.org/series.html)
- [pragma\_\*](https://www.sqlite.org/pragma.html#pragfunc) functions
- [html_each](https://github.com/asg017/sqlite-html/blob/main/docs.md#html_each)

### Virtual tables

`sqlite-loadable-rs` also supports more traditional [virtual tables](https://www.sqlite.org/vtab.html), for tables that have a dynamic schema or need insert/update support.

[`define_virtual_table()`](https://docs.rs/sqlite-loadable/latest/sqlite_loadable/fn.define_virtual_table.html) can define a new read-only virtual table module for the given SQLite connection. [`define_virtual_table_writeable()`](https://docs.rs/sqlite-loadable/latest/sqlite_loadable/fn.define_virtual_table_writeable.html) is also available for tables that support `INSERT`/`UPDATE`/`DELETE`, but this API will probably change.

```rust
define_virtual_table::<CustomVtab>(db, "custom_vtab", None)?
```

These virtual tables can be created in SQL with the `CREATE VIRTUAL TABLE` syntax.

```sql

create virtual table xxx using custom_vtab(arg1=...);

select * from xxx;

```

Some real-world non-Rust examples of traditional virtual tables in SQLite include the [CSV virtual table](https://www.sqlite.org/csv.html), the full-text search [fts5 extension](https://www.sqlite.org/fts5.html#fts5_table_creation_and_initialization), and the [R-Tree extension](https://www.sqlite.org/rtree.html#creating_an_r_tree_index).

### Virtual file system

There are two examples of how to apply this library to create your own vfs.
1. [io_uring_vfs](./benchmarks/vfs/io_uring/)
2. [mem_vfs](./examples/mem_vfs.rs)

## Examples

The [`examples/`](./examples/) directory has a few bare-bones examples of extensions, which you can build with:

```bash
$ cargo build --example hello
$ sqlite3 :memory: '.load target/debug/examples/hello' 'select hello("world");'
hello, world!

# Build all the examples in release mode, with output at target/debug/release/examples/*.dylib
$ cargo build --example --release
```

Some real-world projects that use `sqlite-loadable-rs`:

- [`sqlite-xsv`](https://github.com/asg017/sqlite-xsv) - An extremely fast CSV/TSV parser in SQLite
- [`sqlite-regex`](https://github.com/asg017/sqlite-regex) - An extremely fast and safe regular expression library for SQLite
- [`sqlite-base64`](https://github.com/asg017/sqlite-base64) - Fast base64 encoding and decoding in SQLite

I plan to release many more extensions in the near future!

## Usage

`cargo init --lib` a new project, and add `sqlite-loadable` to your dependencies in `Cargo.toml`.

```toml
[package]
name = "xyz"
version = "0.1.0"
edition = "2021"

[dependencies]
sqlite-loadable = "0.0.3"

[lib]
crate-type=["cdylib"]

```

Then, fill in your `src/lib.rs` with a "hello world" extension:

```rust
use sqlite_loadable::prelude::*;
use sqlite_loadable::{
  api,
  define_scalar_function, Result,
};

pub fn hello(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let name = api::value_text_notnull(values.get(0).expect("1st argument as name"))?;
    api::result_text(context, format!("hello, {}!", name))?;
    Ok(())
}

#[sqlite_entrypoint]
pub fn sqlite3_hello_init(db: *mut sqlite3) -> Result<()> {
    define_scalar_function(db, "hello", 1, hello, FunctionFlags::UTF8 | FunctionFlags::DETERMINISTIC)?;
    Ok(())
}

```

Build it `cargo build`, spin up the SQLite CLI, and try out your new extension!

```sql
$ sqlite3
sqlite> .load target/debug/libhello
sqlite> select hello('world');
hello, world!
```

<small><i>([MacOS workaround](https://til.simonwillison.net/sqlite/trying-macos-extensions))</i></small>

## Benchmarks

See more details at [`benchmarks/`](benchmarks/), but in general, a "hello world" extension built with `sqlite-loadable-rs` is about 10-15% slower than one built in C, and several orders of magnitude faster than extensions written in Go with `riyaz-ali/sqlite` (20-30x faster).

However, it depends on what your extension actually _does_ - very rarely do you need a "hello world" type extension in real life. For example, `sqlite-xsv` is 1.5-1.7x faster than the "offical" [CSV SQLite extension](https://www.sqlite.org/csv.html) written in C, and `sqlite-regex` is 2x faster than the [regexp](https://github.com/sqlite/sqlite/blob/master/ext/misc/regexp.c) extension.

## Caveats

### Heavy use of `unsafe` Rust

`sqlite-loadable-rs` uses the SQLite C API heavily, which means `unsafe` code. I try my best to make it as safe as possible, and it's good that SQLite itself is [one of the most well-tested C codebases in the world](https://www.sqlite.org/testing.html), but you can never be sure!

### Maybe doesn't work in multi-threaded environments

Just because I haven't tested it. If you use SQLite in ["serialized mode"](https://sqlite.org/threadsafe.html) or with `-DSQLITE_THREADSAFE=1`, then I'm not sure if `sqlite-loadable-rs` will work as expected. If you try this and find problems, please file an issue!

### Doesn't work with rusqlite

If you already have Rust code that uses [rusqlite](https://github.com/rusqlite/rusqlite) to make scalar functions or virtual tables, you won't be able to re-use it in `sqlite-loadable-rs`. Sorry!

Though if you want to use an extension built with `sqlite-loadable-rs` in an app that uses rusqlite, consider [`Connection.load_extension()`](https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html#method.load_extension) for dynamic loading, or [`Connection.handle()`](https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html#method.handle) + [`sqlite3_auto_extension()`](https://www.sqlite.org/capi3ref.html#sqlite3_auto_extension) for static compilation.

### Probably can't be compiled into WASM

SQLite by itself can be compiled into WASM, and you can also include extensions written in C if you compile those extensions statically before compiling with emscripten (see [sqlite-lines](https://github.com/asg017/sqlite-lines) or [sqlite-path](https://github.com/asg017/sqlite-path) for examples).

However, the same can't be done with `sqlite-loadable-rs`. As far as I can tell, you can't easily compile a Rust project to WASM if there's a C dependency. There are projects like the `wasm32-unknown-emscripten` target that could maybe solve this, but I haven't gotten it to work yet. But I'm not an expert in emscripten or Rust/WASM, so if you think it's possible, please file an issue!

### Larger binary size

A hello world extension in C is `17KB`, while one in Rust is `469k`. It's still much smaller than one in Go, which is around `2.2M` using `riyaz-ali/sqlite`, but something to consider. It's still small enough where you won't notice most of the time, however.

## Roadmap

- [ ] Stabilize scalar function interface
- [ ] Stabilize virtual table interface
- [ ] Support [aggregate window functions](https://www.sqlite.org/windowfunctions.html#udfwinfunc) ([#1](https://github.com/asg017/sqlite-loadable-rs/issues/1))
- [ ] Support [collating sequences](https://www.sqlite.org/c3ref/create_collation.html) ([#2](https://github.com/asg017/sqlite-loadable-rs/issues/2))
- [x] Support [virtual file systems](sqlite.org/vfs.html) ([#3](https://github.com/asg017/sqlite-loadable-rs/issues/3))

## Supporting

I (Alex 👋🏼) spent a lot of time and energy on this project and [many other open source projects](https://github.com/asg017?tab=repositories&q=&type=&language=&sort=stargazers). If your company or organization uses this library (or you're feeling generous), then please [consider supporting my work](https://alexgarcia.xyz/work.html), or share this project with a friend!
