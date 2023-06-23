//! Defining virtual tables and table functions on sqlite3 database connections.

// ![allow(clippy::not_unsafe_ptr_arg_deref)]

use sqlite3ext_sys::sqlite3_index_info_sqlite3_index_constraint_usage;
use sqlite3ext_sys::{
    sqlite3, sqlite3_context, sqlite3_index_info, sqlite3_index_info_sqlite3_index_constraint,
    sqlite3_index_info_sqlite3_index_orderby, sqlite3_module, sqlite3_value, sqlite3_vtab,
    sqlite3_vtab_cursor,
};

use crate::constants::*;
use std::ffi::CString;
use std::marker::PhantomData;
use std::marker::Sync;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::slice;
use std::str::Utf8Error;

use crate::api::{mprintf, value_type, MprintfError, ValueType};
use crate::errors::{Error, ErrorKind, Result};
use crate::ext::sqlitex_declare_vtab;
use crate::ext::{sqlite3ext_create_module_v2, sqlite3ext_vtab_distinct};
use serde::{Deserialize, Serialize};

/// Possible operators for a given constraint, found and used in xBestIndex and xFilter.
/// <https://www.sqlite.org/c3ref/c_index_constraint_eq.html>
/// TODO EQ=Equals, GT=GreaterThan, etc.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintOperator {
    /// 'Equals', ex "="
    EQ,

    /// 'Greater Than', ex ">"
    GT,

    /// 'Less than or equal to', ex "<="
    LE,

    /// 'Less than', ex "<"
    LT,

    /// 'Greater than or equal to', ex ">="
    GE,

    /// 'Match' function
    MATCH,

    /// 'LIKE' function
    LIKE,

    /// 'Glob' function
    GLOB,

    /// 'REGEXP' function
    REGEXP,

    /// 'Not equal to', ex "!=" or "<>"
    NE,

    /// 'is not' operation
    ISNOT,

    /// 'is not null' operation
    ISNOTNULL,

    /// 'is null' operation
    ISNULL,

    /// 'is' operation
    IS,

    /// 'LIMIT' constraint
    LIMIT,

    /// 'OFFSET' operation
    OFFSET,

    /// custom funciton overload, used in xFindFunction, see <https://www.sqlite.org/vtab.html#xfindfunction>
    FUNCTION(u8),
}

/// Return the `ConstraintOperator` for the given raw operator, usually
/// from sqlite3_index_info.sqlite3_index_constraint.op .
/// Values from <https://www.sqlite.org/c3ref/c_index_constraint_eq.html>
pub fn operator(op: u8) -> Option<ConstraintOperator> {
    match op {
        2 => Some(ConstraintOperator::EQ),
        4 => Some(ConstraintOperator::GT),
        8 => Some(ConstraintOperator::LE),
        16 => Some(ConstraintOperator::LT),
        32 => Some(ConstraintOperator::GE),
        64 => Some(ConstraintOperator::MATCH),
        65 => Some(ConstraintOperator::LIKE),
        66 => Some(ConstraintOperator::GLOB),
        67 => Some(ConstraintOperator::REGEXP),
        68 => Some(ConstraintOperator::NE),
        69 => Some(ConstraintOperator::ISNOT),
        70 => Some(ConstraintOperator::ISNOTNULL),
        71 => Some(ConstraintOperator::ISNULL),
        72 => Some(ConstraintOperator::IS),
        73 => Some(ConstraintOperator::LIMIT),
        74 => Some(ConstraintOperator::OFFSET),
        150..=255 => Some(ConstraintOperator::FUNCTION(op)),
        _ => None,
    }
}

/// Wraps the raw sqlite3_index_info C struct, which represents
/// the possible constraints and outputs the xBestIndex method
/// should use and return.
/// <https://www.sqlite.org/c3ref/index_info.html>
#[derive(Debug)]
pub struct IndexInfo {
    index_info: *mut sqlite3_index_info,
}
impl IndexInfo {
    /// "Mask of SQLITE_INDEX_SCAN_* flags"
    pub fn idx_flag(&self) -> i32 {
        unsafe { (*self.index_info).idxFlags }
    }
    pub fn constraints(&self) -> Vec<Constraint> {
        let constraints = unsafe {
            slice::from_raw_parts(
                (*self.index_info).aConstraint,
                (*self.index_info).nConstraint as usize,
            )
        };

        let constraint_usages = unsafe {
            slice::from_raw_parts_mut(
                (*self.index_info).aConstraintUsage,
                (*self.index_info).nConstraint as usize,
            )
        };

        return constraints
            .iter()
            .zip(constraint_usages.iter_mut())
            .map(|z| Constraint {
                constraint: *z.0,
                usage: z.1,
            })
            .collect();
    }
    pub fn order_bys(&self) -> Vec<OrderBy> {
        let order_bys = unsafe {
            slice::from_raw_parts(
                (*self.index_info).aOrderBy,
                (*self.index_info).nOrderBy as usize,
            )
        };
        return order_bys.iter().map(|o| OrderBy { order_by: *o }).collect();
    }
    pub fn set_idxnum(&mut self, value: i32) {
        unsafe {
            (*self.index_info).idxNum = value;
        }
    }
    pub fn set_idxstr(&mut self, value: &str) -> crate::Result<()> {
        let idxstr = match mprintf(value) {
            Ok(idxstr) => idxstr,
            Err(err) => {
                return match err {
                    MprintfError::Oom => Err(Error::new_message("OOM todo change to code?")),
                    MprintfError::Nul(err) => Err(err.into()),
                }
            }
        };
        unsafe {
            (*self.index_info).idxStr = idxstr;
            (*self.index_info).needToFreeIdxStr = 1;
        }
        Ok(())
    }
    pub fn set_estimated_rows(&mut self, value: i64) {
        unsafe {
            (*self.index_info).estimatedRows = value;
        }
    }

