use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::os::fd::RawFd;
use std::os::raw::c_void;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::rc::Rc;

use io_uring::types::Fd;
use libc::c_char;
use sqlite3ext_sys::{
    SQLITE_IOCAP_ATOMIC, SQLITE_IOCAP_POWERSAFE_OVERWRITE, SQLITE_IOCAP_SAFE_APPEND,
    SQLITE_IOCAP_SEQUENTIAL,
};
use sqlite3ext_sys::{SQLITE_IOERR_SHMLOCK, SQLITE_IOERR_SHMMAP};
use sqlite_loadable::SqliteIoMethods;
use std::io::{Error, ErrorKind, Result};

use sqlite3ext_sys::{SQLITE_BUSY, SQLITE_LOCK_SHARED, SQLITE_OK};

// IO Uring errors: https://codebrowser.dev/linux/linux/include/uapi/asm-generic/errno-base.h.html

use sqlite_loadable::ext::{sqlite3_file, sqlite3ext_vfs_find};
use sqlite_loadable::vfs::shim::{ShimFile, ShimVfs};
use std::{mem, ptr};

use io_uring::{opcode, register, types, IoUring};
use std::io;

use crate::lock::Lock;
use crate::lock::LockKind;

const USER_DATA_OPEN: u64 = 0x1;
const USER_DATA_READ: u64 = 0x2;
const USER_DATA_STATX: u64 = 0x3;
const USER_DATA_WRITE: u64 = 0x4;
const USER_DATA_FALLOCATE: u64 = 0x5;
const USER_DATA_CLOSE: u64 = 0x6;
const USER_DATA_FSYNC: u64 = 0x7;

// Tested on linux 5.15.49, 6.1.0, 6.3.13
pub struct OpsFd {
    ring: Rc<RefCell<IoUring>>,
    file_path: *const char,
    file_fd: Option<i32>,
    file: Option<File>,
    lock: Option<Lock>,
    file_name: String,
}

/// I was tempted really often to convert file_path to Path with a lifetime, PathBuf, CString
/// but it quickly becomes awkward due to libc insistence on read pointers.
/// A purely oxidized project should work with Path.
/// Besides, the pointer memset that file_path is stored, is managed by C,
/// bad things will happen to sqlite3 if you try to take away ownership.
impl OpsFd {
    // Used for tests
    pub fn new(file_path: *const char, ring_size: u32) -> Self {
        let mut ring = Rc::new(RefCell::new(IoUring::new(ring_size).unwrap()));

        Self::from_rc_refcell_ring(file_path, ring)
    }

    pub fn from_rc_refcell_ring(file_path: *const char, ring: Rc<RefCell<IoUring>>) -> Self {
        OpsFd {
            ring,
            file_path,
            file_fd: None,
            file: None,
            lock: None,
            file_name: unsafe {
                CStr::from_ptr(file_path as *const _)
                    .to_str()
                    .unwrap()
                    .to_string()
            },
        }
    }

    // all tests pass
    pub fn open_file(&mut self) -> Result<()> {
        // This calls libc::open
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(self.file_name.as_str())?;

        let raw_fd = file.as_raw_fd();

        self.file = Some(file);
        self.file_fd = Some(raw_fd);

        Ok(())
    }

    // tests pass except anything that writes
    /*
    pub fn open_file(&mut self) -> Result<()> {
        let mut ring = self.ring.as_ref().borrow_mut();

        let dirfd = types::Fd(libc::AT_FDCWD);

        let flags = libc::O_CREAT as u64;

        let openhow = types::OpenHow::new()
            .flags(flags)
            .mode(libc::S_IRUSR as u64 | libc::S_IWUSR as u64);

        let open_e: opcode::OpenAt2 =
            opcode::OpenAt2::new(dirfd, self.file_path as *const _, &openhow);

        unsafe {
            ring.submission()
                .push(&open_e.build().user_data(USER_DATA_OPEN))
                .expect("queue is full");
        }

        ring.submit_and_wait(1).expect("submit failed or timed out");

        let cqes: Vec<io_uring::cqueue::Entry> = ring.completion().map(Into::into).collect();
        let cqe = &cqes.as_slice()[0];
        let result = cqe.result();

        if result < 0 {
            Err(Error::new(
                ErrorKind::Other,
                format!("open_file: raw os error result: {}", -result as i32),
            ))
        } else {
            self.file_fd = Some(result);
            Ok(())
        }
    }
    */

