#![allow(unused)]

use libsqlite3_sys::{SQLITE_IOERR_SHMMAP, SQLITE_IOERR_SHMLOCK};
use sqlite_loadable::vfs::default::DefaultVfs;
use sqlite_loadable::vfs::vfs::create_vfs;

use sqlite_loadable::{prelude::*, SqliteIoMethods, create_file_pointer, register_vfs, Error, ErrorKind, define_scalar_function, api};
use sqlite_loadable::{Result, vfs::traits::SqliteVfs};
use url::Url;

use std::collections::HashMap;
use std::ffi::{CString, CStr};
use std::fs::{File, self};
use std::io::{Write, Read};
use std::os::raw::{c_int, c_void, c_char};
use std::ptr;
use sqlite3ext_sys::{sqlite3_int64, sqlite3_syscall_ptr, sqlite3_file, sqlite3_vfs, sqlite3_vfs_register, sqlite3_io_methods, sqlite3_vfs_find, sqlite3_context_db_handle, sqlite3_file_control};
use libsqlite3_sys::{SQLITE_CANTOPEN, SQLITE_OPEN_MAIN_DB, SQLITE_IOERR_DELETE};
use libsqlite3_sys::{SQLITE_IOCAP_ATOMIC, SQLITE_IOCAP_POWERSAFE_OVERWRITE,
    SQLITE_IOCAP_SAFE_APPEND, SQLITE_IOCAP_SEQUENTIAL};
use libsqlite3_sys::{sqlite3_snprintf, sqlite3_mprintf};

/// Inspired by https://www.sqlite.org/src/file/ext/misc/memvfs.c
struct MemVfs {
    default_vfs: DefaultVfs,
}

const SIZE_LABEL: &str = "size";
const POINTER_LABEL: &str = "pointer";

impl SqliteVfs for MemVfs {
    fn open(&mut self, z_name: *const c_char, p_file: *mut sqlite3_file, flags: c_int, p_res_out: *mut c_int) -> Result<()> {
        let mut rust_file = MemFile {
            file_contents: Vec::new()
        };
        
        /*
        memset(p, 0, sizeof(*p));

        if( (flags & SQLITE_OPEN_MAIN_DB) == 0 ) return SQLITE_CANTOPEN;
        */

        let cant_open = Err(Error::new(ErrorKind::DefineVfs(SQLITE_CANTOPEN)));

        let uri_cstr = unsafe { CStr::from_ptr(z_name) };
        let uri_str = uri_cstr.to_str();
        let parsed_uri = Url::parse(uri_str.expect("should be a valid uri"));

        if (flags & SQLITE_OPEN_MAIN_DB) == 0 {
            return cant_open;
        }

        /*
            p->aData = (unsigned char*)sqlite3_uri_int64(zName,"ptr",0);

            if( p->aData == 0 ) return SQLITE_CANTOPEN;

            p->sz = sqlite3_uri_int64(zName,"sz",0);
            
            if( p->sz < 0 ) return SQLITE_CANTOPEN;
            
            // Set MemFile parameter
            p->szMax = sqlite3_uri_int64(zName,"max",p->sz);
            
            if( p->szMax<p->sz ) return SQLITE_CANTOPEN;
        */

        if let Ok(url) = parsed_uri {
            let mut size: usize = 0;

            let mut query_map: HashMap<String, String> = HashMap::new();
            for (key, value) in url.query_pairs() {
                query_map.insert(key.to_string(), value.to_string());

                if key == SIZE_LABEL {
                    size = value.parse().expect("should be an int");
                }
                if key == POINTER_LABEL {
                    // Parse the ptr value as a u64 hexadecimal address
                    if let Ok(ptr_address) = u64::from_str_radix(&value, 16) {
                        // Assuming ptr_address is a valid memory address, you can read its contents here.
                        let buffer = 
                            unsafe { std::slice::from_raw_parts(ptr_address as *const u8, size) };

                        rust_file.file_contents = buffer.to_vec();
                    }
                }
            }

            if 
                !query_map.contains_key(SIZE_LABEL) &&
                !query_map.contains_key(POINTER_LABEL) {
                return cant_open;
            }

        } else {
            return cant_open;
        }
        
        // Skipped 'freeonclose' parameter', dropping is more idiomatic
        /*
        // This is implemented and active buy default
        p->bFreeOnClose = sqlite3_uri_boolean(zName,"freeonclose",0);

        // This is implemented with traits
        pFile->pMethods = &mem_io_methods;
        */

        // TODO figure out how to drop this, store a pointer to the vfs?
        unsafe { *p_file = *create_file_pointer( rust_file ); }
    
        Ok(())
    }

    fn delete(&mut self, z_name: *const c_char, sync_dir: c_int) -> Result<()> {
        Err(Error::new(ErrorKind::DefineVfs(SQLITE_IOERR_DELETE)))
    }