    /// "The estimatedCost field should be set to the estimated number of disk
    /// access operations required to execute this query against the virtual table."
    /// <https://www.sqlite.org/vtab.html#outputs>
    pub fn set_estimated_cost(&mut self, value: f64) {
        unsafe {
            (*self.index_info).estimatedCost = value;
        }
    }
    // TODO ORDER BY

    // TODO the u64 by itself isn't very useful - offer a func that does the
    // manually bitshifting/checks internally.
    pub fn columns_used(&self) -> u64 {
        unsafe { (*self.index_info).colUsed }
    }
    pub fn distinct(&self) -> i32 {
        unsafe { sqlite3ext_vtab_distinct(self.index_info) }
    }
    // TODO idxFlags
}

/// Wraps the raw sqlite3_index_constraint and sqlite3_index_constraint_usage
/// C structs for ergonomic use in Rust.
#[derive(Debug)]
pub struct Constraint {
    pub constraint: sqlite3_index_info_sqlite3_index_constraint,
    pub usage: *mut sqlite3_index_info_sqlite3_index_constraint_usage,
}

impl Constraint {
    pub fn column_idx(&self) -> i32 {
        (self.constraint).iColumn
    }

    pub fn usable(&self) -> bool {
        (self.constraint).usable != 0
    }
    pub fn op(&self) -> Option<ConstraintOperator> {
        operator((self.constraint).op)
    }

    pub fn set_argv_index(&mut self, i: i32) {
        unsafe { (*self.usage).argvIndex = i };
    }
    pub fn set_omit(&mut self, value: bool) {
        unsafe { (*self.usage).omit = u8::from(value) }
    }
}

#[derive(Debug)]
pub enum OrderByDirection {
    Ascending,
    Descending,
}
#[derive(Debug)]
pub struct OrderBy {
    order_by: sqlite3_index_info_sqlite3_index_orderby,
}
impl OrderBy {
    pub fn icolumn(&self) -> i32 {
        (self.order_by).iColumn
    }
    pub fn direction(&self) -> OrderByDirection {
        if (self.order_by).desc == 1 {
            OrderByDirection::Descending
        } else {
            OrderByDirection::Ascending
        }
    }
}

/// Possible errors to return in xBestIndex.
pub enum BestIndexError {
    /// Returns SQLITE_CONSTRAINT. See <https://www.sqlite.org/vtab.html#return_value>
    Constraint,
    Error,
}
#[repr(transparent)]
struct Module<'vtab, T: VTab<'vtab>> {
    base: sqlite3_module,
    phantom: PhantomData<&'vtab T>,
}

unsafe impl<'vtab, T: VTab<'vtab>> Send for Module<'vtab, T> {}
unsafe impl<'vtab, T: VTab<'vtab>> Sync for Module<'vtab, T> {}

