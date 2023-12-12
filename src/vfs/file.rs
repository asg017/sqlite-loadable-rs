#![allow(non_snake_case)]
#![allow(unused)]

use crate::ext::{sqlite3_file, sqlite3_int64, sqlite3_io_methods};

use sqlite3ext_sys::{
    SQLITE_IOERR_CLOSE, SQLITE_IOERR_FSTAT, SQLITE_IOERR_FSYNC, SQLITE_IOERR_LOCK,
    SQLITE_IOERR_MMAP, SQLITE_IOERR_READ, SQLITE_IOERR_SHMLOCK, SQLITE_IOERR_SHMMAP,
    SQLITE_IOERR_TRUNCATE, SQLITE_IOERR_UNLOCK, SQLITE_IOERR_WRITE,
};

use std::{
    fs::File,
    mem::MaybeUninit,
    os::raw::{c_int, c_void},
};

use crate::vfs::traits::SqliteIoMethods;
use crate::vfs::vfs::handle_error;
use std::io::{Error, ErrorKind, Result};

use super::vfs::handle_int;

// TODO keep a pointer of f and m, then
// This should just close the file, and not do gc
unsafe extern "C" fn x_close<T: SqliteIoMethods>(file: *mut sqlite3_file) -> c_int {
    let mut f = &mut *file.cast::<FileWithAux<T>>();
    let mut aux = f.aux.assume_init_mut();
    let result = aux.close();
    handle_error(result, Some(SQLITE_IOERR_CLOSE))
}

unsafe extern "C" fn x_read<T: SqliteIoMethods>(
    file: *mut sqlite3_file,
    buf: *mut c_void,
    iAmt: c_int,
    iOfst: sqlite3_int64,
) -> c_int {
    let mut f = &mut *file.cast::<FileWithAux<T>>();
    let mut aux = f.aux.assume_init_mut();
    let result = aux.read(buf, iAmt, iOfst);
    handle_error(result, Some(SQLITE_IOERR_READ))
}

unsafe extern "C" fn x_write<T: SqliteIoMethods>(
    file: *mut sqlite3_file,
    buf: *const c_void,
    iAmt: c_int,
    iOfst: sqlite3_int64,
) -> c_int {
    let mut f = &mut *file.cast::<FileWithAux<T>>();
    let mut aux = f.aux.assume_init_mut();
    let result = aux.write(buf, iAmt, iOfst);
    handle_error(result, Some(SQLITE_IOERR_WRITE))
}

unsafe extern "C" fn x_truncate<T: SqliteIoMethods>(
    file: *mut sqlite3_file,
    size: sqlite3_int64,
) -> c_int {
    let mut f = &mut *file.cast::<FileWithAux<T>>();
    let mut aux = f.aux.assume_init_mut();
    let result = aux.truncate(size);
    handle_error(result, Some(SQLITE_IOERR_TRUNCATE))
}

unsafe extern "C" fn x_sync<T: SqliteIoMethods>(file: *mut sqlite3_file, flags: c_int) -> c_int {
    let mut f = &mut *file.cast::<FileWithAux<T>>();
    let mut aux = f.aux.assume_init_mut();
    let result = aux.sync(flags);
    handle_error(result, Some(SQLITE_IOERR_FSYNC))
}

unsafe extern "C" fn x_file_size<T: SqliteIoMethods>(
    file: *mut sqlite3_file,
    pSize: *mut sqlite3_int64,
) -> c_int {
    let mut f = &mut *file.cast::<FileWithAux<T>>();
    let mut aux = f.aux.assume_init_mut();
    let result = aux.file_size(pSize);
    handle_error(result, Some(SQLITE_IOERR_FSTAT))
}

unsafe extern "C" fn x_lock<T: SqliteIoMethods>(file: *mut sqlite3_file, arg2: c_int) -> c_int {
    let mut f = &mut *file.cast::<FileWithAux<T>>();
    let mut aux = f.aux.assume_init_mut();
    let result = aux.lock(arg2);
    handle_int(result, Some(SQLITE_IOERR_LOCK))
}

unsafe extern "C" fn x_unlock<T: SqliteIoMethods>(file: *mut sqlite3_file, arg2: c_int) -> c_int {
    let mut f = &mut *file.cast::<FileWithAux<T>>();
    let mut aux = f.aux.assume_init_mut();
    let result = aux.unlock(arg2);
    handle_int(result, Some(SQLITE_IOERR_UNLOCK))
}

unsafe extern "C" fn x_check_reserved_lock<T: SqliteIoMethods>(
    file: *mut sqlite3_file,
    pResOut: *mut c_int,
) -> c_int {
    let mut f = &mut *file.cast::<FileWithAux<T>>();
    let mut aux = f.aux.assume_init_mut();
    let result = aux.check_reserved_lock(pResOut);
    handle_error(result, None)
}

unsafe extern "C" fn x_file_control<T: SqliteIoMethods>(
    file: *mut sqlite3_file,
    op: c_int,
    pArg: *mut c_void,
) -> c_int {
    let mut f = &mut *file.cast::<FileWithAux<T>>();
    let mut aux = f.aux.assume_init_mut();
    let result = aux.file_control(op, pArg);
    handle_error(result, None)
}

