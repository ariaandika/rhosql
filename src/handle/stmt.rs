use libsqlite3_sys::{self as ffi};
use std::sync::Arc;

use crate::{handle::SqliteHandle, Error, Result};

/// represent the `sqlite3_stmt` object
///
/// this is low level api that mimic how sqlite api formed
///
/// note that if you using high level api,
/// calling one of this function may broke the sqlite state
#[derive(Debug)]
pub struct StatementHandle {
    stmt: *mut ffi::sqlite3_stmt,
    db: Arc<SqliteHandle>,
}

impl StatementHandle {
    pub fn new(stmt: *mut ffi::sqlite3_stmt, db: Arc<SqliteHandle>) -> Self {
        Self { stmt, db }
    }

    pub fn step(&mut self) -> i32 {
        unsafe { ffi::sqlite3_step(self.stmt) }
    }

    pub fn reset(&mut self) -> Result<()> {
        self.db.try_ok(unsafe { ffi::sqlite3_reset(self.stmt) }, Error::Step)
    }

    pub fn clear_bindings(&mut self) -> Result<()> {
        self.db.try_ok(unsafe { ffi::sqlite3_clear_bindings(self.stmt) }, Error::Step)
    }

    pub fn ptr(&self) -> *mut ffi::sqlite3_stmt {
        self.stmt
    }

    pub fn db(&self) -> &SqliteHandle {
        &self.db
    }

    pub fn data_count(&self) -> i32 {
        unsafe { ffi::sqlite3_data_count(self.stmt) }
    }

    pub fn column_type(&self, idx: i32) -> i32 {
        unsafe { ffi::sqlite3_column_type(self.stmt, idx) }
    }

    pub fn column_int(&self, idx: i32) -> i32 {
        unsafe { ffi::sqlite3_column_int(self.stmt, idx) }
    }

    pub fn column_double(&self, idx: i32) -> f64 {
        unsafe { ffi::sqlite3_column_double(self.stmt, idx) }
    }

    pub fn column_text(&self, idx: i32) -> Result<&str> {
        let text = unsafe {
            let text = ffi::sqlite3_column_text(self.stmt, idx);
            std::ffi::CStr::from_ptr(text.cast::<std::ffi::c_char>())
        };
        text.to_str().map_err(Error::NonUtf8Sqlite)
    }

    pub fn column_blob(&self, idx: i32) -> &[u8] {
        unsafe {
            let len = self.column_bytes(idx);
            let data = { ffi::sqlite3_column_blob(self.stmt, idx) };
            std::slice::from_raw_parts(data.cast::<u8>(), len as _)
        }
    }

    pub fn column_bytes(&self, idx: i32) -> i32 {
        unsafe { ffi::sqlite3_column_bytes(self.stmt, idx) }
    }

    pub fn finalize(self) { }
}

impl Drop for StatementHandle {
    fn drop(&mut self) {
        unsafe { ffi::sqlite3_finalize(self.stmt) };
    }
}