/// Define a table function on the given sqlite3 database.
/// "Table function" is the same as "eponymous-only" virtual table
/// described at <https://www.sqlite.org/vtab.html#eponymous_only_virtual_tables>
pub fn define_table_function<'vtab, T: VTab<'vtab> + 'vtab>(
    db: *mut sqlite3,
    name: &str,
    aux: Option<T::Aux>,
) -> Result<()> {
    let m = &Module {
        base: sqlite3_module {
            iVersion: 2,
            xCreate: None,
            xConnect: Some(rust_connect::<T>),
            xBestIndex: Some(rust_best_index::<T>),
            xDisconnect: Some(rust_disconnect::<T>),
            xDestroy: Some(rust_destroy::<T>),
            xOpen: Some(rust_open::<T>),
            xClose: Some(rust_close::<T::Cursor>),
            xFilter: Some(rust_filter::<T::Cursor>),
            xNext: Some(rust_next::<T::Cursor>),
            xEof: Some(rust_eof::<T::Cursor>),
            xColumn: Some(rust_column::<T::Cursor>),
            xRowid: Some(rust_rowid::<T::Cursor>),
            xUpdate: None,
            xBegin: None,
            xSync: None,
            xCommit: None,
            xRollback: None,
            xFindFunction: None,
            xRename: None,
            xSavepoint: None,
            xRelease: None,
            xRollbackTo: None,
            xShadowName: None,
        },
        phantom: PhantomData::<&'vtab T>,
    };
    let cname = CString::new(name)?;
    let p_app = match aux {
        Some(aux) => {
            let boxed_aux: *mut T::Aux = Box::into_raw(Box::new(aux));
            boxed_aux.cast::<c_void>()
        }
        None => ptr::null_mut(),
    };
    let result = unsafe {
        sqlite3ext_create_module_v2(
            db,
            cname.as_ptr(),
            &m.base,
            p_app,
            Some(destroy_aux::<T::Aux>),
        )
    };
    if result != SQLITE_OKAY {
        return Err(Error::new(ErrorKind::TableFunction(result)));
    }
    Ok(())
}
pub fn define_table_function_with_find<'vtab, T: VTabFind<'vtab> + 'vtab>(
    db: *mut sqlite3,
    name: &str,
    aux: Option<T::Aux>,
) -> Result<()> {
    let m = &Module {
        base: sqlite3_module {
            iVersion: 2,
            xCreate: None,
            xConnect: Some(rust_connect::<T>),
            xBestIndex: Some(rust_best_index::<T>),
            xDisconnect: Some(rust_disconnect::<T>),
            xDestroy: Some(rust_destroy::<T>),
            xOpen: Some(rust_open::<T>),
            xClose: Some(rust_close::<T::Cursor>),
            xFilter: Some(rust_filter::<T::Cursor>),
            xNext: Some(rust_next::<T::Cursor>),
            xEof: Some(rust_eof::<T::Cursor>),
            xColumn: Some(rust_column::<T::Cursor>),
            xRowid: Some(rust_rowid::<T::Cursor>),
            xUpdate: None,
            xBegin: None,
            xSync: None,
            xCommit: None,
            xRollback: None,
            xFindFunction: Some(rust_find_function::<T>),
            xRename: None,
            xSavepoint: None,
            xRelease: None,
            xRollbackTo: None,
            xShadowName: None,
        },
        phantom: PhantomData::<&'vtab T>,
    };
    let cname = CString::new(name)?;
    let p_app = match aux {
        Some(aux) => {
            let boxed_aux: *mut T::Aux = Box::into_raw(Box::new(aux));
            boxed_aux.cast::<c_void>()
        }
        None => ptr::null_mut(),
    };
    let result = unsafe {
        sqlite3ext_create_module_v2(
            db,
            cname.as_ptr(),
            &m.base,
            p_app,
            Some(destroy_aux::<T::Aux>),
        )
    };
    if result != SQLITE_OKAY {
        return Err(Error::new(ErrorKind::TableFunction(result)));
    }
    Ok(())
}

// source: https://github.com/rusqlite/rusqlite/blob/12a6d3c1b1bdd58ca7103619b8a133e76d30decd/src/vtab/mod.rs#L931
unsafe extern "C" fn destroy_aux<T>(p: *mut c_void) {
    if !p.is_null() {
        drop(Box::from_raw(p.cast::<T>()));
    }
}

/// Define a virtual table on the sqlite3 database connection. Optionally
/// pass in an auxillary object, which
pub fn define_virtual_table<'vtab, T: VTab<'vtab> + 'vtab>(
    db: *mut sqlite3,
    name: &str,
    aux: Option<T::Aux>,
) -> Result<()> {
    let m = &Module {
        base: sqlite3_module {
            iVersion: 2,
            xCreate: Some(rust_create::<T>),
            xConnect: Some(rust_connect::<T>),
            xBestIndex: Some(rust_best_index::<T>),
            xDisconnect: Some(rust_disconnect::<T>),
            xDestroy: Some(rust_destroy::<T>),
            xOpen: Some(rust_open::<T>),
            xClose: Some(rust_close::<T::Cursor>),
            xFilter: Some(rust_filter::<T::Cursor>),
            xNext: Some(rust_next::<T::Cursor>),
            xEof: Some(rust_eof::<T::Cursor>),
            xColumn: Some(rust_column::<T::Cursor>),
            xRowid: Some(rust_rowid::<T::Cursor>),
            xUpdate: None,
            xBegin: None,
            xSync: None,
            xCommit: None,
            xRollback: None,
            xFindFunction: None,
            xRename: None,
            xSavepoint: None,
            xRelease: None,
            xRollbackTo: None,
            xShadowName: None,
        },
        phantom: PhantomData::<&'vtab T>,
    };
    let cname = CString::new(name)?;
    let app_pointer = match aux {
        Some(aux) => Box::into_raw(Box::new(aux)).cast::<c_void>(),
        None => ptr::null_mut(),
    };
    let result = unsafe {
        sqlite3ext_create_module_v2(
            db,
            cname.as_ptr(),
            &m.base,
            app_pointer,
            Some(destroy_aux::<T::Aux>),
        )
    };
    if result != SQLITE_OKAY {
        return Err(Error::new(ErrorKind::TableFunction(result)));
    }
    Ok(())
}

