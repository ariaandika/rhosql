use libsqlite3_sys::{self as ffi};
use std::{ffi::CStr, ptr, time::Duration};

use super::{OpenFlag, SqliteMutexGuard, StatementHandle};
use crate::{
    Result, SqliteStr,
    error::{ConfigureError, DatabaseError, OpenError, PrepareError},
};

/// represent the `sqlite3` object
///
/// It automatically close the connection on drop.
#[derive(Debug)]
pub struct SqliteHandle {
    sqlite: *mut ffi::sqlite3,
}

impl SqliteHandle {
    /// open a sqlite database
    ///
    /// this is a wrapper for `sqlite3_open_v2()`
    ///
    /// <https://sqlite.org/c3ref/open.html>
    pub fn open_v2<P: SqliteStr>(path: P, flags: OpenFlag) -> Result<Self, OpenError> {
        let mut sqlite = ptr::null_mut();

        // The filename argument is interpreted as UTF-8 for sqlite3_open() and sqlite3_open_v2()
        let path = path.to_nul_string()?;

        let result = unsafe { ffi::sqlite3_open_v2(path.as_ptr(), &mut sqlite, flags.0, ptr::null()) };

        if sqlite.is_null() {
            Err(DatabaseError::new(
                ffi::code_to_str(result).into(),
                result,
            ))?;
        }

        let db = Self { sqlite };
        db.try_result::<OpenError>(result)?;

        Ok(db)
    }

    /// check if result `SQLITE_OK`, otherwise treat as an [`DatabaseError`]
    ///
    /// make sure the possible non error code is only `SQLITE_OK`
    pub fn try_result<E: From<DatabaseError>>(&self, result: i32) -> Result<(), E> {
        match result {
            ffi::SQLITE_OK => Ok(()),
            _ => Err(self.db_error(result)),
        }
    }

    /// convert result code into [`DatabaseError`]
    pub fn db_error<E: From<DatabaseError>>(&self, result: i32) -> E {
        if ffi::SQLITE_MISUSE == result {
            panic!("(bug) sqlite returns SQLITE_MISUSE")
        }
        let msg = match self.errmsg().map(Into::into) {
            Some(msg) => msg,
            None => ffi::code_to_str(result).into(),
        };
        DatabaseError::new(msg, result).into()
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
    pub fn prepare_v2<S: SqliteStr>(&self, sql: S) -> Result<StatementHandle, PrepareError> {
        let mut ppstmt = ptr::null_mut();
        let (ptr, len, _) = sql.as_nulstr();
        self.try_result::<PrepareError>(unsafe {
            ffi::sqlite3_prepare_v2(self.sqlite, ptr, len, &mut ppstmt, ptr::null_mut())
        })?;
        debug_assert!(!ppstmt.is_null(), "we check result above");
        todo!()
        // Ok(StatementHandle::new(ppstmt, self.clone()))
    }

    /// returns the rowid of the most recent successful INSERT into a rowid table
    /// or virtual table on database connection
    ///
    /// If no successful INSERTs into rowid tables have ever occurred on the database connection D,
    /// then sqlite3_last_insert_rowid(D) returns zero.
    ///
    /// this is a wrapper for `sqlite3_last_insert_rowid()`
    pub fn last_insert_rowid(&self) -> i64 {
        unsafe { ffi::sqlite3_last_insert_rowid(self.sqlite) }
    }

    /// attempt to enter a mutex, if another thread is already within the mutex,
    /// this call will block
    ///
    /// this is a wrapper for `sqlite3_last_insert_rowid()`
    pub fn mutex_enter(&self) -> SqliteMutexGuard<'_> {
        let lock = unsafe {
            let lock = ffi::sqlite3_db_mutex(self.sqlite);
            assert!(!lock.is_null(),"connection guarantee to be in serialize mode");
            ffi::sqlite3_mutex_enter(lock);
            lock
        };
        SqliteMutexGuard::new(self, lock)
    }
}

/// Configuration
impl SqliteHandle {
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
    pub fn busy_timeout(&mut self, timeout: impl Into<Duration>) -> Result<(), ConfigureError> {
        let timeout = timeout.into().as_millis().try_into().unwrap_or(i32::MAX);
        self.try_result(unsafe { ffi::sqlite3_busy_timeout(self.sqlite, timeout) })
    }

    /// enables or disables the extended result codes feature of SQLite.
    ///
    /// disabled by default
    ///
    /// this is a wrapper for `sqlite3_extended_result_codes()`
    pub fn extended_result_codes(&mut self, onoff: bool) -> Result<(), ConfigureError> {
        self.try_result(unsafe { ffi::sqlite3_extended_result_codes(self.sqlite, onoff as _)})
    }
}

/// Error code and Message
impl SqliteHandle {
    /// return recent error's English-language text that describes the error,
    /// as either UTF-8 or UTF-16 respectively, or NULL if no error message is available
    ///
    /// this is a wrapper for `sqlite3_errmsg()`
    pub fn errmsg(&self) -> Option<&str> {
        let data = unsafe { ffi::sqlite3_errmsg(self.sqlite) };
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
    pub fn errstr(code: i32) -> Option<&'static str> {
        let data = unsafe { ffi::sqlite3_errstr(code) };
        if data.is_null() {
            return None;
        }
        unsafe { CStr::from_ptr(data) }.to_str().ok()
    }

    /// returns extended result code for recent error
    ///
    /// this is a wrapper for `sqlite3_extended_errcode()`
    pub fn extended_errcode(&self) -> i32 {
        unsafe { ffi::sqlite3_extended_errcode(self.sqlite) }
    }
}

impl Drop for SqliteHandle {
    fn drop(&mut self) {
        unsafe { ffi::sqlite3_close(self.sqlite); };
    }
}