unsafe extern "C" fn x_sector_size<T: SqliteIoMethods>(file: *mut sqlite3_file) -> c_int {
    let mut f = &mut *file.cast::<FileWithAux<T>>();
    let mut aux = f.aux.assume_init_mut();
    let result = aux.sector_size();
    handle_int(result, None)
}

unsafe extern "C" fn x_device_characteristics<T: SqliteIoMethods>(
    file: *mut sqlite3_file,
) -> c_int {
    let mut f = &mut *file.cast::<FileWithAux<T>>();
    let mut aux = f.aux.assume_init_mut();
    let result = aux.device_characteristics();
    handle_int(result, None)
}

unsafe extern "C" fn x_shm_map<T: SqliteIoMethods>(
    file: *mut sqlite3_file,
    iPg: c_int,
    pgsz: c_int,
    arg2: c_int,
    arg3: *mut *mut c_void,
) -> c_int {
    let mut f = &mut *file.cast::<FileWithAux<T>>();
    let mut aux = f.aux.assume_init_mut();
    let result = aux.shm_map(iPg, pgsz, arg2, arg3);
    handle_error(result, Some(SQLITE_IOERR_SHMMAP))
}

unsafe extern "C" fn x_shm_lock<T: SqliteIoMethods>(
    file: *mut sqlite3_file,
    offset: c_int,
    n: c_int,
    flags: c_int,
) -> c_int {
    let mut f = &mut *file.cast::<FileWithAux<T>>();
    let mut aux = f.aux.assume_init_mut();
    let result = aux.shm_lock(offset, n, flags);
    handle_error(result, Some(SQLITE_IOERR_SHMLOCK))
}

unsafe extern "C" fn x_shm_barrier<T: SqliteIoMethods>(file: *mut sqlite3_file) {
    let mut f = &mut *file.cast::<FileWithAux<T>>();
    let mut aux = f.aux.assume_init_mut();
    let result = aux.shm_barrier();
}

unsafe extern "C" fn x_shm_unmap<T: SqliteIoMethods>(
    file: *mut sqlite3_file,
    deleteFlag: c_int,
) -> c_int {
    let mut f = &mut *file.cast::<FileWithAux<T>>();
    let mut aux = f.aux.assume_init_mut();
    let result = aux.shm_unmap(deleteFlag);
    handle_error(result, Some(SQLITE_IOERR_SHMMAP))
}

unsafe extern "C" fn x_fetch<T: SqliteIoMethods>(
    file: *mut sqlite3_file,
    iOfst: sqlite3_int64,
    iAmt: c_int,
    pp: *mut *mut c_void,
) -> c_int {
    let mut f = &mut *file.cast::<FileWithAux<T>>();
    let mut aux = f.aux.assume_init_mut();
    let result = aux.fetch(iOfst, iAmt, pp);
    handle_error(result, None)
}

unsafe extern "C" fn x_unfetch<T: SqliteIoMethods>(
    file: *mut sqlite3_file,
    iOfst: sqlite3_int64,
    p: *mut c_void,
) -> c_int {
    let mut f = &mut *file.cast::<FileWithAux<T>>();
    let mut aux = f.aux.assume_init_mut();
    let result = aux.unfetch(iOfst, p);
    handle_error(result, None)
}

#[repr(C)]
pub struct FileWithAux<T: SqliteIoMethods> {
    pub pMethods: Box<sqlite3_io_methods>,
    pub aux: MaybeUninit<T>,
}

/// See sqlite3OsOpenMalloc and sqlite3OsCloseFree dependency on szOsFile on sqlite3_vfs,
/// this implies that ownership of sqlite3_file and any "sub-type", is with sqlite3
pub unsafe fn prepare_file_ptr<T: SqliteIoMethods>(
    file_ptr: *mut sqlite3_file,
    aux: T,
) -> *const sqlite3_file {
    let f = (file_ptr as *mut FileWithAux<T>).as_mut().unwrap();
    f.pMethods = create_io_methods_boxed::<T>();
    f.aux.write(aux);

    file_ptr
}

pub fn create_io_methods_boxed<T: SqliteIoMethods>() -> Box<sqlite3_io_methods> {
    let m = sqlite3_io_methods {
        iVersion: 3, // this library targets version 3?
        xClose: Some(x_close::<T>),
        xRead: Some(x_read::<T>),
        xWrite: Some(x_write::<T>),
        xTruncate: Some(x_truncate::<T>),
        xSync: Some(x_sync::<T>),
        xFileSize: Some(x_file_size::<T>),
        xLock: Some(x_lock::<T>),
        xUnlock: Some(x_unlock::<T>),
        xCheckReservedLock: Some(x_check_reserved_lock::<T>),
        xFileControl: Some(x_file_control::<T>),
        xSectorSize: Some(x_sector_size::<T>),
        xDeviceCharacteristics: Some(x_device_characteristics::<T>),
        xShmMap: Some(x_shm_map::<T>),
        xShmLock: Some(x_shm_lock::<T>),
        xShmBarrier: Some(x_shm_barrier::<T>),
        xShmUnmap: Some(x_shm_unmap::<T>),
        xFetch: Some(x_fetch::<T>),
        xUnfetch: Some(x_unfetch::<T>),
    };
    Box::new(m)
}

// TODO determine false positive: Valgrind reports mismatch malloc/free? 16B
// VALGRINDFLAGS="--leak-check=full --trace-children=yes --verbose --log-file=leaky.txt" cargo valgrind test