    pub unsafe fn o_read(&mut self, offset: u64, size: u32, buf_out: *mut c_void) -> Result<()> {
        let mut ring = self.ring.as_ref().borrow_mut();

        let fd = types::Fd(self.file_fd.unwrap());
        let mut op = opcode::Read::new(fd, buf_out as *mut _, size).offset(offset);
        ring.submission()
            .push(&op.build().user_data(USER_DATA_READ))
            .expect("queue is full");
        ring.submit_and_wait(1).expect("submit failed or timed out");

        let cqes: Vec<io_uring::cqueue::Entry> = ring.completion().map(Into::into).collect();
        let cqe = &cqes.as_slice()[0];
        let result = cqe.result();

        if result < 0 {
            Err(Error::new(
                ErrorKind::Other,
                format!("read: raw os error result: {}", -result as i32),
            ))
        } else {
            Ok(())
        }
    }

    pub unsafe fn o_write(&mut self, buf_in: *const c_void, offset: u64, size: u32) -> Result<()> {
        let mut ring = self.ring.as_ref().borrow_mut();

        let fd = types::Fd(self.file_fd.unwrap());
        let mut op = opcode::Write::new(fd, buf_in as _, size).offset(offset);
        ring.submission()
            .push(&op.build().user_data(USER_DATA_WRITE))
            .expect("queue is full");
        ring.submit_and_wait(1).expect("submit failed or timed out");

        let cqes: Vec<io_uring::cqueue::Entry> = ring.completion().map(Into::into).collect();
        let cqe = &cqes.as_slice()[0];
        let result = cqe.result();

        if result < 0 {
            Err(Error::new(
                ErrorKind::Other,
                format!("write: raw os error result: {}", -result as i32),
            ))
        } else {
            Ok(())
        }
    }

    // TODO implement with a read then write, also apply linking to guarantee read before write
    // pub unsafe fn o_truncate(&mut self, size: i64) -> Result<()> {
    //     let mut file_size_box = Box::new(0 as u64);
    //     let mut file_size_ptr = Box::into_raw(file_size_box);
    //     self.o_file_size(file_size_ptr);

    //     let mut ring = self.ring.as_ref().borrow_mut();

    //     let fd = types::Fd(self.file_fd.unwrap());
    //     let new_size: u64 = size.try_into().unwrap();
    //     let mut op = opcode::Fallocate::new(fd, (*file_size_ptr) - new_size)
    //         .offset((size - 1).try_into().unwrap())
    //         .mode(libc::FALLOC_FL_COLLAPSE_RANGE);

    //     ring.submission()
    //         .push(&op.build().user_data(USER_DATA_FALLOCATE))
    //         .expect("queue is full");;

    //     ring.submit_and_wait(1)
    //         .expect("submit failed or timed out");

    //     let cqes: Vec<io_uring::cqueue::Entry> = ring.completion().map(Into::into).collect();
    //     let cqe = &cqes.as_slice()[0];
    //     let result = cqe.result();

    //     Box::from_raw(file_size_ptr);

    //     if result < 0 {
    //         Err(Error::new(
    //             ErrorKind::Other,
    //             format!("truncate: raw os error result: {}", -result as i32),
    //         ))
    //     }else {
    //         Ok(())
    //     }
    // }

    pub unsafe fn o_truncate(&mut self, size: i64) -> Result<()> {
        // libc::ftruncate using self.file_fd returns -1
        let result = libc::truncate(self.file_path as *const _, size);
        if result != 0 {
            Err(Error::new(
                ErrorKind::Other,
                format!("truncate: raw os error result: {}", result),
            ))
        } else {
            Ok(())
        }
    }

