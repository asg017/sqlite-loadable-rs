use std::os::unix::ffi::OsStrExt;
use std::ffi::{CString, CStr};
use std::os::raw::c_void;
use std::fs::File;
use std::os::unix::io::{FromRawFd,AsRawFd};

use sqlite_loadable::vfs::default::DefaultFile;
use sqlite_loadable::{Result, Error, ErrorKind, SqliteIoMethods};
use sqlite3ext_sys::sqlite3_file;
use sqlite3ext_sys::{SQLITE_IOCAP_ATOMIC, SQLITE_IOCAP_POWERSAFE_OVERWRITE,
    SQLITE_IOCAP_SAFE_APPEND, SQLITE_IOCAP_SEQUENTIAL};
use sqlite3ext_sys::{SQLITE_IOERR_SHMMAP, SQLITE_IOERR_SHMLOCK};    


// IO Uring errors: https://codebrowser.dev/linux/linux/include/uapi/asm-generic/errno-base.h.html

use std::{ptr, mem};
use sqlite_loadable::ext::sqlite3ext_vfs_find;
use sqlite_loadable::vfs::default::DefaultVfs;

use io_uring::{register, opcode, types, IoUring};
use std::io;

const USER_DATA_OPEN: u64 = 0x1;
const USER_DATA_READ: u64 = 0x2;
const USER_DATA_STATX: u64 = 0x3;
const USER_DATA_WRITE: u64 = 0x4;
const USER_DATA_FALLOCATE: u64 = 0x5;
const USER_DATA_CLOSE: u64 = 0x6;
const USER_DATA_FSYNC: u64 = 0x7;

pub struct Ops {
    ring: IoUring,
    file_path: CString,
    file_fd: Option<i32>,
    default_file: Option<DefaultFile>,
}

impl Ops {
    pub fn new(file_path: CString, ring_size: u32) -> Self {
        // Tested on kernels 5.15.49, 6.3.13
        let mut ring = IoUring::new(ring_size).unwrap();

        Ops {
            ring,
            file_path,
            file_fd: None,
            default_file: None,
        }
    }

    // TODO add O_DIRECT and O_SYNC parameters for systems that actually support it
    pub fn open_file(&mut self) -> Result<()> {
        let dirfd = types::Fd(libc::AT_FDCWD);

        // source: https://stackoverflow.com/questions/5055859/how-are-the-o-sync-and-o-direct-flags-in-open2-different-alike
        // let flags = libc::O_DIRECT as u64 | libc::O_SYNC as u64 | libc::O_CREAT as u64 | libc::O_RDWR as u64;
        let flags = libc::O_CREAT as u64 | libc::O_RDWR as u64;

        let openhow = types::OpenHow::new().flags(flags).mode(libc::S_IRUSR as u64 | libc::S_IWUSR as u64);
    
        let open_e = opcode::OpenAt2::new(dirfd, self.file_path.as_ptr(), &openhow);

        unsafe {
            self.ring.submission()
                .push(
                    &open_e.build()
                    .user_data(USER_DATA_OPEN)
                )
                .map_err(|_| Error::new_message("submission queue is full"))?;
        }

        self.ring.submit_and_wait(1)
            .map_err(|_| Error::new_message("submit failed or timed out"))?;
    
        let cqe = self.ring.completion().next().unwrap();

        let result = cqe.result();

        if result < 0 {
            Err(Error::new_message(format!("raw os error result: {}", -cqe.result() as i32)))?;
        }

        self.file_fd = Some(result.try_into().unwrap());

        Ok(())
    }

    pub unsafe fn o_read(
        &mut self,
        offset: u64,
        size: u32,
        buf_out: *mut c_void,
    ) -> Result<()> {
        // let fd = types::Fixed(self.file_fd.unwrap().try_into().unwrap());
        let fd = types::Fd(self.file_fd.unwrap());
        let mut op = opcode::Read::new(fd, buf_out as *mut _, size)
            .offset(offset);
        self.ring
            .submission()
            .push(&op.build().user_data(USER_DATA_READ))
            .map_err(|_| Error::new_message("submission queue is full"))?;
        self.ring.submit_and_wait(1)
            .map_err(|_| Error::new_message("submit failed or timed out"))?;
        let cqe = self.ring.completion().next().unwrap();
        if cqe.result() < 0 {
            Err(Error::new_message(format!("raw os error result: {}", -cqe.result() as i32)))?;
        }
        Ok(())
    }

