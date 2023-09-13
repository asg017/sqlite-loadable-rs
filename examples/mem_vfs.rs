//! cargo build --example mem_vfs
//! sqlite3 :memory: '.read examples/test.sql'
#![allow(unused)]

use libsqlite3_sys::{SQLITE_IOERR_SHMMAP, SQLITE_IOERR_SHMLOCK};
use sqlite_loadable::vfs::default::DefaultVfs;
use sqlite_loadable::vfs::vfs::create_vfs;

use sqlite_loadable::{prelude::*, SqliteIoMethods, create_file_pointer, register_vfs, Error, ErrorKind};
use sqlite_loadable::{Result, vfs::traits::SqliteVfs};

use std::os::raw::{c_int, c_void, c_char};
use sqlite3ext_sys::{sqlite3_int64, sqlite3_syscall_ptr, sqlite3_file, sqlite3_vfs, sqlite3_vfs_register, sqlite3_io_methods, sqlite3_vfs_find};

/// Inspired by https://www.sqlite.org/src/file/ext/misc/memvfs.c
struct MemVfs {
    default_vfs: DefaultVfs,
}

impl SqliteVfs for MemVfs {
    fn open(&mut self, z_name: *const c_char, p_file: *mut sqlite3_file, flags: c_int, p_res_out: *mut c_int) -> Result<()> {
        let rust_file = MemFile {
            size: 0,
            max_size: 0,
            file_content: Vec::new(),
        };
        
        // TODO finish implementation

        /*
        memset(p, 0, sizeof(*p));

        if( (flags & SQLITE_OPEN_MAIN_DB) == 0 ) return SQLITE_CANTOPEN;

        p->aData = (unsigned char*)sqlite3_uri_int64(zName,"ptr",0);

        if( p->aData == 0 ) return SQLITE_CANTOPEN;

        p->sz = sqlite3_uri_int64(zName,"sz",0);
        
        if( p->sz < 0 ) return SQLITE_CANTOPEN;
        
        // Set MemFile parameter
        p->szMax = sqlite3_uri_int64(zName,"max",p->sz);
        
        if( p->szMax<p->sz ) return SQLITE_CANTOPEN;

        // This is implemented and active by default
        p->bFreeOnClose = sqlite3_uri_boolean(zName,"freeonclose",0);

        // This is implemented with traits
        pFile->pMethods = &mem_io_methods;
        */
        // TODO figure out how to drop this, store a pointer to the vfs?
        unsafe { *p_file = *create_file_pointer( rust_file ); }
    
        Ok(())
    }

    fn delete(&mut self, z_name: *const c_char, sync_dir: c_int) -> Result<()> {
        Ok(())
    }

    fn access(&mut self, z_name: *const c_char, flags: c_int, p_res_out: *mut c_int) -> Result<()> {
        Ok(())
    }

    fn full_pathname(&mut self, z_name: *const c_char, n_out: c_int, z_out: *mut c_char) -> Result<()> {
        Ok(())
    }

    // From here onwards, only calls to the default vfs
    fn dl_open(&mut self, z_filename: *const c_char) -> *mut c_void {
        self.default_vfs.dl_open(z_filename)
    }

    fn dl_error(&mut self, n_byte: c_int, z_err_msg: *mut c_char) {
        self.default_vfs.dl_error(n_byte, z_err_msg)
    }

    fn dl_sym(&mut self, arg2: *mut c_void, z_symbol: *const c_char) -> Option<unsafe extern "C" fn(arg1: *mut sqlite3_vfs, arg2: *mut c_void, z_symbol: *const c_char)> {
        self.default_vfs.dl_sym(arg2, z_symbol)
    }

    fn dl_close(&mut self, arg2: *mut c_void) {
        self.default_vfs.dl_close(arg2)
    }

    fn randomness(&mut self, n_byte: c_int, z_out: *mut c_char) -> Result<()> {
        self.default_vfs.randomness(n_byte, z_out)
    }

    fn sleep(&mut self, microseconds: c_int) -> Result<()> {
        self.default_vfs.sleep(microseconds)
    }

    fn current_time(&mut self, arg2: *mut f64) -> Result<()> {
        self.default_vfs.current_time(arg2)
    }

    fn get_last_error(&mut self, arg2: c_int, arg3: *mut c_char) -> Result<()> {
        self.default_vfs.get_last_error(arg2, arg3)
    }

    fn current_time_int64(&mut self, arg2: *mut sqlite3_int64) -> i32 {
        self.default_vfs.current_time_int64(arg2)
    }

    fn set_system_call(&mut self, z_name: *const c_char, arg2: sqlite3_syscall_ptr) -> i32 {
        self.default_vfs.set_system_call(z_name, arg2)
    }

    fn get_system_call(&mut self, z_name: *const c_char) -> sqlite3_syscall_ptr {
        self.default_vfs.get_system_call(z_name)
    }

    fn next_system_call(&mut self, z_name: *const c_char) -> *const c_char {
        self.default_vfs.next_system_call(z_name)
    }
}

