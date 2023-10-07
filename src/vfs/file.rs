#![allow(non_snake_case)] 
#![allow(unused)] 

use sqlite3ext_sys::{sqlite3_file, sqlite3_int64, sqlite3_io_methods};
use std::{os::raw::{c_int, c_void}};

use crate::{vfs::traits::SqliteIoMethods};
use crate::{Error, Result, ErrorKind};
use crate::vfs::vfs::handle_error;

/// Let aux and methods Boxes go out of scope, thus drop,
/// valgrind flags a false positive(?) on x_open
/// sqlite3 clean up the file itself
unsafe extern "C" fn x_close<T: SqliteIoMethods>(arg1: *mut sqlite3_file) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(arg1.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let mut aux = Box::<T>::from_raw(m.aux.cast());
    let result = aux.close();
    Box::into_raw(f);
    handle_error(result)
}

unsafe extern "C" fn x_read<T: SqliteIoMethods>(
    arg1: *mut sqlite3_file,
    buf: *mut c_void,
    iAmt: c_int,
    iOfst: sqlite3_int64,
) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(arg1.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let mut aux = Box::<T>::from_raw(m.aux.cast());
    let result = aux.read(buf, iAmt, iOfst);
    Box::into_raw(f);
    Box::into_raw(m);
    Box::into_raw(aux);
    handle_error(result)
}


unsafe extern "C" fn x_write<T: SqliteIoMethods>(
    arg1: *mut sqlite3_file,
    buf: *const c_void,
    iAmt: c_int,
    iOfst: sqlite3_int64,
) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(arg1.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let mut aux = Box::<T>::from_raw(m.aux.cast());
    let result = aux.write(buf, iAmt, iOfst);
    Box::into_raw(f);
    Box::into_raw(m);
    Box::into_raw(aux);
    handle_error(result)
}


unsafe extern "C" fn x_truncate<T: SqliteIoMethods>(
    arg1: *mut sqlite3_file,
    size: sqlite3_int64,
) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(arg1.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let mut aux = Box::<T>::from_raw(m.aux.cast());
    let result = aux.truncate(size);
    Box::into_raw(f);
    Box::into_raw(m);
    Box::into_raw(aux);
    handle_error(result)
}


unsafe extern "C" fn x_sync<T: SqliteIoMethods>(
    arg1: *mut sqlite3_file,
    flags: c_int,
) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(arg1.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let mut aux = Box::<T>::from_raw(m.aux.cast());
    let result = aux.sync(flags);
    Box::into_raw(f);
    Box::into_raw(m);
    Box::into_raw(aux);
    handle_error(result)
}


unsafe extern "C" fn x_file_size<T: SqliteIoMethods>(
    arg1: *mut sqlite3_file,
    pSize: *mut sqlite3_int64,
) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(arg1.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let mut aux = Box::<T>::from_raw(m.aux.cast());
    let result = aux.file_size(pSize);
    Box::into_raw(f);
    Box::into_raw(m);
    Box::into_raw(aux);
    handle_error(result)
}

unsafe extern "C" fn x_lock<T: SqliteIoMethods>(
    arg1: *mut sqlite3_file,
    arg2: c_int,
) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(arg1.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let mut aux = Box::<T>::from_raw(m.aux.cast());
    let result = aux.lock(arg2);
    Box::into_raw(f);
    Box::into_raw(m);
    Box::into_raw(aux);
    handle_error(result)
}

unsafe extern "C" fn x_unlock<T: SqliteIoMethods>(
    arg1: *mut sqlite3_file,
    arg2: c_int,
) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(arg1.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let mut aux = Box::<T>::from_raw(m.aux.cast());
    let result = aux.unlock(arg2);
    Box::into_raw(f);
    Box::into_raw(m);
    Box::into_raw(aux);
    handle_error(result)
}

unsafe extern "C" fn x_check_reserved_lock<T: SqliteIoMethods>(
    arg1: *mut sqlite3_file,
    pResOut: *mut c_int,
) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(arg1.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let mut aux = Box::<T>::from_raw(m.aux.cast());
    let result = aux.check_reserved_lock(pResOut);
    Box::into_raw(f);
    Box::into_raw(m);
    Box::into_raw(aux);
    handle_error(result)
}