    // SQLite Documentation:
    // Implement this function to read data from the file at the specified offset and store it in `buf_out`.
    // You can use the same pattern as in `read_file`.
    pub unsafe fn o_fetch(
        &mut self,
        offset: u64,
        size: u32,
        buf_out: *mut *mut c_void,
    ) -> Result<()> {
        self.o_read(offset, size, *buf_out as *mut _)
    }

    pub unsafe fn o_close(&mut self) -> Result<()> {
        let mut ring = self.ring.as_ref().borrow_mut();

        let fd = types::Fd(self.file_fd.unwrap());
        let mut op = opcode::Close::new(fd);

        ring.submission()
            .push(&op.build().user_data(USER_DATA_CLOSE))
            .expect("queue is full");

        ring.submit_and_wait(1).expect("submit failed or timed out");

        let cqes: Vec<io_uring::cqueue::Entry> = ring.completion().map(Into::into).collect();
        let cqe = &cqes.as_slice()[0];
        let result = cqe.result();

        if result < 0 {
            Err(Error::new(
                ErrorKind::Other,
                format!("close: raw os error result: {}", -result as i32),
            ))
        } else {
            Ok(())
        }
    }

    pub unsafe fn o_file_size(&mut self, out: *mut u64) -> Result<()> {
        let mut ring = self.ring.as_ref().borrow_mut();

        let mut statx_buf: libc::statx = unsafe { std::mem::zeroed() };
        let mut statx_buf_ptr: *mut libc::statx = &mut statx_buf;

        let dirfd = types::Fd(libc::AT_FDCWD);
        let statx_op =
            opcode::Statx::new(dirfd, self.file_path as *const _, statx_buf_ptr as *mut _)
                .flags(libc::AT_EMPTY_PATH)
                .mask(libc::STATX_ALL);

        ring.submission()
            .push(&statx_op.build().user_data(USER_DATA_STATX))
            .expect("queue is full");

        ring.submit_and_wait(1).expect("submit failed or timed out");

        let cqes: Vec<io_uring::cqueue::Entry> = ring.completion().map(Into::into).collect();
        let cqe = &cqes.as_slice()[0];
        let result = cqe.result();

        if result < 0 {
            Err(Error::new(
                ErrorKind::Other,
                format!("file_size: raw os error result: {}", -result as i32),
            ))
        } else {
            unsafe {
                *out = statx_buf.stx_size as u64;
            }

            Ok(())
        }
    }

    pub unsafe fn o_fsync(&mut self, flags: i32) -> Result<()> {
        let mut ring = self.ring.as_ref().borrow_mut();

        let fd = types::Fd(self.file_fd.unwrap());
        let op = opcode::Fsync::new(fd);

        ring.submission()
            .push(&op.build().user_data(USER_DATA_FSYNC))
            .expect("queue is full");

        ring.submit_and_wait(1).expect("submit failed or timed out");

        let cqes: Vec<io_uring::cqueue::Entry> = ring.completion().map(Into::into).collect();
        let cqe = &cqes.as_slice()[0];
        let result = cqe.result();

        if result < 0 {
            Err(Error::new(
                ErrorKind::Other,
                format!("fsync: raw os error result: {}", -result as i32),
            ))
        } else {
            Ok(())
        }
    }

    fn is_exclusive_requested_pending_acquired(&mut self, to: LockKind) -> bool {
        if let Some(lock) = &mut self.lock {
            lock.lock(to) && lock.current() == to
        } else {
            false
        }
    }

    fn init_lock(&mut self) -> Result<()> {
        if self.lock.is_none() {
            let cstr = unsafe { CStr::from_ptr(self.file_path as *const _) };

            let str_result = cstr.to_str();

            let err = Error::new(ErrorKind::Other, "bad file name");

            // the fd from the ring, returns: os error 9
            let str = str_result.map_err(|_| err)?;

            let lock = Lock::new(str)?;

            self.lock = Some(lock);
        }
        Ok(())
    }

