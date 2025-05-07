extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=sqlite3/sqlite3.c");
    // This is not needed for now - lets just just cdylib to work right.
    /*
        cc::Build::new()
            .file("sqlite3/sqlite3.c")
            //.include("sqlite3/sqlite3ext.h")
            .include("sqlite3/sqlite3.h")
            .flag("-DSQLITE_CORE")
            .compile("sqlite3");
    */
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::builder()
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .disable_nested_struct_naming()
        .trust_clang_mangling(false)
        .header("sqlite3/sqlite3ext.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR should be defined"));
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
