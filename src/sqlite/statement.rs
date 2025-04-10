use libsqlite3_sys::{self as ffi};

use super::{DataType, SqliteHandle};
use crate::{
    Result,
    common::SqliteStr,
    error::{BindError, DecodeError, ResetError, StepError},
};

/// represent the `sqlite3_stmt` object
#[derive(Debug)]
pub struct StatementHandle {
    stmt: *mut ffi::sqlite3_stmt,
    db: SqliteHandle,
}

impl StatementHandle {
    pub(crate) fn new(stmt: *mut ffi::sqlite3_stmt, db: SqliteHandle) -> Self {
        Self { stmt, db }
    }

    pub fn db(&self) -> &SqliteHandle {
        &self.db
    }

    pub fn step(&mut self) -> Result<bool, StepError> {
        match unsafe { ffi::sqlite3_step(self.stmt) } {
            ffi::SQLITE_ROW => Ok(true),
            ffi::SQLITE_DONE => Ok(false),
            result => Err(self.db.db_error(result)),
        }
    }

    pub fn reset(&mut self) -> Result<(), ResetError> {
        self.db.try_result(unsafe { ffi::sqlite3_reset(self.stmt) })
    }

    pub fn clear_bindings(&mut self) -> Result<(), ResetError> {
        self.db.try_result(unsafe { ffi::sqlite3_clear_bindings(self.stmt) })
    }

    pub fn finalize(self) { }
}

/// parameter encoding
impl StatementHandle {
    pub fn bind_int(&mut self, idx: i32, value: i32) -> Result<(), BindError> {
        self.db.try_result(unsafe { ffi::sqlite3_bind_int(self.stmt, idx, value) })
    }

    pub fn bind_double(&mut self, idx: i32, value: f64) -> Result<(), BindError> {
        self.db.try_result(unsafe { ffi::sqlite3_bind_double(self.stmt, idx, value) })
    }

    pub fn bind_null(&mut self, idx: i32) -> Result<(), BindError> {
        self.db.try_result(unsafe { ffi::sqlite3_bind_null(self.stmt, idx) })
    }

    // todo: maybe choose other than SQLITE_TRANSIENT

    pub fn bind_text<S: SqliteStr>(&mut self, idx: i32, text: S) -> Result<(), BindError> {
        let (ptr, len, dtor) = text.as_sqlite_str()?;
        self.db.try_result(unsafe { ffi::sqlite3_bind_text(self.stmt, idx, ptr, len, dtor) })
    }

    pub fn bind_blob(&mut self, idx: i32, data: &[u8]) -> Result<(), BindError> {
        self.db.try_result(unsafe {
            ffi::sqlite3_bind_blob(
                self.stmt,
                idx,
                data.as_ptr().cast(),
                i32::try_from(data.len()).unwrap_or(i32::MAX),
                ffi::SQLITE_TRANSIENT(),
            )
        })
    }
}

/// column decoding
impl StatementHandle {
    pub fn data_count(&self) -> i32 {
        unsafe { ffi::sqlite3_data_count(self.stmt) }
    }

    pub fn column_type(&self, idx: i32) -> DataType {
        let code = unsafe { ffi::sqlite3_column_type(self.stmt, idx) };
        DataType::from_code(code).expect("sqlite return non datatype from `sqlite3_column_type`")
    }

    pub fn column_int(&self, idx: i32) -> i32 {
        unsafe { ffi::sqlite3_column_int(self.stmt, idx) }
    }

    pub fn column_double(&self, idx: i32) -> f64 {
        unsafe { ffi::sqlite3_column_double(self.stmt, idx) }
    }

    pub fn column_text(&self, idx: i32) -> Result<&str, DecodeError> {
        let text = unsafe {
            let text = ffi::sqlite3_column_text(self.stmt, idx);
            std::ffi::CStr::from_ptr(text.cast())
        };
        text.to_str().map_err(DecodeError::Utf8)
    }

    pub fn column_blob(&self, idx: i32) -> &[u8] {
        unsafe {
            let len = self.column_bytes(idx) as usize;
            let data = ffi::sqlite3_column_blob(self.stmt, idx).cast();
            std::slice::from_raw_parts(data, len)
        }
    }

    pub fn column_bytes(&self, idx: i32) -> i32 {
        unsafe { ffi::sqlite3_column_bytes(self.stmt, idx) }
    }
}

impl Drop for StatementHandle {
    fn drop(&mut self) {
        unsafe { ffi::sqlite3_finalize(self.stmt) };
    }
}

