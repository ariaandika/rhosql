use libsqlite3_sys::{self as ffi};
use std::{ffi::CStr, marker::PhantomData};

use super::{Database, DatabaseError, OpenFlag, Statement, database, statement};

/// An RAII implementation of a [`sqlite3`][1] object. When this structure is
/// dropped (falls out of scope), `sqlite3` will be [`close`][2].
///
/// Database operation is provided by [`DatabaseExt`][3] extension trait.
///
/// [1]: <https://sqlite.org/c3ref/sqlite3.html>
/// [2]: <https://sqlite.org/c3ref/close.html>
/// [3]: super::DatabaseExt
#[derive(Debug)]
pub struct SqliteHandle {
    sqlite: *mut ffi::sqlite3,
}

impl SqliteHandle {
    /// Open new sqlite database.
    ///
    /// Filename should be a valid UTF-8.
    ///
    /// > The filename argument is interpreted as UTF-8 for sqlite3_open() and sqlite3_open_v2()
    /// >
    /// > <https://sqlite.org/c3ref/open.html>
    pub fn open_v2(path: &CStr, flags: OpenFlag) -> Result<Self, DatabaseError> {
        Ok(Self {
            sqlite: super::database::open_v2(path, flags)?,
        })
    }
}

impl Database for SqliteHandle {
    fn as_ptr(&self) -> *mut libsqlite3_sys::sqlite3 {
        self.sqlite
    }
}

/// Close the database
impl Drop for SqliteHandle {
    fn drop(&mut self) {
        if let Err(_err) = database::ffi_db!(sqlite3_close(self.sqlite) as _) {
            #[cfg(feature = "log")]
            log::error!("Failed to close db on drop: {_err}")
        }
    }
}

/// An RAII implementation of a [`sqlite3_stmt`][1] object. When this structure is
/// dropped (falls out of scope), `sqlite3_stmt` will be [`finalize`][2].
///
/// Statement operation is provided by [`StatementExt`][3] extension trait.
///
/// [1]: <https://sqlite.org/c3ref/sqlite3_stmt.html>
/// [2]: <https://sqlite.org/c3ref/finalize.html>
/// [3]: super::StatementExt
#[derive(Debug)]
pub struct StatementHandle {
    stmt: *mut ffi::sqlite3_stmt,
    db: *mut ffi::sqlite3,
}

impl StatementHandle {
    pub(crate) fn new(stmt: *mut ffi::sqlite3_stmt, db: *mut ffi::sqlite3) -> Self {
        Self { stmt, db }
    }

    /// Finalize the prepared statement
    pub fn finalize(self) { }
}

impl Database for StatementHandle {
    fn as_ptr(&self) -> *mut libsqlite3_sys::sqlite3 {
        self.db
    }
}

impl Statement for StatementHandle {
    fn as_stmt_ptr(&self) -> *mut libsqlite3_sys::sqlite3_stmt {
        self.stmt
    }
}

/// Finalize the prepared statement
impl Drop for StatementHandle {
    fn drop(&mut self) {
        if let Err(_err) = statement::ffi_stmt!(sqlite3_finalize(self.db, self.stmt) as _) {
            #[cfg(feature = "log")]
            log::error!("Failed to finalize prepare statement on drop: {_err}")
        }
    }
}

/// An RAII implementation of a [`sqlite3_mutex`][1] object. When this structure is
/// dropped (falls out of scope), `sqlite3_mutex` will be [`released`][1].
///
/// [1]: <https://sqlite.org/c3ref/mutex_alloc.html>
/// [3]: super::StatementExt
pub struct SqliteMutexGuard<'a> {
    lock: *mut ffi::sqlite3_mutex,
    _p: PhantomData<&'a ()>,
}

impl<'a> SqliteMutexGuard<'a> {
    pub(crate) fn new(lock: *mut ffi::sqlite3_mutex) -> Self {
        Self {
            lock,
            _p: PhantomData,
        }
    }
}

impl Drop for SqliteMutexGuard<'_> {
    fn drop(&mut self) {
        unsafe { ffi::sqlite3_mutex_leave(self.lock) }
    }
}