    pub unsafe fn o_write(
        &mut self,
        buf_in: *const c_void,
        offset: u64,
        size: u32,
    ) -> Result<()> {
        // let fd = types::Fixed(self.file_fd.unwrap().try_into().unwrap());
        let fd = types::Fd(self.file_fd.unwrap());
        let mut op = opcode::Write::new(fd, buf_in as *const _, size)
            .offset(offset);
        self.ring
            .submission()
            .push(&op.build().user_data(USER_DATA_WRITE))
            .map_err(|_| Error::new_message("submission queue is full"))?;
        self.ring.submit_and_wait(1)
            .map_err(|_| Error::new_message("submit failed or timed out"))?;
        let cqe = self.ring.completion().next().unwrap();
        if cqe.result() < 0 {
            Err(Error::new_message(format!("raw os error result: {}", -cqe.result() as i32)))?;
        }
        Ok(())
    }

    // TODO find io_uring op, this doesn't work
    pub unsafe fn o_truncate2(&mut self, size: i64) -> Result<()> {
        // let fd = types::Fixed(self.file_fd.unwrap().try_into().unwrap());
        let fd = types::Fd(self.file_fd.unwrap());
        let mut op = opcode::Fallocate::new(fd, size.try_into().unwrap())
            .offset(0)
            // https://github.com/torvalds/linux/blob/633b47cb009d09dc8f4ba9cdb3a0ca138809c7c7/include/uapi/linux/falloc.h#L5
            .mode(libc::FALLOC_FL_KEEP_SIZE);
        
        self.ring
            .submission()
            .push(&op.build().user_data(USER_DATA_FALLOCATE))
            .map_err(|_| Error::new_message("submission queue is full"))?;

        self.ring.submit_and_wait(1)
            .map_err(|_| Error::new_message("submit failed or timed out"))?;

        let cqe = self.ring
            .completion()
            .next()
            .unwrap();
        if cqe.result() < 0 {
            Err(Error::new_message(format!("raw os error result: {}", -cqe.result() as i32)))?;
        }
        Ok(())
    }

    pub unsafe fn o_truncate(&mut self, size: i64) -> Result<()> {
        let result = libc::ftruncate(self.file_fd.unwrap(), size);
        if result == -1 {
            Err(Error::new_message(format!("raw os error result: {}", result)))?;
        }
        Ok(())
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
        let fd = types::Fixed(self.file_fd.unwrap().try_into().unwrap());
        let mut op = opcode::Close::new(fd);
    
        self.ring
            .submission()
            .push(&op.build().user_data(USER_DATA_CLOSE))
            .map_err(|_| Error::new_message("submission queue is full"))?;

        self.ring.submit_and_wait(1)
            .map_err(|_| Error::new_message("submit failed or timed out"))?;

        let cqe = self.ring
            .completion()
            .next()
            .unwrap();
        if cqe.result() < 0 {
            Err(Error::new_message(format!("raw os error result: {}", -cqe.result() as i32)))?;
        }

        Ok(())
    }
    
    pub unsafe fn o_file_size(&mut self, out: *mut u64) -> Result<()> {
        let mut statx_buf: libc::statx = unsafe { std::mem::zeroed() };
        let mut statx_buf_ptr: *mut libc::statx = &mut statx_buf;
    
        let dirfd = types::Fd(libc::AT_FDCWD);
        let statx_op = opcode::Statx::new(dirfd, self.file_path.as_ptr(), statx_buf_ptr as *mut _)
            .flags(libc::AT_EMPTY_PATH)
            .mask(libc::STATX_ALL);

        self.ring
            .submission()
            .push(&statx_op.build().user_data(USER_DATA_STATX))
            .map_err(|_| Error::new_message("submission queue is full"))?;

        self.ring.submit_and_wait(1)
            .map_err(|_| Error::new_message("submit failed or timed out"))?;

        unsafe {
            *out = statx_buf.stx_size as u64;
        }
    
        Ok(())
    }
    
