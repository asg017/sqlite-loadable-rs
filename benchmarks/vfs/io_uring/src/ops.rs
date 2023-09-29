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

use io_uring::{register, opcode, types, IoUring};
use std::io;

// https://github.com/torvalds/linux/blob/633b47cb009d09dc8f4ba9cdb3a0ca138809c7c7/include/uapi/linux/falloc.h#L5
const FALLOC_FL_KEEP_SIZE: i32 = 1;

const USER_DATA_OPEN: u64 = 0x1;
const USER_DATA_READ: u64 = 0x2;
const USER_DATA_STATX: u64 = 0x3;
const USER_DATA_WRITE: u64 = 0x4;
const USER_DATA_FALLOCATE: u64 = 0x5;
const USER_DATA_CLOSE: u64 = 0x6;

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
        let flags = libc::O_DIRECT as u64 | libc::O_SYNC as u64 | libc::O_CREAT as u64 | libc::O_RDWR as u64;

        let openhow = types::OpenHow::new().flags(flags);
    
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

        self.ring.submitter().register_files(&[result])
            .map_err(|_| Error::new_message("failed to register file"))?;
    
        Ok(())
    }

    pub unsafe fn o_read(
        &mut self,
        offset: u64,
        size: u32,
        buf_out: *mut c_void,
    ) -> Result<()> {
        let mut op = opcode::Read::new(types::Fixed(self.file_fd.unwrap().try_into().unwrap()), buf_out as *mut _, size)
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
        let mut op = opcode::Write::new(types::Fixed(self.file_fd.unwrap().try_into().unwrap()), buf_in as *const _, size)
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

    pub unsafe fn o_truncate(&mut self, size: i64) -> Result<()> {
        let mut op = opcode::Fallocate::new(types::Fixed(self.file_fd.unwrap().try_into().unwrap()), size.try_into().unwrap())
            .mode(FALLOC_FL_KEEP_SIZE);
        
        // let mut op = opcode::Fallocate::new(types::Fd(self.file_fd.unwrap()), size.try_into().unwrap())
        //     .mode(FALLOC_FL_KEEP_SIZE);
        
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
        let mut op = opcode::Close::new(types::Fixed(self.file_fd.unwrap().try_into().unwrap()));
    
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

        let _ = self.ring.submitter().unregister_files();

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
    
}

/*
// All fail for some reason
#[cfg(test)]
mod tests {
    use io_uring::{register, opcode};

    #[test]
    fn test_supported_ops() {
        let mut probe = register::Probe::new();
        assert!(probe.is_supported(opcode::OpenAt2::CODE));
        assert!(probe.is_supported(opcode::Read::CODE));
        assert!(probe.is_supported(opcode::Write::CODE));
        assert!(probe.is_supported(opcode::Fallocate::CODE));
        assert!(probe.is_supported(opcode::Close::CODE));
        assert!(probe.is_supported(opcode::Statx::CODE));
    }
}
*/