unsafe extern "C" fn x_file_control<T: SqliteIoMethods>(
    arg1: *mut sqlite3_file,
    op: c_int,
    pArg: *mut c_void,
) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(arg1.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let mut aux = Box::<T>::from_raw(m.aux.cast());
    let result = aux.file_control(arg1, op, pArg);
    Box::into_raw(f);
    Box::into_raw(m);
    Box::into_raw(aux);
    handle_error(result)
}


unsafe extern "C" fn x_sector_size<T: SqliteIoMethods>(arg1: *mut sqlite3_file) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(arg1.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let mut aux = Box::<T>::from_raw(m.aux.cast());
    let result = aux.sector_size();
    Box::into_raw(f);
    Box::into_raw(m);
    Box::into_raw(aux);
    result
}

unsafe extern "C" fn x_device_characteristics<T: SqliteIoMethods>(arg1: *mut sqlite3_file) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(arg1.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let mut aux = Box::<T>::from_raw(m.aux.cast());
    let result = aux.device_characteristics();
    Box::into_raw(f);
    Box::into_raw(m);
    Box::into_raw(aux);
    result
}

unsafe extern "C" fn x_shm_map<T: SqliteIoMethods>(
    arg1: *mut sqlite3_file,
    iPg: c_int,
    pgsz: c_int,
    arg2: c_int,
    arg3: *mut *mut c_void,
) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(arg1.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let mut aux = Box::<T>::from_raw(m.aux.cast());
    let result = aux.shm_map(iPg, pgsz, arg2, arg3);
    Box::into_raw(f);
    Box::into_raw(m);
    Box::into_raw(aux);
    handle_error(result)
}

unsafe extern "C" fn x_shm_lock<T: SqliteIoMethods>(
    arg1: *mut sqlite3_file,
    offset: c_int,
    n: c_int,
    flags: c_int,
) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(arg1.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let mut aux = Box::<T>::from_raw(m.aux.cast());
    let result = aux.shm_lock(offset, n, flags);
    Box::into_raw(f);
    Box::into_raw(m);
    Box::into_raw(aux);
    handle_error(result)
}


unsafe extern "C" fn x_shm_barrier<T: SqliteIoMethods>(arg1: *mut sqlite3_file) {
    let mut f = Box::<FileWithAux<T>>::from_raw(arg1.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let mut aux = Box::<T>::from_raw(m.aux.cast());

    aux.shm_barrier();

    Box::into_raw(f);
    Box::into_raw(m);
    Box::into_raw(aux);
}

unsafe extern "C" fn x_shm_unmap<T: SqliteIoMethods>(
    arg1: *mut sqlite3_file,
    deleteFlag: c_int,
) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(arg1.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let mut aux = Box::<T>::from_raw(m.aux.cast());

    let result = aux.shm_unmap(deleteFlag);
    Box::into_raw(f);
    Box::into_raw(m);
    Box::into_raw(aux);
    handle_error(result)
}

unsafe extern "C" fn x_fetch<T: SqliteIoMethods>(
    arg1: *mut sqlite3_file,
    iOfst: sqlite3_int64,
    iAmt: c_int,
    pp: *mut *mut c_void,
) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(arg1.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let mut aux = Box::<T>::from_raw(m.aux.cast());

    let result = aux.fetch(iOfst, iAmt, pp);
    Box::into_raw(f);
    Box::into_raw(m);
    Box::into_raw(aux);
    handle_error(result)
}

unsafe extern "C" fn x_unfetch<T: SqliteIoMethods>(
    arg1: *mut sqlite3_file,
    iOfst: sqlite3_int64,
    p: *mut c_void,
) -> c_int {
    let mut f = Box::<FileWithAux<T>>::from_raw(arg1.cast::<FileWithAux<T>>());
    let mut m = Box::<MethodsWithAux<T>>::from_raw(f.0.cast_mut());
    let mut aux = Box::<T>::from_raw(m.aux.cast());

    let result = aux.unfetch(iOfst, p);
    Box::into_raw(f);
    Box::into_raw(m);
    Box::into_raw(aux);
    handle_error(result)
}

// C struct polymorphism, given the alignment and field sequence are the same
#[repr(C)]
pub(crate) struct FileWithAux<T: SqliteIoMethods> (*const MethodsWithAux<T>);


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
        aux: Box::into_raw(Box::new(aux))
    }
}