    pub fn lock_or_unlock(&mut self, lock_request: i32) -> Result<i32> {
        self.init_lock()?;
        LockKind::from_repr(lock_request)
            .map(|kind| self.is_exclusive_requested_pending_acquired(kind))
            .map(|ok_or_busy| if ok_or_busy { SQLITE_OK } else { SQLITE_BUSY })
            .ok_or_else(|| Error::new(ErrorKind::Other, "Missing lock"))
    }

    pub fn lock_reserved(&mut self) -> Result<bool> {
        self.init_lock()?;
        if let Some(lock) = &mut self.lock {
            Ok(lock.reserved())
        } else {
            Err(Error::new(ErrorKind::Other, "Missing lock"))
        }
    }
}

// TODO remove *mut sqlite3_file
impl SqliteIoMethods for OpsFd {
    fn close(&mut self) -> Result<()> {
        log::trace!("file close");

        unsafe { self.o_close() }
    }

    fn read(&mut self, buf: *mut c_void, s: i32, ofst: i64) -> Result<()> {
        log::trace!("file read");

        unsafe { self.o_read(ofst as u64, s as u32, buf) }
    }

    fn write(&mut self, buf: *const c_void, s: i32, ofst: i64) -> Result<()> {
        log::trace!("file write");

        unsafe { self.o_write(buf, ofst as u64, s as u32) }
    }

    fn truncate(&mut self, size: i64) -> Result<()> {
        log::trace!("file truncate");

        unsafe { self.o_truncate(size) }
    }

    fn sync(&mut self, flags: i32) -> Result<()> {
        log::trace!("file sync");

        unsafe { self.o_fsync(flags) }
    }

    fn file_size(&mut self, p_size: *mut i64) -> Result<()> {
        log::trace!("file size");

        unsafe { self.o_file_size(p_size as *mut u64) }
    }

    fn lock(&mut self, arg2: i32) -> Result<i32> {
        log::trace!("file lock");
        self.lock_or_unlock(arg2)
    }

    fn unlock(&mut self, arg2: i32) -> Result<i32> {
        log::trace!("file unlock");
        self.lock_or_unlock(arg2)
    }

    fn check_reserved_lock(&mut self, p_res_out: *mut i32) -> Result<()> {
        log::trace!("file check reserved lock");

        let lock_reserved = self.lock_reserved()?;
        unsafe {
            *p_res_out = if lock_reserved { 1 } else { 0 };
        }
        Ok(())
    }

    /// See https://www.sqlite.org/c3ref/file_control.html
    /// and also https://www.sqlite.org/c3ref/c_fcntl_begin_atomic_write.html
    fn file_control(&mut self, op: i32, p_arg: *mut c_void) -> Result<()> {
        log::trace!("file control");
        Ok(())
    }

    fn sector_size(&mut self) -> Result<i32> {
        log::trace!("sector size");
        Ok(1024)
    }

    fn device_characteristics(&mut self) -> Result<i32> {
        log::trace!("device characteristics");
        let x = SQLITE_IOCAP_ATOMIC
            | SQLITE_IOCAP_POWERSAFE_OVERWRITE
            | SQLITE_IOCAP_SAFE_APPEND
            | SQLITE_IOCAP_SEQUENTIAL;
        Ok(x)
    }

    fn shm_map(&mut self, i_pg: i32, pgsz: i32, arg2: i32, arg3: *mut *mut c_void) -> Result<()> {
        log::trace!("shm map");
        Ok(())
    }

    fn shm_lock(&mut self, offset: i32, n: i32, flags: i32) -> Result<()> {
        log::trace!("shm lock");
        Ok(())
    }

    fn shm_barrier(&mut self) -> Result<()> {
        log::trace!("shm barrier");
        Ok(())
    }

    fn shm_unmap(&mut self, delete_flag: i32) -> Result<()> {
        log::trace!("shm unmap");
        Ok(())
    }

    fn fetch(&mut self, ofst: i64, size: i32, pp: *mut *mut c_void) -> Result<()> {
        unsafe {
            log::trace!("file fetch");
            self.o_fetch(ofst as u64, size as u32, pp)
        }
    }

    fn unfetch(&mut self, i_ofst: i64, p: *mut c_void) -> Result<()> {
        log::trace!("file unfetch");
        Ok(())
    }
}
