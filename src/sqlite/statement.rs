use libsqlite3_sys::{self as ffi};
use std::ptr;

use super::{
    DataType, DatabaseError, StepResult,
    database::ffi_db,
    error::{BindError, DecodeError, PrepareError, ResetError, StepError},
};
use crate::common::SqliteStr;

macro_rules! ffi_stmt {
    (@ $method:ident($db:expr, $stmt:expr $(, $($args:expr),*)?), $into:ty $(, $ret:expr)?) => {
        match {
            let db = $db;
            let result = unsafe { libsqlite3_sys::$method($stmt $(, $($args),*)?) };
            (db,result)
        } {
            (_, libsqlite3_sys::SQLITE_OK) => Ok({ $($ret)? }),
            (db, result) => Err(<$into>::from(super::DatabaseError::from_code(result, db))),
        }
    };
    ($method:ident($db:expr, $stmt:expr $(, $($args:expr),*)?) as _ $(, $ret:expr)?) => {
        super::statement::ffi_stmt!(@ $method($db, $stmt $(, $($args),*)?), super::DatabaseError $(, $ret)?)
    };
    ($method:ident($db:expr, $stmt:expr $(, $($args:expr),*)?) $(, $ret:expr)?) => {
        super::statement::ffi_stmt!(@ $method($db, $stmt $(, $($args),*)?), _ $(, $ret)?)
    };
}

pub(super) use ffi_stmt;

/// Create a prepared statement.
///
/// this is a wrapper for `sqlite3_prepare_v2()`
///
/// quoted from sqlite docs:
///
/// > If the caller knows that the supplied string is nul-terminated, then there is a small performance
/// > advantage to passing an nByte parameter that is the number of bytes in the input string
/// > *including* the nul-terminator.
///
/// providing sql via cstr may benefit a small performance advantage
///
/// <https://sqlite.org/c3ref/prepare.html>
pub fn prepare_v2<S: SqliteStr>(db: *mut ffi::sqlite3, sql: S) -> Result<*mut ffi::sqlite3_stmt, PrepareError> {
    let mut stmt = ptr::null_mut();
    let (ptr, len, _) = sql.as_nulstr();
    match ffi_db!(sqlite3_prepare_v2(db, ptr, len, &mut stmt, ptr::null_mut())) {
        Ok(()) => {
            #[cfg(feature = "log")]
            log::debug!("prepared {sql:?}");
            Ok(stmt)
        },
        Err(err) => Err(err),
    }
}

/// A trait that represent [`sqlite3_stmt`][1] object.
///
/// Statement operation provided by [`StatementExt`].
///
/// [1]: <https://sqlite.org/c3ref/sqlite3_stmt.html>
pub trait Statement {
    fn as_stmt_ptr(&self) -> *mut ffi::sqlite3_stmt;
}

impl<S> Statement for &S where S: Statement {
    fn as_stmt_ptr(&self) -> *mut ffi::sqlite3_stmt {
        S::as_stmt_ptr(self)
    }
}

impl Statement for *mut ffi::sqlite3_stmt {
    fn as_stmt_ptr(&self) -> *mut ffi::sqlite3_stmt {
        *self
    }
}

impl<T> StatementExt for T where T: Statement { }

/// Statement operation.
pub trait StatementExt: Statement {
    /// Returns the database connection handle to which a prepared statement belongs.
    fn as_db_ptr(&self) -> *mut ffi::sqlite3 {
        unsafe { ffi::sqlite3_db_handle(self.as_stmt_ptr()) }
    }

    fn step(&self) -> Result<StepResult, StepError> {
        match unsafe { ffi::sqlite3_step(self.as_stmt_ptr()) } {
            ffi::SQLITE_ROW => Ok(StepResult::Row),
            ffi::SQLITE_DONE => Ok(StepResult::Done),
            result => Err(DatabaseError::from_code(result, self.as_db_ptr()).into()),
        }
    }

    fn reset(&self) -> Result<(), ResetError> {
        ffi_stmt!(sqlite3_reset(self.as_db_ptr(), self.as_stmt_ptr()))
    }

    fn clear_bindings(&self) -> Result<(), ResetError> {
        ffi_stmt!(sqlite3_clear_bindings(self.as_db_ptr(), self.as_stmt_ptr()))
    }

