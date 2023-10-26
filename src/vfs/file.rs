#![allow(non_snake_case)]
#![allow(unused)]

use sqlite3ext_sys::{
    sqlite3_file, sqlite3_int64, sqlite3_io_methods, SQLITE_IOERR_CLOSE, SQLITE_IOERR_FSTAT,
    SQLITE_IOERR_FSYNC, SQLITE_IOERR_LOCK, SQLITE_IOERR_MMAP, SQLITE_IOERR_READ,
    SQLITE_IOERR_SHMLOCK, SQLITE_IOERR_SHMMAP, SQLITE_IOERR_TRUNCATE, SQLITE_IOERR_UNLOCK,
    SQLITE_IOERR_WRITE,
};
use std::os::raw::{c_int, c_void};

use crate::vfs::traits::SqliteIoMethods;
use crate::vfs::vfs::handle_error;
use std::io::{Error, ErrorKind, Result};

use super::vfs::handle_int;

/// Let aux and methods Boxes go out of scope, thus drop,
unsafe extern "C" fn x_close<T: SqliteIoMethods>(file: *mut sqlite3_file) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(file.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let result = m.aux.close(file);
    // disabling reduces leak to 16B,  also fixes free-ing invalid pointer, on rusql, sqlite3 just crashes
    // Box::into_raw(m);

    // disabling crashes valgrind, reason: stack smashing, and free invalid pointer
    // otherwise 8 bytes leak
    Box::into_raw(f);

    // Disabling both fails the unit tests, free(): invalid pointer
    handle_error(result, Some(SQLITE_IOERR_CLOSE))
}

unsafe extern "C" fn x_read<T: SqliteIoMethods>(
    file: *mut sqlite3_file,
    buf: *mut c_void,
    iAmt: c_int,
    iOfst: sqlite3_int64,
) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(file.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let result = m.aux.read(file, buf, iAmt, iOfst);
    Box::into_raw(f);
    Box::into_raw(m);
    handle_error(result, Some(SQLITE_IOERR_READ))
}

unsafe extern "C" fn x_write<T: SqliteIoMethods>(
    file: *mut sqlite3_file,
    buf: *const c_void,
    iAmt: c_int,
    iOfst: sqlite3_int64,
) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(file.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let result = m.aux.write(file, buf, iAmt, iOfst);
    Box::into_raw(f);
    Box::into_raw(m);
    handle_error(result, Some(SQLITE_IOERR_WRITE))
}

unsafe extern "C" fn x_truncate<T: SqliteIoMethods>(
    file: *mut sqlite3_file,
    size: sqlite3_int64,
) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(file.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let result = m.aux.truncate(file, size);
    Box::into_raw(f);
    Box::into_raw(m);
    handle_error(result, Some(SQLITE_IOERR_TRUNCATE))
}

unsafe extern "C" fn x_sync<T: SqliteIoMethods>(file: *mut sqlite3_file, flags: c_int) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(file.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let result = m.aux.sync(file, flags);
    Box::into_raw(f);
    Box::into_raw(m);
    handle_error(result, Some(SQLITE_IOERR_FSYNC))
}

unsafe extern "C" fn x_file_size<T: SqliteIoMethods>(
    file: *mut sqlite3_file,
    pSize: *mut sqlite3_int64,
) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(file.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let result = m.aux.file_size(file, pSize);
    Box::into_raw(f);
    Box::into_raw(m);
    handle_error(result, Some(SQLITE_IOERR_FSTAT))
}

unsafe extern "C" fn x_lock<T: SqliteIoMethods>(file: *mut sqlite3_file, arg2: c_int) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(file.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let result = m.aux.lock(file, arg2);
    Box::into_raw(f);
    Box::into_raw(m);
    handle_int(result, Some(SQLITE_IOERR_LOCK))
}

unsafe extern "C" fn x_unlock<T: SqliteIoMethods>(file: *mut sqlite3_file, arg2: c_int) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(file.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let result = m.aux.unlock(file, arg2);
    Box::into_raw(f);
    Box::into_raw(m);
    handle_int(result, Some(SQLITE_IOERR_UNLOCK))
}