    fn access(&mut self, z_name: *const c_char, flags: c_int, p_res_out: *mut c_int) -> Result<()> {
        unsafe {
            *p_res_out = 0;
        }
        Ok(())
    }

    fn full_pathname(&mut self, z_name: *const c_char, n_out: c_int, z_out: *mut c_char) -> Result<()> {
        // TODO see if format! is actually easier and less unsafe:
        // ...format!("{}", CString::new())...
        unsafe { sqlite3_snprintf(n_out, z_out, CString::new("%s").expect("should be  fine").clone().as_ptr(), z_name); }
        Ok(())
    }

    /// From here onwards, all calls are redirected to the default vfs
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

    fn randomness(&mut self, n_byte: c_int, z_out: *mut c_char) -> c_int {
         self.default_vfs.randomness(n_byte, z_out)
    }

    fn sleep(&mut self, microseconds: c_int) -> c_int {
        self.default_vfs.sleep(microseconds)
    }

    fn current_time(&mut self, arg2: *mut f64) -> c_int {
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
    file_contents: Vec<u8>,
}

impl SqliteIoMethods for MemFile {
    /// The original example contains an explicit deallocation,
    /// but the base implementation takes care of that already
    /// with a Box::from_raw, that forces the datastructure
    /// to drop at the end of the scope
    fn close(&mut self) -> Result<()> {
        Ok(())
    }

    fn read(&mut self, buf: *mut c_void, size: usize, offset: usize) -> Result<()> {
        /*
        memcpy(buf, p->aData+iOfst, iAmt);
        */

        let source = &mut self.file_contents;

        // TODO do not assume alignment is correct, check
        unsafe {
            let src_ptr = source.as_ptr().offset(offset as isize);
            let dst_ptr = buf;
            ptr::copy_nonoverlapping(src_ptr, dst_ptr.cast(), size);
        }

        Ok(())
    }

    fn write(&mut self, buf: *const c_void, size: usize, offset: usize) -> Result<()> {
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
        let new_length = size + offset;
        if new_length > self.file_contents.len() {
            self.file_contents.resize(new_length, 0);
        }

        // Get a mutable pointer to the destination data
        let dest_ptr = self.file_contents.as_mut_ptr();

        // Use copy_from_nonoverlapping to copy data from source to dest
        unsafe {
            ptr::copy_nonoverlapping(buf.offset(offset as isize), dest_ptr.cast(), size);
            self.file_contents.set_len(new_length)
        };

        Ok(())
    }

    fn truncate(&mut self, size: usize) -> Result<()> {
        // original:
        /*
            if( size > p->sz ) {
                if( size > p->szMax ) {
                    return SQLITE_FULL;
                }
                memset(p->aData + p->sz, 0, size-p->sz); // extend to what is required
            }
            p->sz = size; 
            return SQLITE_OK;        
        */

        self.file_contents.resize(size, 0);

        Ok(())
    }

    fn sync(&mut self, flags: c_int) -> Result<()> {
        Ok(())
    }

    fn file_size(&mut self, p_size: *mut sqlite3_int64) -> Result<()> {
        unsafe { *p_size = self.file_contents.len().try_into().unwrap(); }
        Ok(())
    }

    fn lock(&mut self, arg2: c_int) -> Result<()> {
        Ok(())
    }

    fn unlock(&mut self, arg2: c_int) -> Result<()> {
        Ok(())
    }

    fn check_reserved_lock(&mut self, p_res_out: *mut c_int) -> Result<()> {
        // *pResOut = 0
        unsafe{ *p_res_out = 0; }
        // TODO consider putting this in a struct
        Ok(())
    }

    fn file_control(&mut self, op: c_int, p_arg: *mut c_void) -> Result<()> {
        /*
            int rc = SQLITE_NOTFOUND;
            if( op==SQLITE_FCNTL_VFSNAME ){
                *(char**)pArg = sqlite3_mprintf("mem(%p,%lld)", p->aData, p->sz);
                rc = SQLITE_OK;
            }
            // TODO use rust formatting and then create pointers
            return rc;
        */
        // TODO see if format! is actually easier and less unsafe:
        // ...format!("{}", CString::new())...
        unsafe {
            let new_args: *mut c_char = sqlite3_mprintf(CString::new("%p,%lld").expect("should be  fine").clone().as_ptr(), self.file_contents.as_ptr(), self.file_contents.len());
            let out: *mut *mut char = p_arg.cast();
            *out = new_args.cast(); // TODO test with scalar functions
        }

        Ok(())
    }

    fn sector_size(&mut self) -> c_int {
        1024
        // TODO consider putting this in a struct
    }

    fn device_characteristics(&mut self) -> c_int {
        // TODO consider putting this in a struct
        SQLITE_IOCAP_ATOMIC | 
        SQLITE_IOCAP_POWERSAFE_OVERWRITE |
        SQLITE_IOCAP_SAFE_APPEND |
        SQLITE_IOCAP_SEQUENTIAL
    }