    // TODO write unit test
    pub unsafe fn o_fsync(&mut self, flags: i32) -> Result<()> {
        let fd = types::Fixed(self.file_fd.unwrap().try_into().unwrap());
        let op = opcode::Fsync::new(fd);

        self.ring
            .submission()
            .push(&op.build().user_data(USER_DATA_FSYNC))
            .map_err(|_| Error::new_message("submission queue is full"))?;

        self.ring.submit_and_wait(1)
            .map_err(|_| Error::new_message("submit failed or timed out"))?;

        let cqe = self.ring
            .completion()
            .next()
            .unwrap();

        if cqe.result() < 0 {
            Err(Error::new_message(format!("raw os error result: {}", -cqe.result() as i32)))?;
        }
        Ok(())
    }
}

impl SqliteIoMethods for Ops {
    fn close(&mut self) -> Result<()> {
        unsafe { self.o_close() }
    }

    fn read(&mut self, buf: *mut c_void, s: i32, ofst: i64) -> Result<()> {
        unsafe { self.o_read(ofst as u64, s as u32, buf) }
    }

    fn write(&mut self, buf: *const c_void, s: i32, ofst: i64) -> Result<()> {
        unsafe { self.o_write(buf, ofst as u64, s as u32) }
    }

    fn truncate(&mut self, size: i64) -> Result<()> {
        unsafe { self.o_truncate(size) }
    }

    fn sync(&mut self, flags: i32) -> Result<()> {
        unsafe { self.o_fsync(flags) }
    }

    fn file_size(&mut self, p_size: *mut i64) -> Result<()> {
        unsafe { self.o_file_size(p_size as *mut u64) }
    }

    fn lock(&mut self, arg2: i32) -> Result<()> {
        Ok(())
    }

    fn unlock(&mut self, arg2: i32) -> Result<()> {
        Ok(())
    }

    fn check_reserved_lock(&mut self, p_res_out: *mut i32) -> Result<()> {
        unsafe{ *p_res_out = 0; }
        Ok(())
    }

    /// See https://www.sqlite.org/c3ref/file_control.html
    /// and also https://www.sqlite.org/c3ref/c_fcntl_begin_atomic_write.html
    fn file_control(&mut self, file: *mut sqlite3_file, op: i32, p_arg: *mut c_void) -> Result<()> {
        if let None = self.default_file {
            let orig_file: *mut sqlite3_file = unsafe { file.offset(1) };
            self.default_file = Some(DefaultFile::from_ptr(orig_file));
        }
        Ok(())
    }

    fn sector_size(&mut self) -> i32 {
        1024
    }

    fn device_characteristics(&mut self) -> i32 {
        SQLITE_IOCAP_ATOMIC | 
        SQLITE_IOCAP_POWERSAFE_OVERWRITE |
        SQLITE_IOCAP_SAFE_APPEND |
        SQLITE_IOCAP_SEQUENTIAL
    }

    fn shm_map(&mut self, i_pg: i32, pgsz: i32, arg2: i32, arg3: *mut *mut c_void) -> Result<()> {
        Err(Error::new(ErrorKind::DefineVfs(SQLITE_IOERR_SHMMAP)))
    }

    fn shm_lock(&mut self, offset: i32, n: i32, flags: i32) -> Result<()> {
        // SQLITE_IOERR_SHMLOCK is deprecated?
        Err(Error::new(ErrorKind::DefineVfs(SQLITE_IOERR_SHMLOCK)))
    }

    fn shm_barrier(&mut self) -> Result<()> {
        Ok(())
    }

    fn shm_unmap(&mut self, delete_flag: i32) -> Result<()> {
        Ok(())
    }

    fn fetch(&mut self, ofst: i64, size: i32, pp: *mut *mut c_void) -> Result<()> {
        unsafe { self.o_fetch(ofst as u64, size as u32, pp) }
    }

    fn unfetch(&mut self, i_ofst: i64, p: *mut c_void) -> Result<()> {
        Ok(())
    }
}