unsafe extern "C" fn x_check_reserved_lock<T: SqliteIoMethods>(
    file: *mut sqlite3_file,
    pResOut: *mut c_int,
) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(file.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let result = m.aux.check_reserved_lock(file, pResOut);
    Box::into_raw(f);
    Box::into_raw(m);
    handle_error(result, None)
}

unsafe extern "C" fn x_file_control<T: SqliteIoMethods>(
    file: *mut sqlite3_file,
    op: c_int,
    pArg: *mut c_void,
) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(file.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let result = m.aux.file_control(file, op, pArg);
    Box::into_raw(f);
    Box::into_raw(m);
    handle_error(result, None)
}

unsafe extern "C" fn x_sector_size<T: SqliteIoMethods>(file: *mut sqlite3_file) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(file.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let result = m.aux.sector_size(file);
    Box::into_raw(f);
    Box::into_raw(m);
    handle_int(result, None)
}

unsafe extern "C" fn x_device_characteristics<T: SqliteIoMethods>(
    file: *mut sqlite3_file,
) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(file.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let result = m.aux.device_characteristics(file);
    Box::into_raw(f);
    Box::into_raw(m);
    handle_int(result, None)
}

unsafe extern "C" fn x_shm_map<T: SqliteIoMethods>(
    file: *mut sqlite3_file,
    iPg: c_int,
    pgsz: c_int,
    arg2: c_int,
    arg3: *mut *mut c_void,
) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(file.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let result = m.aux.shm_map(file, iPg, pgsz, arg2, arg3);
    Box::into_raw(f);
    Box::into_raw(m);
    handle_error(result, Some(SQLITE_IOERR_SHMMAP))
}

unsafe extern "C" fn x_shm_lock<T: SqliteIoMethods>(
    file: *mut sqlite3_file,
    offset: c_int,
    n: c_int,
    flags: c_int,
) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(file.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let result = m.aux.shm_lock(file, offset, n, flags);
    Box::into_raw(f);
    Box::into_raw(m);
    handle_error(result, Some(SQLITE_IOERR_SHMLOCK))
}

unsafe extern "C" fn x_shm_barrier<T: SqliteIoMethods>(file: *mut sqlite3_file) {
    let mut f = Box::<FileWithAux<T>>::from_raw(file.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());

    m.aux.shm_barrier(file);

    Box::into_raw(f);
    Box::into_raw(m);
}

unsafe extern "C" fn x_shm_unmap<T: SqliteIoMethods>(
    file: *mut sqlite3_file,
    deleteFlag: c_int,
) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(file.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());

    let result = m.aux.shm_unmap(file, deleteFlag);
    Box::into_raw(f);
    Box::into_raw(m);
    handle_error(result, None)
}

unsafe extern "C" fn x_fetch<T: SqliteIoMethods>(
    file: *mut sqlite3_file,
    iOfst: sqlite3_int64,
    iAmt: c_int,
    pp: *mut *mut c_void,
) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(file.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());

    let result = m.aux.fetch(file, iOfst, iAmt, pp);
    Box::into_raw(f);
    Box::into_raw(m);
    handle_error(result, Some(SQLITE_IOERR_MMAP))
}

unsafe extern "C" fn x_unfetch<T: SqliteIoMethods>(
    file: *mut sqlite3_file,
    iOfst: sqlite3_int64,
    p: *mut c_void,
) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(file.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());

    let result = m.aux.unfetch(file, iOfst, p);
    Box::into_raw(f);
    Box::into_raw(m);
    handle_error(result, Some(SQLITE_IOERR_MMAP))
}

// C struct polymorphism, given the alignment and field sequence are the same
#[repr(C)]
pub struct FileWithAux<T: SqliteIoMethods>(*const MethodsWithAux<T>);

unsafe fn create_io_methods<T: SqliteIoMethods>(aux: T) -> MethodsWithAux<T> {
    MethodsWithAux {
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
        aux,
    }
}

