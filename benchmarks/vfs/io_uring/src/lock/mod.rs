pub(crate) mod kind;
pub(crate) mod file;
pub(crate) mod range;
pub(crate) mod wal;
pub(crate) mod wrapper;
pub(crate) mod traits;

/*
use std::{fs::{self, Permissions}, ffi::CString, sync::{Mutex, Arc}, mem::MaybeUninit, collections::HashMap, pin::Pin, io::ErrorKind};
use sqlite3ext_sys::{self, sqlite3_file, SQLITE_IOERR_LOCK, SQLITE_OK, SQLITE_IOERR_UNLOCK, SQLITE_IOERR_CHECKRESERVEDLOCK, SQLITE_BUSY};

use self::{kind::LockKind, range::RangeLock, wal::{WalConnection, WalIndexLock, WalIndex}, file::FileLock};

// TODO: add to [Vfs]?
const MAX_PATH_LENGTH: usize = 512;


#[repr(C)]
struct FileState {
    last_error: Arc<Mutex<Option<(i32, std::io::Error)>>>,
    next_id: usize,
    ext: MaybeUninit<FileExt>, // TODO drop manually
}

#[repr(C)]
struct FileExt {
    // vfs: Arc<V>,
    // vfs_name: CString,
    // db_name: String,
    // file: F,
    delete_on_close: bool,
    /// The last error; shared with the VFS.
    last_error: Arc<Mutex<Option<(i32, std::io::Error)>>>,
    /// The last error number of this file/connection (not shared with the VFS).
    last_errno: i32,
    wal_index: Option<(F::WalIndex, bool)>,
    wal_index_regions: HashMap<u32, Pin<Box<[u8; 32768]>>>,
    wal_index_locks: HashMap<u8, WalIndexLock>,
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

impl FileExt {
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

        let lock = match LockKind::from_i32(e_lock) {
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
                        .any(|(_, lock)| *lock == WalIndexLock::Exclusive);

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

        let lock = match LockKind::from_i32(e_lock) {
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

    fn wal_index(&self, readonly: bool) -> Result<WalIndex, std::io::Error> {
        let path = self.path.with_extension(format!(
            "{}-shm",
            self.path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.split_once('-').map(|(f, _)| f).unwrap_or(ext))
                .unwrap_or("db")
        ));
        let is_new = !path.exists();

        let mut opts = fs::OpenOptions::new();
        opts.read(true);
        if !readonly {
            opts.write(true).create(true).truncate(false);
        }

        let file = opts.open(&path)?;
        let mut file_lock = FileLock::new(file);
        if !readonly && file_lock.exclusive() {
            // If it is the first connection to open the database, truncate the index.
            let new_file = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)
                .map_err(|err| err)?;

            let new_lock = FileLock::new(new_file);

            if is_new {
                let mode = permissions(&self.path)?;
                let perm = Permissions::from_mode(mode);
                // Match permissions of main db file, but don't downgrade to readonly.
                if !perm.readonly() {
                    fs::set_permissions(&path, perm)?;
                }
            }

            // Transition previous lock to shared before getting a shared on the new file
            // descriptor to make sure that there isn't any other concurrent process/thread getting
            // an exclusive lock during the transition.
            assert!(file_lock.shared());
            assert!(new_lock.shared());

            file_lock = new_lock;
        } else {
            file_lock.wait_shared();
        }

        Ok(WalConnection {
            path,
            file_lock,
            wal_lock: RangeLock::new(self.file_ino),
            readonly,
        })
    }
}
*/