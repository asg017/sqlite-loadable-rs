pub(crate) mod kind;
pub(crate) mod file;
pub(crate) mod range;
pub(crate) mod wal;
pub(crate) mod wrapper;
pub(crate) mod traits;

use std::{fs::{self, Permissions}, ffi::CString, sync::{Mutex, Arc}, mem::MaybeUninit, collections::HashMap, pin::Pin, io::ErrorKind, os::unix::prelude::PermissionsExt};
use sqlite3ext_sys::{self, sqlite3_file, SQLITE_IOERR_LOCK, SQLITE_OK, SQLITE_IOERR_UNLOCK, SQLITE_IOERR_CHECKRESERVEDLOCK, SQLITE_BUSY};

use crate::connection::permissions;

use self::{kind::LockKind, range::RangeLock, wal::{WalConnection, WalIndex}, file::FileLock, traits::DatabaseHandle};

// TODO: add to [Vfs]?
const MAX_PATH_LENGTH: usize = 512;

struct State<V> {
    name: CString,
    vfs: Arc<V>,
    last_error: Arc<Mutex<Option<(i32, std::io::Error)>>>,
    next_id: usize,
}

#[repr(C)]
struct FileExt<V, F: DatabaseHandle> {
    vfs: Arc<V>,
    vfs_name: CString,
    db_name: String,
    file: F,
    delete_on_close: bool,
    /// The last error; shared with the VFS.
    last_error: Arc<Mutex<Option<(i32, std::io::Error)>>>,
    /// The last error number of this file/connection (not shared with the VFS).
    last_errno: i32,
    wal_index: Option<(F::WalIndex, bool)>,
    wal_index_regions: HashMap<u32, Pin<Box<[u8; 32768]>>>,
    wal_index_locks: HashMap<u8, LockKind>,
    has_exclusive_lock: bool,
    id: usize,
    chunk_size: Option<usize>,
    persist_wal: bool,
    powersafe_overwrite: bool,
}

impl<V> State<V> {
    fn set_last_error(&mut self, no: i32, err: std::io::Error) -> i32 {
        // log::error!("{} ({})", err, no);
        *(self.last_error.lock().unwrap()) = Some((no, err));
        no
    }
}

impl<V, F: DatabaseHandle> FileExt<V, F> {
    fn set_last_error(&mut self, no: i32, err: std::io::Error) -> i32 {
        // log::error!("{} ({})", err, no);
        *(self.last_error.lock().unwrap()) = Some((no, err));
        self.last_errno = no;
        no
    }
}

fn null_ptr_error() -> std::io::Error {
    std::io::Error::new(ErrorKind::Other, "received null pointer")
}

impl<V, F: DatabaseHandle> FileExt<V, F> {
    /// Lock a file.
    pub(crate) fn lock(
        // p_file: *mut sqlite3_file,
        &mut self,
        e_lock: i32,
    ) -> i32 {
        // let state = match file_state(p_file) {
        //     Ok(f) => f,
        //     Err(_) => return SQLITE_IOERR_LOCK,
        // };
        // log::trace!("[{}] lock ({})", state.id, state.db_name);

        let lock = match LockKind::from_repr(e_lock) {
            Some(lock) => lock,
            None => return SQLITE_IOERR_LOCK,
        };
        match self.file.lock(lock) {
            Ok(true) => {
                self.has_exclusive_lock = lock == LockKind::Exclusive;
                // log::trace!("[{}] lock={:?} ({})", state.id, lock, state.db_name);

                // If just acquired a exclusive database lock while not having any exclusive lock
                // on the wal index, make sure the wal index is up to date.
                if self.has_exclusive_lock {
                    let has_exclusive_wal_index = self
                        .wal_index_locks
                        .iter()
                        .any(|(_, lock)| *lock == LockKind::Exclusive);

                    if !has_exclusive_wal_index {
                        // log::trace!(
                        //     "[{}] acquired exclusive db lock, pulling wal index changes",
                        //     state.id,
                        // );

                        if let Some((wal_index, _)) = self.wal_index.as_mut() {
                            for (region, data) in &mut self.wal_index_regions {
                                if let Err(err) = wal_index.pull(*region as u32, data) {
                                    // log::error!(
                                    //     "[{}] pulling wal index changes failed: {}",
                                    //     state.id,
                                    //     err
                                    // )
                                }
                            }
                        }
                    }
                }

                SQLITE_OK
            }
            Ok(false) => {
                // log::trace!(
                //     "[{}] busy (denied {:?}) ({})",
                //     state.id,
                //     lock,
                //     state.db_name
                // );
                SQLITE_BUSY
            }
            Err(err) => self.set_last_error(SQLITE_IOERR_LOCK, err),
        }
    }

    /// Unlock a file.
    pub(crate) fn unlock(
        // p_file: *mut sqlite3_file,
        &mut self,
        e_lock: i32,
    ) -> i32 {
        // let state = match file_state(p_file) {
        //     Ok(f) => f,
        //     Err(_) => return SQLITE_IOERR_UNLOCK,
        // };
        // log::trace!("[{}] unlock ({})", state.id, state.db_name);

        let lock = match LockKind::from_repr(e_lock) {
            Some(lock) => lock,
            None => return SQLITE_IOERR_UNLOCK,
        };
        match self.file.unlock(lock) {
            Ok(true) => {
                self.has_exclusive_lock = lock == LockKind::Exclusive;
                // log::trace!("[{}] unlock={:?} ({})", state.id, lock, state.db_name);
                SQLITE_OK
            }
            Ok(false) => SQLITE_BUSY,
            Err(err) => self.set_last_error(SQLITE_IOERR_UNLOCK, err),
        }
    }

    /// Check if another file-handle holds a [LockKind::Reserved] lock on a file.
    pub(crate) fn check_reserved_lock(
        // p_file: *mut sqlite3_file,
        &mut self,
        p_res_out: *mut i32,
    ) -> i32 {
        // let state = match file_state(p_file) {
        //     Ok(f) => f,
        //     Err(_) => return SQLITE_IOERR_CHECKRESERVEDLOCK,
        // };
        // log::trace!("[{}] check_reserved_lock ({})", state.id, state.db_name);

        // #[cfg(feature = "sqlite_test")]
        // if simulate_io_error() {
        //     return SQLITE_IOERR_CHECKRESERVEDLOCK;
        // }

        if let Err(err) = self.file.reserved().and_then(|is_reserved| {
            let p_res_out: &mut i32 = unsafe { p_res_out.as_mut().ok_or_else(null_ptr_error) }?;
            *p_res_out = is_reserved as i32;
            Ok(())
        }) {
            return self.set_last_error(SQLITE_IOERR_UNLOCK, err);
        }

        SQLITE_OK
    }
}
