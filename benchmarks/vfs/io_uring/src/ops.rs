use std::os::unix::ffi::OsStrExt;
use std::ffi::{CString, CStr};
use std::os::raw::c_void;
use std::fs::File;
use std::os::unix::io::{FromRawFd,AsRawFd};

use sqlite_loadable::{Result, Error, ErrorKind};

// IO Uring errors: https://codebrowser.dev/linux/linux/include/uapi/asm-generic/errno-base.h.html

use std::{ptr, mem};
use sqlite_loadable::ext::sqlite3ext_vfs_find;
use sqlite_loadable::vfs::default::DefaultVfs;

use io_uring::{opcode, types, IoUring};
use std::io;

// https://github.com/torvalds/linux/blob/633b47cb009d09dc8f4ba9cdb3a0ca138809c7c7/include/uapi/linux/falloc.h#L5
const FALLOC_FL_KEEP_SIZE: u32 = 1;

pub struct Ops {
    ring: IoUring,
    file_path: CString,
    file_fd: Option<i32>
}

impl Ops {
    pub fn new(file_path: CString, ring_size: u32) -> Self {
        // Tested on kernels 5.15.49, 6.3.13
        // let mut ring = IoUring::new(ring_size).unwrap(); // 3/5
        let mut ring = IoUring::builder().setup_sqpoll(500).build(ring_size).unwrap(); // 3/5

        Ops {
            ring,
            file_path,
            file_fd: None,
        }
    }

    pub fn open_file(&mut self) -> Result<()> {
        let dirfd = types::Fd(libc::AT_FDCWD);

        // source: https://stackoverflow.com/questions/5055859/how-are-the-o-sync-and-o-direct-flags-in-open2-different-alike
        let flags = libc::O_DIRECT as u64 | libc::O_SYNC as u64 | libc::O_CREAT as u64;

        let openhow = types::OpenHow::new().flags(flags);
    
        let open_e = opcode::OpenAt2::new(dirfd, self.file_path.as_ptr(), &openhow)
            .build()
            .user_data(0xB33F);
    
        unsafe {
            self.ring.submission()
                .push(&open_e)
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

        // Doesn't make a difference
        // self.ring.submitter().register_files(&[result])
        //     .map_err(|_| Error::new_message("failed to register file"))?;
    
        Ok(())
    }

    pub unsafe fn o_read(
        &mut self,
        offset: u64,
        size: u32,
        buf_out: *mut c_void,
    ) -> Result<()> {
        let mut op = opcode::Read::new(types::Fd(self.file_fd.unwrap()), buf_out as *mut _, size)
            .offset(offset);
        self.ring
            .submission()
            .push(&op.build().user_data(1));
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
        let mut op = opcode::Write::new(types::Fd(self.file_fd.unwrap()), buf_in as *const _, size)
            .offset(offset);
        self.ring
            .submission()
            .push(&op.build().user_data(2));
        self.ring.submit_and_wait(1)
            .map_err(|_| Error::new_message("submit failed or timed out"))?;
        let cqe = self.ring.completion().next().unwrap();
        if cqe.result() < 0 {
            Err(Error::new_message(format!("raw os error result: {}", -cqe.result() as i32)))?;
        }
        Ok(())
    }

    /*
    // TODO is there also a ftruncate for io_uring? fallocate?
    pub unsafe fn o_truncate(&mut self, size: i64) -> Result<()> {
        let result = libc::ftruncate(self.file_fd.unwrap(), size);
        if result == -1 {
            Err(Error::new_message(format!("raw os error result: {}", result)))?;
        }
        Ok(())
    }
    */

    pub unsafe fn o_truncate(&mut self, size: i64) -> Result<()> {
        let mut op = opcode::Fallocate::new(types::Fd(self.file_fd.unwrap()), size.try_into().unwrap())
            .mode(FALLOC_FL_KEEP_SIZE);
        
        self.ring
            .submission()
            .push(&op.build().user_data(3));

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

    pub unsafe fn o_file_size(&mut self, out: *mut u64) -> Result<()> {

        let file = File::from_raw_fd(self.file_fd.unwrap());
        let size = file.metadata().unwrap().len();

        unsafe {
            *out = size;
        }

        Ok(())
    }
}
