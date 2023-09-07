#![doc = include_str!("../README.md")]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod api;
pub mod collation;
mod constants;
pub mod entrypoints;
pub mod errors;

#[cfg(feature = "exec")]
pub mod exec;
pub mod ext; // TODO dont expose
pub mod prelude;
pub mod scalar;
pub mod table;
pub mod vtab_argparse;
pub mod vfs;

#[doc(inline)]
pub use errors::{Error, ErrorKind, Result};

#[doc(inline)]
pub use scalar::{define_scalar_function, define_scalar_function_with_aux, FunctionFlags};

#[doc(inline)]
pub use collation::define_collation;

#[doc(inline)]
pub use table::{
    define_table_function, define_virtual_table, define_virtual_table_writeable,
    define_virtual_table_writeablex, BestIndexError,
};
