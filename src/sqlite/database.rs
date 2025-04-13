use libsqlite3_sys::{self as ffi};
use std::{ffi::CStr, ptr, time::Duration};

use super::{DatabaseError, OpenFlag, SqliteMutexGuard, error::ConfigureError};

macro_rules! ffi_db {
    (@ $method:ident($db:expr $(, $($args:expr),*)?), $into:ty $(, $ret:expr)?) => {
        match {
            let db = $db;
            let result = unsafe { libsqlite3_sys::$method(db $(, $($args),*)?) };
            (db,result)
        } {
            (_, libsqlite3_sys::SQLITE_OK) => Ok({ $($ret)? }),
            (db, result) => Err(<$into>::from(super::DatabaseError::from_code(result, db))),
        }
    };
    ($method:ident($db:expr $(, $($args:expr),*)?) as _ $(, $ret:expr)?) => {
        super::database::ffi_db!(@ $method($db $(, $($args),*)?), super::DatabaseError $(, $ret)?)
    };
    ($method:ident($db:expr $(, $($args:expr),*)?) $(, $ret:expr)?) => {
        super::database::ffi_db!(@ $method($db $(, $($args),*)?), _ $(, $ret)?)
    };
}

pub(super) use ffi_db;

/// Open new sqlite database.
///
/// Filename should be a valid UTF-8.
///
/// > The filename argument is interpreted as UTF-8 for sqlite3_open() and sqlite3_open_v2()
/// >
/// > <https://sqlite.org/c3ref/open.html>
pub fn open_v2(path: &CStr, flags: OpenFlag) -> Result<*mut ffi::sqlite3, DatabaseError> {
    let mut sqlite = ptr::null_mut();

    let result = unsafe { ffi::sqlite3_open_v2(path.as_ptr(), &mut sqlite, flags.0, ptr::null()) };

    if sqlite.is_null() {
        return Err(DatabaseError {
            message: ffi::code_to_str(result).into(),
            code: result,
        });
    }

    match result {
        ffi::SQLITE_OK => Ok(sqlite),
        _ => Err(DatabaseError::from_code(result, sqlite)),
    }
}

/// A trait that represent [`sqlite3`][1] object.
///
/// Database operation provided by [`DatabaseExt`].
///
/// [1]: <https://sqlite.org/c3ref/sqlite3.html>
pub trait Database {
    fn as_ptr(&self) -> *mut ffi::sqlite3;
}

impl<D> Database for &mut D where D: Database {
    fn as_ptr(&self) -> *mut libsqlite3_sys::sqlite3 {
        D::as_ptr(self)
    }
}

impl<D> Database for &D where D: Database {
    fn as_ptr(&self) -> *mut libsqlite3_sys::sqlite3 {
        D::as_ptr(self)
    }
}

impl Database for *mut ffi::sqlite3 {
    fn as_ptr(&self) -> *mut ffi::sqlite3 {
        *self
    }
}

impl<T> DatabaseExt for T where T: Database { }

/// Database operation.
pub trait DatabaseExt: Database {
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
    fn mutex_enter(&self) -> SqliteMutexGuard {
        let lock = unsafe {
            let lock = ffi::sqlite3_db_mutex(self.as_ptr());
            assert!(!lock.is_null(),"connection guarantee to be in serialize mode");
            ffi::sqlite3_mutex_enter(lock);
            lock
        };
        SqliteMutexGuard::new(lock)
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
        ffi_db!(sqlite3_busy_timeout(self.as_ptr(), timeout))
    }

    /// enables or disables the extended result codes feature of SQLite.
    ///
    /// disabled by default
    ///
    /// this is a wrapper for `sqlite3_extended_result_codes()`
    fn extended_result_codes(&mut self, onoff: bool) -> Result<(), ConfigureError> {
        ffi_db!(sqlite3_extended_result_codes(self.as_ptr(), onoff as _))
    }

    // NOTE: Error code and Message

    /// return recent error's English-language text that describes the error,
    /// as either UTF-8 or UTF-16 respectively, or NULL if no error message is available
    ///
    /// this is a wrapper for `sqlite3_errmsg()`
    fn errmsg(&self) -> Option<&str> {
        let data = unsafe { ffi::sqlite3_errmsg(self.as_ptr()) };
        match data.is_null() {
            true => None,
            false => unsafe { CStr::from_ptr(data) }.to_str().ok(),
        }
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
        match data.is_null() {
            true => None,
            false => unsafe { CStr::from_ptr(data) }.to_str().ok(),
        }
    }

    /// returns extended result code for recent error
    ///
    /// this is a wrapper for `sqlite3_extended_errcode()`
    fn extended_errcode(&self) -> i32 {
        unsafe { ffi::sqlite3_extended_errcode(self.as_ptr()) }
    }

    /// Close the database.
    ///
    /// <https://sqlite.org/c3ref/close.html>
    fn close(&self) -> Result<(), DatabaseError> {
        ffi_db!(sqlite3_close(self.as_ptr()) as _)
    }
}

