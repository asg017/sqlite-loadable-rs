use sqlite3ext_sys::{sqlite3, sqlite3_stmt, sqlite3_value, SQLITE_ROW};
use std::{
    error::Error,
    ffi::{c_char, c_void, CString},
};

use crate::ext::{
    sqlite3ext_bind_text, sqlite3ext_column_bytes, sqlite3ext_column_int64, sqlite3ext_column_text,
    sqlite3ext_finalize, sqlite3ext_prepare_v2, sqlite3ext_step,
};

pub struct Statement {
    stmt: *mut sqlite3_stmt,
}

unsafe extern "C" fn destructor(raw: *mut c_void) {
    drop(CString::from_raw(raw.cast::<c_char>()));
}

impl Statement {
    pub fn prepare(db: *mut sqlite3, sql: &str) -> Result<Self, Box<dyn Error>> {
        let s = unsafe { CString::from_vec_unchecked(sql.into()) };

        let n: i32 = sql.len().try_into().unwrap();
        let mut stmt: *mut sqlite3_stmt = std::ptr::null_mut();
        unsafe {
            sqlite3ext_prepare_v2(db, s.as_ptr(), -1, &mut stmt, std::ptr::null_mut());
        }
        Ok(Statement { stmt })
    }
    fn bind_i32(&mut self, param_idx: i32, value: i32) -> Result<(), Box<dyn Error>> {
        todo!();
        Ok(())
    }
    pub fn bind_text(&mut self, param_idx: i32, value: &str) -> Result<(), Box<dyn Error>> {
        let bytes = value.as_bytes();
        unsafe {
            let s = CString::from_vec_unchecked(bytes.into());

            let n: i32 = bytes.len().try_into().unwrap();
            // CString and into_raw() is needed here, that way we can pass in a proper destructor so
            // SQLite can drop the allocated memory (avoids segfaults)
            sqlite3ext_bind_text(self.stmt, param_idx, s.into_raw(), n, Some(destructor));
        }
        Ok(())
    }
    fn bind_blob(&mut self, param_idx: i32, value: &[u8]) -> Result<(), Box<dyn Error>> {
        todo!();
        Ok(())
    }
    pub fn execute(&mut self) -> Rows {
        Rows { stmt: self.stmt }
    }
    fn execute_to_completion() -> Result<(), ()> {
        todo!();
    }
}

impl Drop for Statement {
    fn drop(&mut self) {
        unsafe { sqlite3ext_finalize(self.stmt) };
    }
}
pub struct Rows {
    stmt: *mut sqlite3_stmt,
}

impl Iterator for Rows {
    type Item = Result<Row, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        let code = unsafe { sqlite3ext_step(self.stmt) };
        match code {
            100 => Some(Ok(Row { stmt: self.stmt })),
            _ => None,
        }
    }
}

pub struct Row {
    stmt: *mut sqlite3_stmt,
}

impl Row {
    pub fn get<T: Value>(&self, idx: i32) -> Result<T, ()> {
        Value::value_result(self.stmt, idx)
    }
}

pub trait Value: Sized {
    fn value_result(stmt: *mut sqlite3_stmt, column_idx: i32) -> Result<Self, ()>;
}

impl Value for i64 {
    fn value_result(stmt: *mut sqlite3_stmt, column_idx: i32) -> Result<i64, ()> {
        Ok(unsafe { sqlite3ext_column_int64(stmt, column_idx) })
    }
}
impl Value for String {
    fn value_result(stmt: *mut sqlite3_stmt, column_idx: i32) -> Result<String, ()> {
        unsafe {
            let n = sqlite3ext_column_bytes(stmt, column_idx);
            let s = sqlite3ext_column_text(stmt, column_idx);
            let string = std::str::from_utf8(std::slice::from_raw_parts(s, n as usize));
            Ok(string.unwrap().to_string())
        }
    }
}
