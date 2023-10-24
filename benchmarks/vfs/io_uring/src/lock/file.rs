use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::ops::Range;
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use std::{env, io};

use super::kind::LockKind;
use super::wrapper::{flock_unlock, flock_shared, flock_exclusive};

pub struct FileLock {
    file: Option<File>,
    fd: RawFd,
}

impl FileLock {
    pub fn new(file: File) -> Self {
        Self {
            fd: file.as_raw_fd(),
            file: Some(file),
        }
    }

    pub fn file(&mut self) -> &mut File {
        self.file
            .get_or_insert_with(|| unsafe { File::from_raw_fd(self.fd) })
    }

    pub fn unlock(&self) {
        flock_unlock(self.fd);
    }

    pub fn shared(&self) -> bool {
        flock_shared(self.fd)
    }

    pub fn wait_shared(&self) {
        flock_wait_shared(self.fd)
    }

    pub fn exclusive(&self) -> bool {
        flock_exclusive(self.fd)
    }

    pub fn wait_exclusive(&self) {
        flock_wait_exclusive(self.fd)
    }
}

pub(crate) fn flock_wait_shared(fd: RawFd) {
    unsafe {
        if libc::flock(fd, libc::LOCK_SH) == 0 {
            return;
        }
    }

    let err = std::io::Error::last_os_error();
    panic!("lock shared failed: {}", err);
}

pub(crate) fn flock_wait_exclusive(fd: RawFd) {
    unsafe {
        if libc::flock(fd, libc::LOCK_EX) == 0 {
            return;
        }
    }

    let err = std::io::Error::last_os_error();
    panic!("lock exclusive failed: {}", err);
}

impl Drop for FileLock {
    fn drop(&mut self) {
        self.unlock();
        self.file.take();
    }
}