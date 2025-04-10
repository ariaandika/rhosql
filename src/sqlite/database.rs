use libsqlite3_sys::{self as ffi};
use std::{ffi::CStr, ptr, sync::Arc, time::Duration};

use super::{OpenFlag, SqliteMutexGuard, StatementHandle};
use crate::{
    Result, SqliteStr,
    error::{ConfigureError, DatabaseError, OpenError, PrepareError},
};

/// represent the `sqlite3` object
///
/// it is required that sqlite compiled with serialized mode enabled, thus this type is `Send` and `Sync`
///
/// this type is `Send` and `Sync`
#[derive(Debug, Clone)]
pub struct SqliteHandle {
    sqlite: Arc<*mut ffi::sqlite3>,
}

/// SAFETY: Checked that sqlite compiled with `SERIALIZE_MODE`
/// thus synchronization is handled by sqlite
unsafe impl Send for SqliteHandle{}

/// SAFETY: Checked that sqlite compiled with `SERIALIZE_MODE`
/// thus synchronization is handled by sqlite
unsafe impl Sync for SqliteHandle{}


impl SqliteHandle {
    /// open a sqlite database
    ///
    /// this is a wrapper for `sqlite3_open_v2()`
    ///
    /// <https://sqlite.org/c3ref/open.html>
    pub fn open_v2<P: SqliteStr>(path: P, flags: OpenFlag) -> Result<Self, OpenError> {
        // for unsafe `Send` and `Sync` impl
        // https://www.sqlite.org/threadsafe.html#compile_time_selection_of_threading_mode
        const SERIALIZE_MODE: i32 = 1;
        let thread_safe = unsafe { ffi::sqlite3_threadsafe() };
        if thread_safe != SERIALIZE_MODE {
            Err(OpenError::NotSerializeMode)?;
        }

        let mut sqlite = ptr::null_mut();

        // The filename argument is interpreted as UTF-8 for sqlite3_open() and sqlite3_open_v2()
        let path = path.to_nul_string()?;

        let result = unsafe { ffi::sqlite3_open_v2(path.as_ptr(), &mut sqlite, flags.0, ptr::null()) };

        if sqlite.is_null() {
            Err(OpenError::from(DatabaseError::new(
                ffi::code_to_str(result).into(),
                result,
            )))?;
        }

        let db = Self { sqlite: Arc::new(sqlite) };
        db.try_result::<OpenError>(result)?;

        // for unsafe `Send` and `Sync` impl
        // If the threading mode is Single-thread or Multi-thread then this routine returns a NULL pointer.
        // https://sqlite.org/c3ref/db_mutex.html
        let mutex = unsafe { ffi::sqlite3_db_mutex(*db.sqlite) };
        if mutex.is_null() {
            Err(OpenError::NotSerializeMode)?;
        }

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

    /// convert result code into [`Error`]
    pub fn db_error<E: From<DatabaseError>>(&self, result: i32) -> E {
        if ffi::SQLITE_MISUSE == result {
            panic!("(bug) sqlite returns SQLITE_MISUSE")
        }
        let msg = match self.errmsg().map(Into::into) {
            Some(msg) => msg,
            None => ffi::code_to_str(result).into(),
        };
        E::from(DatabaseError::new(msg, result))
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
            ffi::sqlite3_prepare_v2(*self.sqlite, ptr, len, &mut ppstmt, ptr::null_mut())
        })?;
        debug_assert!(!ppstmt.is_null(), "we check result above");
        Ok(StatementHandle::new(ppstmt, self.clone()))
    }

    /// returns the rowid of the most recent successful INSERT into a rowid table
    /// or virtual table on database connection
    ///
    /// If no successful INSERTs into rowid tables have ever occurred on the database connection D,
    /// then sqlite3_last_insert_rowid(D) returns zero.
    ///
    /// this is a wrapper for `sqlite3_last_insert_rowid()`
    pub fn last_insert_rowid(&self) -> i64 {
        unsafe { ffi::sqlite3_last_insert_rowid(*self.sqlite) }
    }

    /// attempt to enter a mutex, if another thread is already within the mutex,
    /// this call will block
    ///
    /// this is a wrapper for `sqlite3_last_insert_rowid()`
    pub fn mutex_enter(&self) -> SqliteMutexGuard<'_> {
        let lock = unsafe {
            let lock = ffi::sqlite3_db_mutex(*self.sqlite);
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
        self.try_result(unsafe { ffi::sqlite3_busy_timeout(*self.sqlite, timeout) })
    }

    /// enables or disables the extended result codes feature of SQLite.
    ///
    /// disabled by default
    ///
    /// this is a wrapper for `sqlite3_extended_result_codes()`
    pub fn extended_result_codes(&mut self, onoff: bool) -> Result<(), ConfigureError> {
        self.try_result(unsafe { ffi::sqlite3_extended_result_codes(*self.sqlite, onoff as _)})
    }
}

/// Error code and Message
impl SqliteHandle {
    /// return recent error's English-language text that describes the error,
    /// as either UTF-8 or UTF-16 respectively, or NULL if no error message is available
    ///
    /// this is a wrapper for `sqlite3_errmsg()`
    pub fn errmsg(&self) -> Option<&str> {
        let data = unsafe { ffi::sqlite3_errmsg(*self.sqlite) };
        if data.is_null() {
            return None;
        }
        unsafe { CStr::from_ptr(data) }.to_str().ok()
    }

    /// returns English-language text that describes the result code E, as UTF-8,
    /// or NULL if E is not an result code for which a text error message is available.
    ///
    /// in sqlite, this function does not require the database pointer, however, the allocation
    /// of the message is managed by sqlite, thus we dont know the lifetime, so instead
    /// use the database lifetime for it
    ///
    /// for safety, consider cloning the message immediately
    ///
    /// this is a wrapper for `sqlite3_errstr()`
    pub fn errstr(&self, code: i32) -> Option<&str> {
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
        unsafe { ffi::sqlite3_extended_errcode(*self.sqlite) }
    }
}

impl Drop for SqliteHandle {
    fn drop(&mut self) {
        if Arc::strong_count(&self.sqlite) != 1 {
            return
        }
        unsafe { ffi::sqlite3_close(*self.sqlite); };
    }
}

