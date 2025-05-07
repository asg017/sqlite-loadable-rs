use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::{c_void, CStr, CString};
use std::io::ErrorKind;
use std::mem::{size_of, ManuallyDrop, MaybeUninit};
use std::ops::Range;
use std::os::raw::{c_char, c_int};
use std::pin::Pin;
use std::ptr::null_mut;
use std::slice;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use strum::FromRepr;

#[derive(Debug, Clone, PartialEq)]
pub struct OpenOptions {
    /// The object type that is being opened.
    pub kind: OpenKind,

    /// The access an object is opened with.
    pub access: OpenAccess,
}

/*
SQLITE_OPEN_MEMORY: i32 = 128;
SQLITE_OPEN_MAIN_DB: i32 = 256;
SQLITE_OPEN_TEMP_DB: i32 = 512;
SQLITE_OPEN_TRANSIENT_DB: i32 = 1024;
SQLITE_OPEN_MAIN_JOURNAL: i32 = 2048;
SQLITE_OPEN_TEMP_JOURNAL: i32 = 4096;
SQLITE_OPEN_SUBJOURNAL: i32 = 8192;
SQLITE_OPEN_SUPER_JOURNAL: i32 = 16384;
SQLITE_OPEN_NOMUTEX: i32 = 32768;
SQLITE_OPEN_FULLMUTEX: i32 = 65536;
SQLITE_OPEN_SHAREDCACHE: i32 = 131072;
SQLITE_OPEN_PRIVATECACHE: i32 = 262144;
SQLITE_OPEN_WAL: i32 = 524288;
SQLITE_OPEN_NOFOLLOW: i32 = 16777216;
SQLITE_OPEN_MASTER_JOURNAL: i32 = 16384;
*/

/// The object type that is being opened.
#[derive(FromRepr, Debug, Clone, Copy, PartialEq)]
#[repr(i32)]
pub enum OpenKind {
    MainDb = 256,         // SQLITE_OPEN_MAIN_DB,
    MainJournal = 2048,   // SQLITE_OPEN_MAIN_JOURNAL
    TempDb = 512,         // SQLITE_OPEN_TEMP_DB
    TempJournal = 4096,   // SQLITE_OPEN_TEMP_JOURNAL
    TransientDb = 1024,   // SQLITE_OPEN_TRANSIENT_DB
    SubJournal = 8192,    // SQLITE_OPEN_SUBJOURNAL
    SuperJournal = 16384, // SQLITE_OPEN_SUPER_JOURNAL / SQLITE_OPEN_MASTER_JOURNAL
    Wal = 524288,         // SQLITE_OPEN_WAL
}

/*
pub const SQLITE_OPEN_READONLY: i32 = 1;
pub const SQLITE_OPEN_READWRITE: i32 = 2;
pub const SQLITE_OPEN_CREATE: i32 = 4;
*/
/// The access an object is opened with.
#[derive(FromRepr, Debug, Clone, Copy, PartialEq)]
#[repr(i32)]
pub enum OpenAccess {
    /// Read access.
    Read = 1,

    /// Write access (includes read access).
    Write = 2,

    /// Create the file if it does not exist (includes write and read access).
    Create = 6,

    /// Create the file, but throw if it it already exist (includes write and read access).
    CreateNewThrowIfExists = 8,
}
