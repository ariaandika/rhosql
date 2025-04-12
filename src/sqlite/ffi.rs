use libsqlite3_sys::{self as ffi};
use std::{ffi::CStr, ptr, time::Duration};

use super::{OpenFlag, SqliteMutexGuard, StatementHandle};
use crate::{error::{ConfigureError, DatabaseError, OpenError, PrepareError}, SqliteStr};

/// A trait that represent [`sqlite3`][1] object.
///
/// [1]: <https://sqlite.org/c3ref/sqlite3.html>
pub trait Database {
    fn as_ptr(&self) -> *mut ffi::sqlite3;
}

/// Open new sqlite database.
///
/// # Errors
///
/// Returns `Err` if path is not UTF-8 or sqlite returns error code.
///
/// > The filename argument is interpreted as UTF-8 for sqlite3_open() and sqlite3_open_v2()
///
/// <https://sqlite.org/c3ref/open.html>
pub fn open_v2(path: &CStr, flags: OpenFlag) -> Result<*mut ffi::sqlite3, OpenError> {
    let mut sqlite = ptr::null_mut();

    path.to_str()?;

    let result = unsafe { ffi::sqlite3_open_v2(path.as_ptr(), &mut sqlite, flags.0, ptr::null()) };

    if sqlite.is_null() {
        Err(DatabaseError::new(ffi::code_to_str(result).into(), result))?;
    }

    match sqlite.try_result(result) {
        Ok(()) => Ok(sqlite),
        Err(err) => Err(err),
    }
}

impl Database for *mut ffi::sqlite3 {
    fn as_ptr(&self) -> *mut ffi::sqlite3 {
        *self
    }
}

pub trait DatabaseExt: Database {
    fn try_result<E: From<DatabaseError>>(&self, result: i32) -> Result<(), E> {
        super::error::try_result(self.as_ptr(), result)
    }

    fn db_error<E: From<DatabaseError>>(&self, result: i32) -> E {
        super::error::db_error(self.as_ptr(), result)
    }

    /// create a prepared statement
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
    fn prepare_v2<S: SqliteStr>(&self, sql: S) -> Result<StatementHandle, PrepareError> {
        StatementHandle::prepare(sql, self.as_ptr())
    }

    /// returns the rowid of the most recent successful INSERT into a rowid table
    /// or virtual table on database connection
    ///
    /// If no successful INSERTs into rowid tables have ever occurred on the database connection D,
    /// then sqlite3_last_insert_rowid(D) returns zero.
    ///
    /// this is a wrapper for `sqlite3_last_insert_rowid()`
    fn last_insert_rowid(&self) -> i64 {
        unsafe { ffi::sqlite3_last_insert_rowid(self.as_ptr()) }
    }

    /// attempt to enter a mutex, if another thread is already within the mutex,
    /// this call will block
    ///
    /// this is a wrapper for `sqlite3_last_insert_rowid()`
    fn mutex_enter(&self) -> SqliteMutexGuard<'_> {
        let lock = unsafe {
            let lock = ffi::sqlite3_db_mutex(self.as_ptr());
            assert!(!lock.is_null(),"connection guarantee to be in serialize mode");
            ffi::sqlite3_mutex_enter(lock);
            lock
        };
        todo!()
        // SqliteMutexGuard::new(self, lock)
    }

    //
    // NOTE: Configuration
    //

    /// This routine sets a busy handler that sleeps for a specified amount of time when a table is locked.
    ///
    /// The handler will sleep multiple times until at least "ms" milliseconds of sleeping have accumulated.
    /// After at least "ms" milliseconds of sleeping,
    /// the handler returns 0 which causes sqlite3_step() to return SQLITE_BUSY.
    ///
    /// Calling this routine with an argument less than or equal to zero turns off all busy handlers.
    ///
    /// If another busy handler was defined (using sqlite3_busy_handler()) prior to calling this routine,
    /// that other busy handler is cleared.
    ///
    /// this is a wrapper for `sqlite3_busy_timeout()`
    fn busy_timeout(&mut self, timeout: impl Into<Duration>) -> Result<(), ConfigureError> {
        let timeout = timeout.into().as_millis().try_into().unwrap_or(i32::MAX);
        self.try_result(unsafe { ffi::sqlite3_busy_timeout(self.as_ptr(), timeout) })
    }

    /// enables or disables the extended result codes feature of SQLite.
    ///
    /// disabled by default
    ///
    /// this is a wrapper for `sqlite3_extended_result_codes()`
    fn extended_result_codes(&mut self, onoff: bool) -> Result<(), ConfigureError> {
        self.try_result(unsafe { ffi::sqlite3_extended_result_codes(self.as_ptr(), onoff as _)})
    }


    // NOTE: Error code and Message

    /// return recent error's English-language text that describes the error,
    /// as either UTF-8 or UTF-16 respectively, or NULL if no error message is available
    ///
    /// this is a wrapper for `sqlite3_errmsg()`
    fn errmsg(&self) -> Option<&str> {
        let data = unsafe { ffi::sqlite3_errmsg(self.as_ptr()) };
        if data.is_null() {
            return None;
        }
        unsafe { CStr::from_ptr(data) }.to_str().ok()
    }

    /// returns English-language text that describes the result code E, as UTF-8,
    /// or NULL if E is not an result code for which a text error message is available.
    ///
    /// this is a wrapper for `sqlite3_errstr()`
    //
    // apparantly the string is static
    //
    // > Return a static string that describes the kind of error specified in the argument.
    //
    // <https://github.com/sqlite/sqlite/blob/0aa95099f5003dc99f599ab77ac0004950b281ef/src/main.c#L1636>
    fn errstr(code: i32) -> Option<&'static str> {
        let data = unsafe { ffi::sqlite3_errstr(code) };
        if data.is_null() {
            return None;
        }
        unsafe { CStr::from_ptr(data) }.to_str().ok()
    }

    /// returns extended result code for recent error
    ///
    /// this is a wrapper for `sqlite3_extended_errcode()`
    fn extended_errcode(&self) -> i32 {
        unsafe { ffi::sqlite3_extended_errcode(self.as_ptr()) }
    }
}

impl<T> DatabaseExt for T where T: Database { }