    // NOTE: parameter encoding

    /// Bind integer to parameter at given index.
    ///
    /// Note that parameter index is one based.
    fn bind_int(&self, idx: i32, value: i32) -> Result<(), BindError> {
        ffi_stmt!(sqlite3_bind_int(self.as_db_ptr(), self.as_stmt_ptr(), idx, value))
    }

    /// Bind float to parameter at given index.
    ///
    /// Note that parameter index is one based.
    fn bind_double(&self, idx: i32, value: f64) -> Result<(), BindError> {
        ffi_stmt!(sqlite3_bind_double(self.as_db_ptr(), self.as_stmt_ptr(), idx, value))
    }

    /// Bind null to parameter at given index.
    ///
    /// Note that parameter index is one based.
    fn bind_null(&self, idx: i32) -> Result<(), BindError> {
        ffi_stmt!(sqlite3_bind_null(self.as_db_ptr(), self.as_stmt_ptr(), idx))
    }

    // todo: maybe choose other than SQLITE_TRANSIENT

    /// Bind text to parameter at given index.
    ///
    /// Note that parameter index is one based.
    fn bind_text<S: SqliteStr>(&self, idx: i32, text: S) -> Result<(), BindError> {
        let (ptr, len, dtor) = text.as_sqlite_str()?;
        ffi_stmt!(sqlite3_bind_text(self.as_db_ptr(), self.as_stmt_ptr(), idx, ptr, len, dtor))
    }

    /// Bind blob to parameter at given index.
    ///
    /// Note that parameter index is one based.
    fn bind_blob(&self, idx: i32, data: &[u8]) -> Result<(), BindError> {
        ffi_stmt!(sqlite3_bind_blob(
            self.as_db_ptr(),
            self.as_stmt_ptr(),
            idx,
            data.as_ptr().cast(),
            i32::try_from(data.len()).unwrap_or(i32::MAX),
            ffi::SQLITE_TRANSIENT()
        ))
    }

    // NOTE: column decoding



    /// Returns the number of columns, with or without results.
    fn column_count(&self) -> i32 {
        unsafe { ffi::sqlite3_column_count(self.as_stmt_ptr()) }
    }

    /// Returns the number of values (columns) of the currently executing statement.
    ///
    /// With no results it returns 0.
    fn data_count(&self) -> i32 {
        unsafe { ffi::sqlite3_data_count(self.as_stmt_ptr()) }
    }

    fn column_type(&self, idx: i32) -> DataType {
        let code = unsafe { ffi::sqlite3_column_type(self.as_stmt_ptr(), idx) };
        DataType::from_code(code).expect("sqlite return non datatype from `sqlite3_column_type`")
    }

    fn column_int(&self, idx: i32) -> i32 {
        unsafe { ffi::sqlite3_column_int(self.as_stmt_ptr(), idx) }
    }

    fn column_double(&self, idx: i32) -> f64 {
        unsafe { ffi::sqlite3_column_double(self.as_stmt_ptr(), idx) }
    }

    fn column_text(&self, idx: i32) -> Result<&str, DecodeError> {
        let text = unsafe {
            let text = ffi::sqlite3_column_text(self.as_stmt_ptr(), idx);
            std::ffi::CStr::from_ptr(text.cast())
        };
        text.to_str().map_err(DecodeError::Utf8)
    }

    fn column_blob(&self, idx: i32) -> &[u8] {
        unsafe {
            let len = self.column_bytes(idx) as usize;
            let data = ffi::sqlite3_column_blob(self.as_stmt_ptr(), idx).cast();
            std::slice::from_raw_parts(data, len)
        }
    }

    fn column_bytes(&self, idx: i32) -> i32 {
        unsafe { ffi::sqlite3_column_bytes(self.as_stmt_ptr(), idx) }
    }

    /// Delete the prepared staement.
    ///
    /// <https://sqlite.org/c3ref/finalize.html>
    fn finalize(&self) -> Result<(), DatabaseError> {
        ffi_stmt!(sqlite3_finalize(self.as_db_ptr(), self.as_stmt_ptr()) as _)
    }
}

