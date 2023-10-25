use std::fs::{File, OpenOptions};
use std::os::unix::fs::MetadataExt;
use std::os::unix::io::{AsRawFd, IntoRawFd, RawFd};
use std::os::unix::prelude::FromRawFd;
use std::path::Path;
use std::{env, io};

use super::kind::LockKind;

/// SQLite's default locking on UNIX systems is quite involved to work around certain limitations
/// of POSIX locks. See https://github.com/sqlite/sqlite/blob/master/src/os_unix.c#L1026-L1114 for
/// details.
///
/// Since I don't want to re-implement that, I am going with something simpler which should suffice
/// the use-case of a VFS only used for tests. The locking uses BSD locks instead of POSIX locks.
/// Since SQLite has five different lock states, one single BSD lock is not enough (one BSD lock is
/// either unlocked, shared or exclusive). This is why each database lock consists out of two BSD
/// locks. They work as follows to achieve the SQLite lock states:
///
/// |               {name}.db        /tmp/{ino}.lck
///
/// unlocked        unlocked         unlocked
///
/// shared          shared           shared -> unlocked ¹
///
/// reserved        shared           exclusive -> shared ²
///
/// pending         shared           exclusive
///
/// exclusive       exclusive        exclusive
///
///
/// ¹ The shared lock is first acquired, but then unlocked again. The shared lock is not kept to
/// allow the creation of an exclusive lock for a reserved lock.
///
/// ² The reserved lock must still allow new shared locks, which is why it is only tested for
/// exclusivity first (to make sure that there is neither a pending nor exclusive lock) and then
/// downgraded to a shared lock. Keeping the shared lock prevents any other reserved lock as there
/// can only be one.

pub struct Lock {
    fd1: RawFd,
    fd1_owned: bool,
    fd2: RawFd,
    current: LockKind,
}

impl Lock {
    pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        let f1 = File::open(path)?;

        let f2 = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(env::temp_dir().join(format!("{}.lck", f1.metadata()?.ino())))?;

        Ok(Lock {
            fd1: f1.into_raw_fd(),
            fd1_owned: true,
            fd2: f2.into_raw_fd(),
            current: LockKind::None,
        })
    }

    pub fn from_file(f1: &File) -> io::Result<Self> {
        let f2 = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(env::temp_dir().join(format!("{}.lck", f1.metadata()?.ino())))?;

        Ok(Lock {
            fd1: f1.as_raw_fd(),
            fd1_owned: false,
            fd2: f2.into_raw_fd(),
            current: LockKind::None,
        })
    }

    pub fn from_raw_fd(f1_fd: &RawFd) -> io::Result<Self> {
        let f1 = unsafe { File::from_raw_fd(*f1_fd) };
        Self::from_file(&f1)
    }

    pub fn current(&self) -> LockKind {
        self.current
    }

    pub fn reserved(&self) -> bool {
        if self.current > LockKind::Shared {
            return true;
        }

        if flock_exclusive(self.fd2) {
            flock_unlock(self.fd2);
            false
        } else {
            true
        }
    }

    /// Transition the lock to the given [LockKind].
    ///
    /// # Panics
    ///
    /// Panics for invalid lock transitions or failed unlocks (which are not expected to happen).
    pub fn lock(&mut self, to: LockKind) -> bool {
        if self.current == to {
            return true;
        }

        // Never move from unlocked to anything higher than shared
        if self.current == LockKind::None && to != LockKind::Shared {
            panic!(
                "cannot transition from unlocked to anything higher than shared (tried: {:?})",
                to
            )
        }

        match to {
            LockKind::None => {
                flock_unlock(self.fd1);

                if matches!(self.current, LockKind::Pending | LockKind::Exclusive) {
                    flock_unlock(self.fd2);
                }

                self.current = LockKind::None;

                return true;
            }

            LockKind::Shared => {
                if self.current != LockKind::Reserved {
                    if !flock_shared(self.fd1) {
                        return false;
                    }
                }

                if flock_shared(self.fd2) {
                    flock_unlock(self.fd2);
                    self.current = LockKind::Shared;
                    true
                } else if matches!(self.current, LockKind::Pending | LockKind::Exclusive) {
                    panic!("failed to transition to shared from {:?}", self.current);
                } else if self.current == LockKind::None {
                    flock_unlock(self.fd1);
                    false
                } else {
                    false
                }
            }

            LockKind::Reserved => {
                // A shared lock is always held when a reserved lock is requested
                if self.current != LockKind::Shared {
                    panic!(
                        "must hold a shared lock when requesting a reserved lock (current: {:?})",
                        self.current
                    )
                }

                if flock_exclusive(self.fd2) {
                    flock_shared(self.fd2);
                    self.current = LockKind::Reserved;
                    true
                } else {
                    false
                }
            }

            LockKind::Pending => {
                panic!("cannot explicitly request pending lock (request explicit lock instead)")
            }

            LockKind::Exclusive => {
                if self.current != LockKind::Pending && !flock_exclusive(self.fd2) {
                    return false;
                }

                if !flock_exclusive(self.fd1) {
                    self.current = LockKind::Pending;
                    return true;
                }

                self.current = LockKind::Exclusive;
                true
            }
        }
    }
}

pub(crate) fn flock_unlock(fd: RawFd) {
    unsafe {
        if libc::flock(fd, libc::LOCK_UN | libc::LOCK_NB) != 0 {
            panic!("unlock failed: {}", std::io::Error::last_os_error());
        }
    }
}

pub(crate) fn flock_shared(fd: RawFd) -> bool {
    unsafe {
        if libc::flock(fd, libc::LOCK_SH | libc::LOCK_NB) == 0 {
            return true;
        }
    }

    let err = std::io::Error::last_os_error();
    if err.raw_os_error().unwrap() == libc::EWOULDBLOCK {
        return false;
    }

    panic!("lock shared failed: {}", err);
}

pub(crate) fn flock_exclusive(fd: RawFd) -> bool {
    unsafe {
        if libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) == 0 {
            return true;
        }
    }

    let err = std::io::Error::last_os_error();
    if err.raw_os_error().unwrap() == libc::EWOULDBLOCK {
        return false;
    }

    panic!("lock exclusive failed: {}", err);
}

impl Drop for Lock {
    fn drop(&mut self) {
        self.lock(LockKind::None);

        // Close file descriptors.
        unsafe {
            if self.fd1_owned {
                File::from_raw_fd(self.fd1);
            }
            File::from_raw_fd(self.fd2);
        }
    }
}