pub fn define_virtual_table_writeable<'vtab, T: VTabWriteable<'vtab> + 'vtab>(
    db: *mut sqlite3,
    name: &str,
    aux: Option<T::Aux>,
) -> Result<()> {
    let m = &Module {
        base: sqlite3_module {
            iVersion: 2,
            xCreate: Some(rust_create::<T>),
            xConnect: Some(rust_connect::<T>),
            xBestIndex: Some(rust_best_index::<T>),
            xDisconnect: Some(rust_disconnect::<T>),
            xDestroy: Some(rust_destroy::<T>),
            xOpen: Some(rust_open::<T>),
            xClose: Some(rust_close::<T::Cursor>),
            xFilter: Some(rust_filter::<T::Cursor>),
            xNext: Some(rust_next::<T::Cursor>),
            xEof: Some(rust_eof::<T::Cursor>),
            xColumn: Some(rust_column::<T::Cursor>),
            xRowid: Some(rust_rowid::<T::Cursor>),
            xUpdate: Some(rust_update::<T>),
            xBegin: None,    //Some(rust_begin::<T>),
            xSync: None,     //Some(rust_sync::<T>),
            xCommit: None,   //Some(rust_commit::<T>),
            xRollback: None, //Some(rust_rollback::<T>),
            xFindFunction: None,
            xRename: None,
            xSavepoint: None,
            xRelease: None,
            xRollbackTo: None,
            xShadowName: None,
        },
        phantom: PhantomData::<&'vtab T>,
    };
    let cname = CString::new(name)?;
    let p_app = match aux {
        Some(aux) => {
            let boxed_aux: *mut T::Aux = Box::into_raw(Box::new(aux));
            boxed_aux.cast::<c_void>()
        }
        None => ptr::null_mut(),
    };
    let result = unsafe {
        sqlite3ext_create_module_v2(
            db,
            cname.as_ptr(),
            &m.base,
            p_app,
            Some(destroy_aux::<T::Aux>),
        )
    };
    if result != SQLITE_OKAY {
        return Err(Error::new(ErrorKind::TableFunction(result)));
    }
    Ok(())
}

pub fn define_virtual_table_writeable_with_transactions<
    'vtab,
    T: VTabWriteableWithTransactions<'vtab> + 'vtab,
>(
    db: *mut sqlite3,
    name: &str,
    aux: Option<T::Aux>,
) -> Result<()> {
    let m = &Module {
        base: sqlite3_module {
            iVersion: 2,
            xCreate: Some(rust_create::<T>),
            xConnect: Some(rust_connect::<T>),
            xBestIndex: Some(rust_best_index::<T>),
            xDisconnect: Some(rust_disconnect::<T>),
            xDestroy: Some(rust_destroy::<T>),
            xOpen: Some(rust_open::<T>),
            xClose: Some(rust_close::<T::Cursor>),
            xFilter: Some(rust_filter::<T::Cursor>),
            xNext: Some(rust_next::<T::Cursor>),
            xEof: Some(rust_eof::<T::Cursor>),
            xColumn: Some(rust_column::<T::Cursor>),
            xRowid: Some(rust_rowid::<T::Cursor>),
            xUpdate: Some(rust_update::<T>),
            xBegin: Some(rust_begin::<T>),
            xSync: Some(rust_sync::<T>),
            xCommit: Some(rust_commit::<T>),
            xRollback: Some(rust_rollback::<T>),
            xFindFunction: None,
            xRename: None,
            xSavepoint: None,
            xRelease: None,
            xRollbackTo: None,
            xShadowName: None,
        },
        phantom: PhantomData::<&'vtab T>,
    };
    let cname = CString::new(name)?;
    let p_app = match aux {
        Some(aux) => {
            let boxed_aux: *mut T::Aux = Box::into_raw(Box::new(aux));
            boxed_aux.cast::<c_void>()
        }
        None => ptr::null_mut(),
    };
    let result = unsafe {
        sqlite3ext_create_module_v2(
            db,
            cname.as_ptr(),
            &m.base,
            p_app,
            Some(destroy_aux::<T::Aux>),
        )
    };
    if result != SQLITE_OKAY {
        return Err(Error::new(ErrorKind::TableFunction(result)));
    }
    Ok(())
}

pub fn define_virtual_table_writeablex<'vtab, T: VTabWriteable<'vtab> + 'vtab>(
    db: *mut sqlite3,
    name: &str,
    aux: Option<T::Aux>,
) -> Result<()> {
    let m = &Module {
        base: sqlite3_module {
            iVersion: 2,
            xCreate: None,
            xConnect: Some(rust_connect::<T>),
            xBestIndex: Some(rust_best_index::<T>),
            xDisconnect: Some(rust_disconnect::<T>),
            xDestroy: Some(rust_destroy::<T>),
            xOpen: Some(rust_open::<T>),
            xClose: Some(rust_close::<T::Cursor>),
            xFilter: Some(rust_filter::<T::Cursor>),
            xNext: Some(rust_next::<T::Cursor>),
            xEof: Some(rust_eof::<T::Cursor>),
            xColumn: Some(rust_column::<T::Cursor>),
            xRowid: Some(rust_rowid::<T::Cursor>),
            xUpdate: Some(rust_update::<T>),
            xBegin: None,    //Some(rust_begin::<T>),
            xSync: None,     //Some(rust_sync::<T>),
            xCommit: None,   //Some(rust_commit::<T>),
            xRollback: None, //Some(rust_rollback::<T>),
            xFindFunction: None,
            xRename: None,
            xSavepoint: None,
            xRelease: None,
            xRollbackTo: None,
            xShadowName: None,
        },
        phantom: PhantomData::<&'vtab T>,
    };
    let cname = CString::new(name)?;
    let p_app = match aux {
        Some(aux) => {
            let boxed_aux: *mut T::Aux = Box::into_raw(Box::new(aux));
            boxed_aux.cast::<c_void>()
        }
        None => ptr::null_mut(),
    };
    let result = unsafe {
        sqlite3ext_create_module_v2(
            db,
            cname.as_ptr(),
            &m.base,
            p_app,
            Some(destroy_aux::<T::Aux>),
        )
    };
    if result != SQLITE_OKAY {
        return Err(Error::new(ErrorKind::TableFunction(result)));
    }
    Ok(())
}