pub fn create_file_pointer<T: SqliteIoMethods>(actual_methods: T) -> *mut sqlite3_file {
    unsafe {
        let methods = create_io_methods::<T>(actual_methods);
        let methods_ptr = Box::into_raw(Box::new(methods));

        let p = FileWithAux::<T>(methods_ptr);

        // TODO determine false positive: Valgrind reports mismatch malloc/free? 16B
        // VALGRINDFLAGS="--leak-check=full --trace-children=yes --verbose --log-file=leaky.txt" cargo valgrind test
        Box::into_raw(Box::new(p)).cast()
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MethodsWithAux<T: SqliteIoMethods> {
    pub iVersion: ::std::os::raw::c_int,
    pub xClose: ::std::option::Option<
        unsafe extern "C" fn(file: *mut sqlite3_file) -> ::std::os::raw::c_int,
    >,
    pub xRead: ::std::option::Option<
        unsafe extern "C" fn(
            file: *mut sqlite3_file,
            arg2: *mut ::std::os::raw::c_void,
            iAmt: ::std::os::raw::c_int,
            iOfst: sqlite3_int64,
        ) -> ::std::os::raw::c_int,
    >,
    pub xWrite: ::std::option::Option<
        unsafe extern "C" fn(
            file: *mut sqlite3_file,
            arg2: *const ::std::os::raw::c_void,
            iAmt: ::std::os::raw::c_int,
            iOfst: sqlite3_int64,
        ) -> ::std::os::raw::c_int,
    >,
    pub xTruncate: ::std::option::Option<
        unsafe extern "C" fn(file: *mut sqlite3_file, size: sqlite3_int64) -> ::std::os::raw::c_int,
    >,
    pub xSync: ::std::option::Option<
        unsafe extern "C" fn(
            file: *mut sqlite3_file,
            flags: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub xFileSize: ::std::option::Option<
        unsafe extern "C" fn(
            file: *mut sqlite3_file,
            pSize: *mut sqlite3_int64,
        ) -> ::std::os::raw::c_int,
    >,
    pub xLock: ::std::option::Option<
        unsafe extern "C" fn(
            file: *mut sqlite3_file,
            arg2: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub xUnlock: ::std::option::Option<
        unsafe extern "C" fn(
            file: *mut sqlite3_file,
            arg2: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub xCheckReservedLock: ::std::option::Option<
        unsafe extern "C" fn(
            file: *mut sqlite3_file,
            pResOut: *mut ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub xFileControl: ::std::option::Option<
        unsafe extern "C" fn(
            file: *mut sqlite3_file,
            op: ::std::os::raw::c_int,
            pArg: *mut ::std::os::raw::c_void,
        ) -> ::std::os::raw::c_int,
    >,
    pub xSectorSize: ::std::option::Option<
        unsafe extern "C" fn(file: *mut sqlite3_file) -> ::std::os::raw::c_int,
    >,
    pub xDeviceCharacteristics: ::std::option::Option<
        unsafe extern "C" fn(file: *mut sqlite3_file) -> ::std::os::raw::c_int,
    >,
    // shm = shared memory
    pub xShmMap: ::std::option::Option<
        unsafe extern "C" fn(
            file: *mut sqlite3_file,
            iPg: ::std::os::raw::c_int,
            pgsz: ::std::os::raw::c_int,
            arg2: ::std::os::raw::c_int,
            arg3: *mut *mut ::std::os::raw::c_void,
        ) -> ::std::os::raw::c_int,
    >,
    pub xShmLock: ::std::option::Option<
        unsafe extern "C" fn(
            file: *mut sqlite3_file,
            offset: ::std::os::raw::c_int,
            n: ::std::os::raw::c_int,
            flags: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub xShmBarrier: ::std::option::Option<unsafe extern "C" fn(file: *mut sqlite3_file)>,
    pub xShmUnmap: ::std::option::Option<
        unsafe extern "C" fn(
            file: *mut sqlite3_file,
            deleteFlag: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub xFetch: ::std::option::Option<
        unsafe extern "C" fn(
            file: *mut sqlite3_file,
            iOfst: sqlite3_int64,
            iAmt: ::std::os::raw::c_int,
            pp: *mut *mut ::std::os::raw::c_void,
        ) -> ::std::os::raw::c_int,
    >,
    pub xUnfetch: ::std::option::Option<
        unsafe extern "C" fn(
            file: *mut sqlite3_file,
            iOfst: sqlite3_int64,
            p: *mut ::std::os::raw::c_void,
        ) -> ::std::os::raw::c_int,
    >,
    pub aux: T,
}
