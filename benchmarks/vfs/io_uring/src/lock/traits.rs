#![allow(clippy::question_mark)]
//! Create a custom SQLite virtual file system by implementing the [Vfs] trait and registering it
//! using [register].

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

use super::kind::LockKind;
use super::wal::{WalIndex, WalConnection};


#[derive(Debug, Clone, PartialEq)]
pub struct OpenOptions {
    /// The object type that is being opened.
    pub kind: OpenKind,

    /// The access an object is opened with.
    pub access: OpenAccess,

    /// The file should be deleted when it is closed.
    delete_on_close: bool,
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
    MainDb = 256, // SQLITE_OPEN_MAIN_DB,
    MainJournal = 2048, // SQLITE_OPEN_MAIN_JOURNAL
    TempDb = 512, // SQLITE_OPEN_TEMP_DB
    TempJournal = 4096, // SQLITE_OPEN_TEMP_JOURNAL
    TransientDb = 1024, // SQLITE_OPEN_TRANSIENT_DB
    SubJournal = 8192, // SQLITE_OPEN_SUBJOURNAL
    SuperJournal = 16384, // SQLITE_OPEN_SUPER_JOURNAL / SQLITE_OPEN_MASTER_JOURNAL
    Wal = 524288, // SQLITE_OPEN_WAL
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
    Create = 4,

    /// Create the file, but throw if it it already exist (includes write and read access).
    CreateNewThrowIfExists = 8, // TODO figure out how to support on io_uring
}


/// A file opened by [Vfs].
pub trait DatabaseHandle: Sync {
    /// An optional trait used to store a WAL (write-ahead log).
    type WalIndex: WalIndex;

    /// Lock the database. Returns whether the requested lock could be acquired.
    /// Locking sequence:
    /// - The lock is never moved from [LockKind::None] to anything higher than [LockKind::Shared].
    /// - A [LockKind::Pending] is never requested explicitly.
    /// - A [LockKind::Shared] is always held when a [LockKind::Reserved] lock is requested
    fn lock(&mut self, lock: LockKind) -> Result<bool, std::io::Error>;

    /// Unlock the database.
    fn unlock(&mut self, lock: LockKind) -> Result<bool, std::io::Error> {
        self.lock(lock)
    }

    /// Check if the database this handle points to holds a [LockKind::Reserved],
    /// [LockKind::Pending] or [LockKind::Exclusive] lock.
    fn reserved(&mut self) -> Result<bool, std::io::Error>;

    /// Return the current [LockKind] of the this handle.
    fn current_lock(&self) -> Result<LockKind, std::io::Error>;

    fn set_chunk_size(&self, _chunk_size: usize) -> Result<(), std::io::Error> {
        Ok(())
    }

    /// Check if the underlying data of the handle got moved or deleted. When moved, the handle can
    /// still be read from, but not written to anymore.
    fn moved(&self) -> Result<bool, std::io::Error> {
        Ok(false)
    }

    fn wal_index(&self, readonly: bool) -> Result<Self::WalIndex, std::io::Error>;
}

pub trait Open: Sync {
    /// The file returned by [Vfs::open].
    type Handle: DatabaseHandle;

    /// Open the database `db` (of type `opts.kind`).
    fn open(&self, db: &str, opts: OpenOptions) -> Result<Self::Handle, std::io::Error>;

    /*
    /// Delete the database `db`.
    fn delete(&self, db: &str) -> Result<(), std::io::Error>;

    /// Check if a database `db` already exists.
    fn exists(&self, db: &str) -> Result<bool, std::io::Error>;
    */

    /// Generate and return a path for a temporary database.
    fn temporary_name(&self) -> String;

    /*
    /// Populate the `buffer` with random data.
    fn random(&self, buffer: &mut [i8]);

    /// Sleep for `duration`. Return the duration actually slept.
    fn sleep(&self, duration: Duration) -> Duration;

    /// Check access to `db`. The default implementation always returns `true`.
    fn access(&self, _db: &str, _write: bool) -> Result<bool, std::io::Error> {
        Ok(true)
    }
    */

    /// Retrieve the full pathname of a database `db`.
    fn full_pathname<'a>(&self, db: &'a str) -> Result<Cow<'a, str>, std::io::Error> {
        Ok(db.into())
    }
}