    fn shm_map(&mut self, i_pg: c_int, pgsz: c_int, arg2: c_int, arg3: *mut *mut c_void) -> Result<()> {
        Err(Error::new(ErrorKind::DefineVfs(SQLITE_IOERR_SHMMAP)))
    }

    fn shm_lock(&mut self, offset: c_int, n: c_int, flags: c_int) -> Result<()> {
        // SQLITE_IOERR_SHMLOCK is deprecated?
        Err(Error::new(ErrorKind::DefineVfs(SQLITE_IOERR_SHMLOCK)))
    }

    fn shm_barrier(&mut self) -> Result<()> {
        Ok(())
    }

    fn shm_unmap(&mut self, delete_flag: c_int) -> Result<()> {
        Ok(())
    }

    fn fetch(&mut self, offset: usize, size: usize, pp: *mut *mut c_void) -> Result<()> {
        // orig: *pp = (void*)(p->aData + iOfst);
        let memory_location = self.file_contents.as_mut_ptr();
        unsafe { *pp = memory_location.add(offset).cast(); }
        Ok(())
    }

    fn unfetch(&mut self, i_ofst: usize, p: *mut c_void) -> Result<()> {
        Ok(())
    }
}

/// Usage: "ATTACH memvfs_from_file('test.db') AS inmem;"
fn vfs_from_file(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let path = api::value_text(&values[0]).map_err(|_| Error::new_message("can't determine path arg"))?;
    
    let metadata = fs::metadata(path).map_err(|_| Error::new_message("can't determine file size"))?;
    let file_size = metadata.len() as usize;

    let mut file = File::open(path).map_err(|_| Error::new_message("can't open file"))?;
    let mut file_contents: Vec<u8> = Vec::with_capacity(file_size);
    file.read_to_end(&mut file_contents).map_err(|_| Error::new_message("can't read to the end"))?;
    
    let mut heap_buffer: Box<[u8]> = vec![0; file_size].into_boxed_slice();
    unsafe {
        ptr::copy_nonoverlapping(file_contents.as_ptr(), heap_buffer.as_mut_ptr(), file_size);
    }

    let box_ptr = Box::into_raw(heap_buffer);

    let address_str = format!("{:p}", ptr::addr_of!(box_ptr));

    // TODO memory passed here might leak

    let text_output = format!("file://mem?vfs=memvfs&{}={}&{}={}", POINTER_LABEL, address_str, SIZE_LABEL, file_size);

    api::result_text(context, text_output);

    Ok(())
}

fn vfs_to_file(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let path = api::value_text(&values[0]).map_err(|_| Error::new_message("can't determine path arg"))?;
    
    let mut file = File::create(path).map_err(|_| Error::new_message("can't create file"))?;
    let mut vfs_file_ptr: *mut MemFile = ptr::null_mut();

    let db = unsafe { sqlite3_context_db_handle(context) };

    // ? is more idiomatic, but this shouldn't fail
    let schema = CString::new("memvfs").expect("should be a valid name");
    let schema_ptr = schema.as_ptr();

    // workaround for bindings.rs generated with the wrong type
    const SQLITE_FCNTL_FILE_POINTER: i32 = 7;

    unsafe { sqlite3_file_control(db, schema_ptr, SQLITE_FCNTL_FILE_POINTER, vfs_file_ptr.cast()) };

    let file_contents = &(unsafe { &*vfs_file_ptr }).file_contents;

    file.write_all(&file_contents).map_err(|_| Error::new_message("can't write to file"))?;

    file.flush().map_err(|_| Error::new_message("can't flush file"))?;

    // TODO really check for memory leaks
    Ok(())
}

#[sqlite_entrypoint_permanent]
pub fn sqlite3_memvfs_init(db: *mut sqlite3) -> Result<()> {
    let vfs: sqlite3_vfs = create_vfs(
        MemVfs { default_vfs: DefaultVfs::new() }, "memvfs", 1024);
    register_vfs(vfs, true)?;

    let flags = FunctionFlags::UTF8 | FunctionFlags::DETERMINISTIC;
    define_scalar_function(db, "memvfs_from_file", 1, vfs_from_file, flags)?;
    define_scalar_function(db, "memvfs_to_file", 1, vfs_to_file, flags)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use rusqlite::{ffi::sqlite3_auto_extension, Connection};

    #[test]
    fn test_rusqlite_auto_extension() {
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite3_memvfs_init as *const (),
            )));
        }

        let conn = Connection::open_in_memory().unwrap();

        conn.execute("ATTACH memvfs_from_file('from.db') AS inmem;", ());

        conn.execute("CREATE TABLE t3(x, y)", ());
        conn.execute("INSERT INTO t3 VALUES('a', 4),('b', 5),('c', 3),('d', 8),('e', 1)", ());

        conn.execute("memvfs_to_file('to.db')", ());
    }
}