pub trait VTab<'vtab>: Sized {
    type Aux;
    type Cursor: VTabCursor;

    fn create(
        db: *mut sqlite3,
        aux: Option<&Self::Aux>,
        args: VTabArguments,
    ) -> Result<(String, Self)> {
        Self::connect(db, aux, args)
    }

    fn connect(
        db: *mut sqlite3,
        aux: Option<&Self::Aux>,
        args: VTabArguments,
    ) -> Result<(String, Self)>;

    fn best_index(&self, info: IndexInfo) -> core::result::Result<(), BestIndexError>;

    fn open(&'vtab mut self) -> Result<Self::Cursor>;

    fn destroy(&self) -> Result<()> {
        Ok(())
    }
}

pub trait VTabWriteable<'vtab>: VTab<'vtab> {
    fn update(&'vtab mut self, operation: UpdateOperation, p_rowid: *mut i64) -> Result<()>;
}
pub trait VTabFind<'vtab>: VTab<'vtab> {
    // TODO should be able to return SQLITE_INDEX_CONSTRAINT_FUNCTION or more
    fn find_function(
        &'vtab mut self,
        argc: i32,
        name: &str,
    ) -> Option<unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value)>;
}

pub trait VTabWriteableWithTransactions<'vtab>: VTabWriteable<'vtab> {
    fn begin(&'vtab mut self) -> Result<()>;
    fn sync(&'vtab mut self) -> Result<()>;
    fn commit(&'vtab mut self) -> Result<()>;
    fn rollback(&'vtab mut self) -> Result<()>;
}

pub trait VTabWriteableNestedTransactions<'vtab>: VTabWriteable<'vtab> {
    fn savepoint(&'vtab mut self, id: c_int) -> Result<()>;
    fn release(&'vtab mut self, id: c_int) -> Result<()>;
    fn rollback_to(&'vtab mut self, id: c_int) -> Result<()>;
}

pub trait VTabCursor: Sized {
    fn filter(
        &mut self,
        idx_num: c_int,
        idx_str: Option<&str>,
        values: &[*mut sqlite3_value],
    ) -> Result<()>;
    fn next(&mut self) -> Result<()>;
    fn eof(&self) -> bool;
    fn column(&self, ctx: *mut sqlite3_context, i: c_int) -> Result<()>;
    fn rowid(&self) -> Result<i64>;
}

use std::ffi::CStr;

/// Represents all the arguments given to the virtual table implementation
/// during [xCreate](https://www.sqlite.org/vtab.html#xcreate), from the
/// underlying `argv`/`argc` strings. Parsed to be more easily readable.
///
/// You most likely want to pass in `.arguments` into
/// [vtab_argparse::parse_argument](crate::vtab_argparse::parse_argument).
pub struct VTabArguments {
    /// Name of the module being invoked, the argument in the USING clause.
    /// Example: `"CREATE VIRTUAL TABLE xxx USING custom_vtab"` would have
    /// a `module_name` of `"custom_vtab"`.
    /// Sourced from `argv[0]`
    pub module_name: String,
    /// Name of the database where the virtual table will be created,
    /// typically `"main"` or `"temp"` or another name from an
    /// [`ATTACH`'ed database](https://www.sqlite.org/lang_attach.html).
    /// Sourced from `argv[1]`
    pub database_name: String,

    /// Name of the table being created.
    /// Example: `"CREATE VIRTUAL TABLE xxx USING custom_vtab"` would
    /// have a `table_name` of `"xxx"`.
    /// Sourced from `argv[2]`
    pub table_name: String,
    /// The remaining arguments given in the constructor of the virtual
    /// table, inside `CREATE VIRTUAL TABLE xxx USING custom_vtab(...)`.
    /// Sourced from `argv[3:]`
    pub arguments: Vec<String>,
}

fn c_string_to_string(c: &*const c_char) -> std::result::Result<String, Utf8Error> {
    let bytes = unsafe { CStr::from_ptr(c.to_owned()).to_bytes() };
    Ok(std::str::from_utf8(bytes)?.to_string())
}
fn process_create_args(
    argc: c_int,
    argv: *const *const c_char,
) -> std::result::Result<VTabArguments, Utf8Error> {
    let raw_args = unsafe { slice::from_raw_parts(argv, argc as usize) };
    let mut args = Vec::with_capacity(argc as usize);
    for arg in raw_args {
        args.push(c_string_to_string(arg)?);
    }

    // SQLite guarantees that argv[0-2] will be filled, hence the .expects() -
    // If SQLite is wrong, then may god save our souls
    let module_name = args
        .get(0)
        .expect("argv[0] should be the name of the module")
        .to_owned();
    let database_name = args
        .get(1)
        .expect("argv[1] should be the name of the database the module is in")
        .to_owned();
    let table_name = args
        .get(2)
        .expect("argv[2] should be the name of the virtual table")
        .to_owned();
    let arguments = &args[3..];

    Ok(VTabArguments {
        module_name,
        database_name,
        table_name,
        arguments: arguments.to_vec(),
    })
}
/// <https://www.sqlite.org/vtab.html#the_xcreate_method>
// TODO set error message properly
unsafe extern "C" fn rust_create<'vtab, T>(
    db: *mut sqlite3,
    aux: *mut c_void,
    argc: c_int,
    argv: *const *const c_char,
    pp_vtab: *mut *mut sqlite3_vtab,
    err_msg: *mut *mut c_char,
) -> c_int
where
    T: VTab<'vtab>,
{
    let aux = aux.cast::<T::Aux>();
    let args = match process_create_args(argc, argv) {
        Ok(args) => args,
        Err(_) => return SQLITE_ERROR,
    };
    match T::create(db, aux.as_ref(), args) {
        Ok((sql, vtab)) => match CString::new(sql) {
            Ok(c_sql) => {
                let rc = sqlitex_declare_vtab(db, c_sql.as_ptr());
                if rc == SQLITE_OKAY {
                    let boxed_vtab: *mut T = Box::into_raw(Box::new(vtab));
                    *pp_vtab = boxed_vtab.cast::<sqlite3_vtab>();
                    SQLITE_OKAY
                } else {
                    rc
                }
            }
            Err(_err) => SQLITE_ERROR,
        },
        Err(err) => {
            if let ErrorKind::Message(msg) = err.kind() {
                if let Ok(err) = mprintf(msg) {
                    *err_msg = err;
                }
            };
            err.code()
        }
    }
}

/// <https://www.sqlite.org/vtab.html#the_xconnect_method>
// TODO set error message properly
unsafe extern "C" fn rust_connect<'vtab, T>(
    db: *mut sqlite3,
    aux: *mut c_void,
    argc: c_int,
    argv: *const *const c_char,
    pp_vtab: *mut *mut sqlite3_vtab,
    err_msg: *mut *mut c_char,
) -> c_int
where
    T: VTab<'vtab>,
{
    let aux = aux.cast::<T::Aux>();
    let args = match process_create_args(argc, argv) {
        Ok(args) => args,
        Err(_) => return SQLITE_ERROR,
    };
    match T::connect(db, aux.as_ref(), args) {
        Ok((sql, vtab)) => match CString::new(sql) {
            Ok(c_sql) => {
                let rc = sqlitex_declare_vtab(db, c_sql.as_ptr());
                if rc == SQLITE_OKAY {
                    let boxed_vtab: *mut T = Box::into_raw(Box::new(vtab));
                    *pp_vtab = boxed_vtab.cast::<sqlite3_vtab>();
                    SQLITE_OKAY
                } else {
                    rc
                }
            }
            Err(_err) => SQLITE_ERROR,
        },
        Err(err) => {
            if let ErrorKind::Message(msg) = err.kind() {
                if let Ok(err) = mprintf(msg) {
                    *err_msg = err;
                }
            };
            err.code()
        }
    }
}

/// <https://www.sqlite.org/vtab.html#the_xbestindex_method>
// TODO set error message properly
unsafe extern "C" fn rust_best_index<'vtab, T>(
    vtab: *mut sqlite3_vtab,
    index_info: *mut sqlite3_index_info,
) -> c_int
where
    T: VTab<'vtab>,
{
    let vt = vtab.cast::<T>();
    match (*vt).best_index(IndexInfo { index_info }) {
        Ok(_) => SQLITE_OKAY,
        Err(e) => match e {
            BestIndexError::Constraint => SQLITE_CONSTRAINT,
            BestIndexError::Error => SQLITE_ERROR,
        },
    }
}

/// <https://www.sqlite.org/vtab.html#the_xdisconnect_method>
// TODO set error message properly
unsafe extern "C" fn rust_disconnect<'vtab, T>(vtab: *mut sqlite3_vtab) -> c_int
where
    T: VTab<'vtab>,
{
    if vtab.is_null() {
        return SQLITE_OKAY;
    }
    let vtab = vtab.cast::<T>();
    drop(Box::from_raw(vtab));
    SQLITE_OKAY
}

/// <https://www.sqlite.org/vtab.html#the_xdestroy_method>
// TODO set error message properly
unsafe extern "C" fn rust_destroy<'vtab, T>(vtab: *mut sqlite3_vtab) -> c_int
where
    T: VTab<'vtab>,
{
    if vtab.is_null() {
        return SQLITE_OKAY;
    }
    let vt = vtab.cast::<T>();
    match (*vt).destroy() {
        Ok(_) => SQLITE_OKAY,
        Err(err) => err.code(),
    }
}

/// <https://www.sqlite.org/vtab.html#the_xopen_method>
// TODO set error message properly
unsafe extern "C" fn rust_open<'vtab, T: 'vtab>(
    vtab: *mut sqlite3_vtab,
    pp_cursor: *mut *mut sqlite3_vtab_cursor,
) -> c_int
where
    T: VTab<'vtab>,
{
    let vt = vtab.cast::<T>();
    match (*vt).open() {
        Ok(cursor) => {
            let boxed_cursor: *mut T::Cursor = Box::into_raw(Box::new(cursor));
            *pp_cursor = boxed_cursor.cast::<sqlite3_vtab_cursor>();
            SQLITE_OKAY
        }
        Err(err) => err.code(),
    }
}

// https://www.sqlite.org/vtab.html#the_xupdate_method
#[derive(Debug)]
pub enum UpdateOperation<'a> {
    Delete(&'a *mut sqlite3_value),
    Insert {
        values: &'a [*mut sqlite3_value],
        rowid: Option<&'a *mut sqlite3_value>,
    },
    Update {
        _values: &'a [*mut sqlite3_value],
    },
}

fn determine_update_operation<'a>(
    argc: c_int,
    argv: *mut *mut sqlite3_value,
) -> UpdateOperation<'a> {
    let args = unsafe { slice::from_raw_parts(argv, argc as usize) };

    // "The value of argc will be 1 for a pure delete operation"
    if argc == 1 {
        return UpdateOperation::Delete(
            args.get(0)
                .expect("argv[0] should be non-null for DELETE operations"),
        );
    }

    let argv0 = args
        .get(0)
        .expect("argv[0] should be defined on all non-delete operations");
    let argv1 = args
        .get(1)
        .expect("argv[1] should be defined on all non-delete operations");

    //  argc > 1 AND argv[0] = NULL
    // "INSERT: A new row is inserted with column values taken from argv[2] and following."
    if value_type(argv1) == ValueType::Null {
        let rowid = if value_type(argv0) == ValueType::Null {
            None
        } else {
            Some(argv1)
        };
        UpdateOperation::Insert {
            values: args
                .get(2..)
                .expect("argv[0-1] should be defined on INSERT operations"),
            rowid,
        }
    }
    // argc > 1 AND argv[0] ≠ NULL AND argv[0] = argv[1]
    // "UPDATE: The row with rowid or PRIMARY KEY argv[0] is updated with new values in argv[2] and following parameters.'
    else if argv0 == argv1 {
        UpdateOperation::Update {
            _values: args
                .get(2..)
                .expect("argv[0-1] should be defined on INSERT operations"),
        }
    }
    //argc > 1 AND argv[0] ≠ NULL AND argv[0] ≠ argv[1]
    // "UPDATE with rowid or PRIMARY KEY change: The row with rowid or PRIMARY KEY argv[0] is updated with
    // the rowid or PRIMARY KEY in argv[1] and new values in argv[2] and following parameters. "
    // what the hell does this even mean
    else if true {
        todo!();
    } else {
        todo!("some unsupported update operation?")
    }
}
/// <https://www.sqlite.org/vtab.html#the_xupdate_method>
// TODO set error message properly
unsafe extern "C" fn rust_update<'vtab, T: 'vtab>(
    vtab: *mut sqlite3_vtab,
    argc: c_int,
    argv: *mut *mut sqlite3_value,
    p_rowid: *mut i64,
) -> c_int
where
    T: VTabWriteable<'vtab>,
{
    let vt = vtab.cast::<T>();

    match (*vt).update(determine_update_operation(argc, argv), p_rowid) {
        Ok(_) => SQLITE_OKAY,
        Err(err) => err.code(),
    }
}

/// <https://www.sqlite.org/vtab.html#the_xbegin_method>
// TODO set error message properly
unsafe extern "C" fn rust_begin<'vtab, T: 'vtab>(vtab: *mut sqlite3_vtab) -> c_int
where
    T: VTabWriteableWithTransactions<'vtab>,
{
    let vt = vtab.cast::<T>();
    match (*vt).begin() {
        Ok(_) => SQLITE_OKAY,
        Err(err) => err.code(),
    }
}

/// <https://www.sqlite.org/vtab.html#the_xsync_method>
// TODO set error message properly
unsafe extern "C" fn rust_sync<'vtab, T: 'vtab>(vtab: *mut sqlite3_vtab) -> c_int
where
    T: VTabWriteableWithTransactions<'vtab>,
{
    let vt = vtab.cast::<T>();
    match (*vt).sync() {
        Ok(_) => SQLITE_OKAY,
        Err(err) => err.code(),
    }
}

/// <https://www.sqlite.org/vtab.html#the_xrollback_method>
// TODO set error message properly
unsafe extern "C" fn rust_rollback<'vtab, T: 'vtab>(vtab: *mut sqlite3_vtab) -> c_int
where
    T: VTabWriteableWithTransactions<'vtab>,
{
    let vt = vtab.cast::<T>();
    match (*vt).rollback() {
        Ok(_) => SQLITE_OKAY,
        Err(err) => err.code(),
    }
}

/// <https://www.sqlite.org/vtab.html#the_xcommit_method>
// TODO set error message properly
unsafe extern "C" fn rust_commit<'vtab, T: 'vtab>(vtab: *mut sqlite3_vtab) -> c_int
where
    T: VTabWriteableWithTransactions<'vtab>,
{
    let vt = vtab.cast::<T>();
    match (*vt).commit() {
        Ok(_) => SQLITE_OKAY,
        Err(err) => err.code(),
    }
}

/// <https://www.sqlite.org/vtab.html#the_xfindfunction_method>
// TODO set error message properly
unsafe extern "C" fn rust_find_function<'vtab, T: 'vtab>(
    vtab: *mut sqlite3_vtab,
    n_arg: c_int,
    name: *const c_char,
    p_xfunc: *mut Option<unsafe extern "C" fn(*mut sqlite3_context, i32, *mut *mut sqlite3_value)>,
    p_p_arg: *mut *mut c_void,
) -> c_int
where
    T: VTabFind<'vtab>,
{
    let vt = vtab.cast::<T>();
    let name = CStr::from_ptr(name).to_bytes();
    let name = std::str::from_utf8_unchecked(name);

    match (*vt).find_function(n_arg, name) {
        Some(function) => {
            (*p_xfunc) = Some(function);
            1 // TODO give option to return non 1 funcs
        }
        None => 0,
    }
}

/// <https://www.sqlite.org/vtab.html#the_xclose_method>
// TODO set error message properly
unsafe extern "C" fn rust_close<C>(cursor: *mut sqlite3_vtab_cursor) -> c_int
where
    C: VTabCursor,
{
    let cr = cursor.cast::<C>();
    drop(Box::from_raw(cr));
    SQLITE_OKAY
}

/// <https://www.sqlite.org/vtab.html#the_xfilter_method>
// TODO set error message properly
unsafe extern "C" fn rust_filter<C>(
    cursor: *mut sqlite3_vtab_cursor,
    idx_num: c_int,
    idx_str: *const c_char,
    argc: c_int,
    argv: *mut *mut sqlite3_value,
) -> c_int
where
    C: VTabCursor,
{
    use std::str;
    let idx_name = if idx_str.is_null() {
        None
    } else {
        let c_slice = CStr::from_ptr(idx_str).to_bytes();
        Some(str::from_utf8_unchecked(c_slice))
    };
    let cr = cursor.cast::<C>();
    //cursor_error(cursor, )
    let args = slice::from_raw_parts_mut(argv, argc as usize);
    match (*cr).filter(idx_num, idx_name, args) {
        Ok(()) => SQLITE_OKAY,
        Err(err) => {
            if let ErrorKind::Message(msg) = err.kind() {
                if let Ok(err) = mprintf(msg) {
                    (*(*cursor).pVtab).zErrMsg = err;
                }
            };
            err.code()
        }
    }
}

/// <https://www.sqlite.org/vtab.html#the_xnext_method>
// TODO set error message properly
unsafe extern "C" fn rust_next<C>(cursor: *mut sqlite3_vtab_cursor) -> c_int
where
    C: VTabCursor,
{
    let cr = cursor.cast::<C>();
    //cursor_error(cursor, (*cr).next())
    match (*cr).next() {
        Ok(()) => SQLITE_OKAY,
        Err(err) => {
            if let ErrorKind::Message(msg) = err.kind() {
                if let Ok(err) = mprintf(msg) {
                    (*(*cursor).pVtab).zErrMsg = err;
                }
            };
            err.code()
        }
    }
}

/// <https://www.sqlite.org/vtab.html#the_xeof_method>
// TODO set error message properly
unsafe extern "C" fn rust_eof<C>(cursor: *mut sqlite3_vtab_cursor) -> c_int
where
    C: VTabCursor,
{
    let cr = cursor.cast::<C>();
    (*cr).eof() as c_int
}

/// <https://www.sqlite.org/vtab.html#the_xcolumn_method>
// TODO set error message properly
unsafe extern "C" fn rust_column<C>(
    cursor: *mut sqlite3_vtab_cursor,
    ctx: *mut sqlite3_context,
    i: c_int,
) -> c_int
where
    C: VTabCursor,
{
    let cr = cursor.cast::<C>();
    //result_error(ctx, (*cr).column(&mut ctxt, i))
    match (*cr).column(ctx, i) {
        Ok(()) => SQLITE_OKAY,
        Err(err) => {
            if let ErrorKind::Message(msg) = err.kind() {
                if let Ok(err) = mprintf(msg) {
                    (*(*cursor).pVtab).zErrMsg = err;
                }
            };
            err.code()
        }
    }
}

/// "A successful invocation of this method will cause *pRowid to be filled with the rowid of row
/// that the virtual table cursor pCur is currently pointing at.
/// This method returns SQLITE_OKAY on success. It returns an appropriate error code on failure."
/// <https://www.sqlite.org/vtab.html#the_xrowid_method>
// TODO set error message properly
unsafe extern "C" fn rust_rowid<C>(cursor: *mut sqlite3_vtab_cursor, p_rowid: *mut i64) -> c_int
where
    C: VTabCursor,
{
    let cr = cursor.cast::<C>();
    match (*cr).rowid() {
        Ok(rowid) => {
            *p_rowid = rowid;
            SQLITE_OKAY
        }
        Err(err) => err.code(),
    }
}