struct MemFile {
    size: sqlite3_int64, // equal to self.data.len()
    max_size: sqlite3_int64,
    file_content: Vec<u8>,
}

impl SqliteIoMethods for MemFile {
    fn close(&mut self) -> Result<()> {
        // The example contains an explicit deallocation,
        // but the base implementation takes care of that already
        // with a Box::from_raw, that forces the datastructure
        // to drop at the end of the scope
        Ok(())
    }

    fn read(&mut self, buf: *mut c_void, i_amt: c_int, i_ofst: sqlite3_int64) -> Result<()> {
        Ok(())
        /*
        // TODO write requested data to buf
        memcpy(buf, p->aData+iOfst, iAmt);
        */
    }

    fn write(&mut self, buf: *const c_void, i_amt: c_int, i_ofst: sqlite3_int64) -> Result<()> {
        Ok(())
        /*
            if( (iOfst + iAmt) > p->sz ) {
                // Error if exceeds allocation
                if( (iOfst+iAmt) > p->szMax ) {
                    return SQLITE_FULL;
                }
                // Pre-allocate space with memset
                if( iOfst > p->sz ) {
                    memset(p->aData + p->sz, 0, iOfst - p->sz);
                }
                p->sz = iOfst + iAmt;
            }
            // append buf to memory
            memcpy(p->aData + iOfst, buf, iAmt);
            return SQLITE_OK;
        */
    }

    fn truncate(&mut self, size: sqlite3_int64) -> Result<()> {
        // TODO error if allocation is full
        // original:
        /*
            if( size > p->sz ) {
                if( size > p->szMax ) {
                    return SQLITE_FULL;
                }
                memset(p->aData + p->sz, 0, size-p->sz); // double the size
            }
            p->sz = size; 
            return SQLITE_OK;        
        */
        Ok(())
    }

    fn sync(&mut self, flags: c_int) -> Result<()> {
        Ok(())
    }

    fn file_size(&mut self, p_size: *mut sqlite3_int64) -> Result<()> {
        // TODO *p_size = self.file_content.len()
        Ok(())
    }

    fn lock(&mut self, arg2: c_int) -> Result<()> {
        Ok(())
    }

    fn unlock(&mut self, arg2: c_int) -> Result<()> {
        Ok(())
    }

    fn check_reserved_lock(&mut self, p_res_out: *mut c_int) -> Result<()> {
        // TODO OK(()) -> *pResOut = 0
        // TODO consider putting this in a struct
        Ok(())
    }

    fn file_control(&mut self, op: c_int, p_arg: *mut c_void) -> Result<()> {
        Ok(())
        // TODO change type to support this:
        /*
            int rc = SQLITE_NOTFOUND;
            if( op==SQLITE_FCNTL_VFSNAME ){
                *(char**)pArg = sqlite3_mprintf("mem(%p,%lld)", p->aData, p->sz);
                rc = SQLITE_OK;
            }
            // TODO use rust formatting and then create pointers
            return rc;
        */
    }

    fn sector_size(&mut self) -> Result<()> {
        Ok(())
        // TODO change type to support this: 1024
        // TODO consider putting this in a struct
    }

    fn device_characteristics(&mut self) -> Result<()> {
        Ok(())
        // TODO change type to support this
        // TODO consider putting this in a struct
        /*
        SQLITE_IOCAP_ATOMIC | 
         SQLITE_IOCAP_POWERSAFE_OVERWRITE |
         SQLITE_IOCAP_SAFE_APPEND |
         SQLITE_IOCAP_SEQUENTIAL
        */
    }

    fn shm_map(&mut self, i_pg: c_int, pgsz: c_int, arg2: c_int, arg3: *mut *mut c_void) -> Result<()> {
        Err(Error::new(ErrorKind::DefineVfs(SQLITE_IOERR_SHMMAP)))
    }

    fn shm_lock(&mut self, offset: c_int, n: c_int, flags: c_int) -> Result<()> {
        // SQLITE_IOERR_SHMLOCK is deprecated
        Err(Error::new(ErrorKind::DefineVfs(SQLITE_IOERR_SHMLOCK)))
    }

    fn shm_barrier(&mut self) -> Result<()> {
        Ok(())
    }

    fn shm_unmap(&mut self, delete_flag: c_int) -> Result<()> {
        Ok(())
    }

    fn fetch(&mut self, i_ofst: sqlite3_int64, i_amt: c_int, pp: *mut *mut c_void) -> Result<()> {
        // unsafe { *pp = self.file_content + }
        // TODO provide memory location
        Ok(())
    }

    fn unfetch(&mut self, i_ofst: sqlite3_int64, p: *mut c_void) -> Result<()> {
        Ok(())
    }
}


#[sqlite_entrypoint_permanent]
pub fn sqlite3_memvfs_init(db: *mut sqlite3) -> Result<()> {
    let vfs: sqlite3_vfs = create_vfs(
        MemVfs { default_vfs: DefaultVfs::new() }, "memvfs", 1024);
    register_vfs(vfs, true)?;
    Ok(())
}