pub fn create_file_pointer<T: SqliteIoMethods>(actual_methods: T) -> *mut sqlite3_file {
    unsafe {
        let methods = create_io_methods::<T>(actual_methods);
        let methods_ptr = Box::into_raw(Box::new(methods));

        let p = FileWithAux::<T>(methods_ptr);

        let p = Box::into_raw(Box::new(p));
        
        p.cast()
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct MethodsWithAux<T: SqliteIoMethods> {
    pub iVersion: ::std::os::raw::c_int,
    pub xClose: ::std::option::Option<
        unsafe extern "C" fn(arg1: *mut sqlite3_file) -> ::std::os::raw::c_int,
    >,
    pub xRead: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut sqlite3_file,
            arg2: *mut ::std::os::raw::c_void,
            iAmt: ::std::os::raw::c_int,
            iOfst: sqlite3_int64,
        ) -> ::std::os::raw::c_int,
    >,
    pub xWrite: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut sqlite3_file,
            arg2: *const ::std::os::raw::c_void,
            iAmt: ::std::os::raw::c_int,
            iOfst: sqlite3_int64,
        ) -> ::std::os::raw::c_int,
    >,
    pub xTruncate: ::std::option::Option<
        unsafe extern "C" fn(arg1: *mut sqlite3_file, size: sqlite3_int64) -> ::std::os::raw::c_int,
    >,
    pub xSync: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut sqlite3_file,
            flags: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub xFileSize: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut sqlite3_file,
            pSize: *mut sqlite3_int64,
        ) -> ::std::os::raw::c_int,
    >,
    pub xLock: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut sqlite3_file,
            arg2: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub xUnlock: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut sqlite3_file,
            arg2: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub xCheckReservedLock: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut sqlite3_file,
            pResOut: *mut ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub xFileControl: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut sqlite3_file,
            op: ::std::os::raw::c_int,
            pArg: *mut ::std::os::raw::c_void,
        ) -> ::std::os::raw::c_int,
    >,
    pub xSectorSize: ::std::option::Option<
        unsafe extern "C" fn(arg1: *mut sqlite3_file) -> ::std::os::raw::c_int,
    >,
    pub xDeviceCharacteristics: ::std::option::Option<
        unsafe extern "C" fn(arg1: *mut sqlite3_file) -> ::std::os::raw::c_int,
    >,
    pub xShmMap: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut sqlite3_file,
            iPg: ::std::os::raw::c_int,
            pgsz: ::std::os::raw::c_int,
            arg2: ::std::os::raw::c_int,
            arg3: *mut *mut ::std::os::raw::c_void,
        ) -> ::std::os::raw::c_int,
    >,
    pub xShmLock: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut sqlite3_file,
            offset: ::std::os::raw::c_int,
            n: ::std::os::raw::c_int,
            flags: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub xShmBarrier: ::std::option::Option<unsafe extern "C" fn(arg1: *mut sqlite3_file)>,
    pub xShmUnmap: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut sqlite3_file,
            deleteFlag: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub xFetch: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut sqlite3_file,
            iOfst: sqlite3_int64,
            iAmt: ::std::os::raw::c_int,
            pp: *mut *mut ::std::os::raw::c_void,
        ) -> ::std::os::raw::c_int,
    >,
    pub xUnfetch: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut sqlite3_file,
            iOfst: sqlite3_int64,
            p: *mut ::std::os::raw::c_void,
        ) -> ::std::os::raw::c_int,
    >,
    pub aux: *mut T,
}