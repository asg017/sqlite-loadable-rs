use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::ops::Range;
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use std::{env, io};

use super::file::FileLock;
use super::LockKind;
use super::lock::{flock_exclusive, flock_shared, flock_unlock};

/// SQLite's default locking on UNIX systems is quite involved to work around certain limitations
/// of POSIX locks. See https://github.com/sqlite/sqlite/blob/master/src/os_unix.c#L1026-L1114 for
/// details.
///
/// Since I don't want to re-implement that, I am going with something simpler which should suffice
/// the use-case of a VFS only used for tests. The locking uses BSD locks instead of POSIX locks.
///
/// BSD locks unfortunately don't support locks on by ranges, which is why the following creates
/// a file per assumed byte. This is quite heavy on file access and usage of file descriptors, but
/// should suffice for the purpose of a test vfs.
pub struct RangeLock {
    ino: u64,
    locks: HashMap<u8, (RawFd, LockKind)>,
}

impl RangeLock {
    pub fn new(ino: u64) -> Self {
        Self {
            ino,
            locks: Default::default(),
        }
    }

    pub fn lock(&mut self, range: Range<u8>, to: LockKind) -> io::Result<bool> {
        // get exclusive lock on file descriptor that acts as a mutex
        let mutex = FileLock::new(
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(env::temp_dir().join(format!("{}_m.lck", self.ino)))?,
        );
        mutex.wait_exclusive(); // is unlocked as soon as mutex is dropped

        for i in range.clone() {
            let (fd, current) = match self.locks.get(&i) {
                Some(fd) => fd,
                None => {
                    let f = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create(true)
                        .open(env::temp_dir().join(format!("{}_{}.lck", self.ino, i)))?;
                    self.locks
                        .entry(i)
                        .or_insert((f.into_raw_fd(), LockKind::None))
                }
            };

            if *current == to {
                continue;
            }

            let ok = match to {
                LockKind::None => {
                    flock_unlock(*fd);
                    true
                }
                LockKind::Shared => flock_shared(*fd),
                LockKind::Exclusive => flock_exclusive(*fd),
                _ => todo!(),
            };
            if !ok {
                // revert locks
                for i in range.start..=i {
                    if let Some((fd, current)) = self.locks.get_mut(&i) {
                        match current {
                            LockKind::None => flock_unlock(*fd),
                            LockKind::Shared => {
                                flock_shared(*fd);
                            }
                            LockKind::Exclusive => {
                                flock_exclusive(*fd);
                            }
                            _ => todo!(),
                        }
                    }
                }

                return Ok(false);
            }
        }

        if to == LockKind::None {
            // Remove to free up file descriptors
            for i in range {
                if let Some((fd, _)) = self.locks.remove(&i) {
                    unsafe { File::from_raw_fd(fd) };
                }
            }
        } else {
            // update current locks once all where successful
            for i in range {
                if let Some((_, current)) = self.locks.get_mut(&i) {
                    *current = to;
                }
            }
        }

        Ok(true)
    }
}

impl Drop for RangeLock {
    fn drop(&mut self) {
        // unlock all
        for (_, (fd, lock)) in std::mem::take(&mut self.locks) {
            if lock == LockKind::None {
                continue;
            }

            flock_unlock(fd);
            unsafe { File::from_raw_fd(fd) };
        }
    }
}
