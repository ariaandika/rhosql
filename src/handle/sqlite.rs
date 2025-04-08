use libsqlite3_sys::{self as ffi};
use std::{ffi::CStr, ptr};

use crate::{common::SqliteStr, Error, Result};

// NOTE: destructor implementation
// 1. share Arc and only close when everything is dropped, like prepared_statement
// 2. share Weak Arc and runtime check on Weak reference on any operation, then return error
// for now, option 1 is used as it seems simpler

/// represent the `sqlite3` object
///
/// this is low level struct which mimic how sqlite3 api are formed
///
/// for high level api use [`Connection`]
///
/// [`Connection`]: crate::Connection
#[derive(Debug, Clone)]
pub struct SqliteHandle {
    sqlite: *mut ffi::sqlite3,
}

macro_rules! doc {
    ($($tt:item)*) => { $(
        #[doc = "SAFETY: Checked that sqlite compiled with SERIALIZE_MODE"]
        #[doc = "thus synchronization is handled by sqlite"]
        $tt
    )* }
}
doc! {
    unsafe impl Send for SqliteHandle { }
    unsafe impl Sync for SqliteHandle { }
}

impl SqliteHandle {
    /// open a sqlite database
    ///
    /// this is a wrapper for `sqlite3_open_v2()`
    ///
    /// <https://sqlite.org/c3ref/open.html>
    pub fn open_v2<P: SqliteStr>(path: P, flags: i32) -> Result<Self> {
        // for unsafe `Send` and `Sync` impl
        // https://www.sqlite.org/threadsafe.html#compile_time_selection_of_threading_mode
        const SERIALIZE_MODE: i32 = 1;
        let thread_safe = unsafe { ffi::sqlite3_threadsafe() };
        if thread_safe != SERIALIZE_MODE {
            return Err(Error::NonSerialized)
        }

        let mut sqlite = ptr::null_mut();

        // The filename argument is interpreted as UTF-8 for sqlite3_open() and sqlite3_open_v2()
        let path = path.to_nul_string()?;

        let result = unsafe { ffi::sqlite3_open_v2(path.as_ptr(), &mut sqlite, flags, ptr::null()) };

        if sqlite.is_null() {
            return Err(ffi::Error::new(result).into());
        }

        let db = Self { sqlite };
        db.try_ok(result, Error::Open)?;
        Ok(db)
    }

    /// check if result SQLITE_OK, otherwise treat as an error
    ///
    /// make sure the possible non error code is only SQLITE_OK
    pub fn try_ok(&self, result: i32, map: fn(String) -> Error) -> Result<()> {
        match result {
            ffi::SQLITE_OK => Ok(()),
            _ => Err(self.error(result, map)),
        }
    }

    /// convert result code into [`Error`]
    pub fn error(&self, result: i32, map: fn(String) -> Error) -> Error {
        match result {
            ffi::SQLITE_BUSY => Error::SqliteBusy,
            ffi::SQLITE_MISUSE => {
                panic!("(bug) sqlite returns SQLITE_MISUSE")
            },
            ffi::SQLITE_ERROR => unsafe {
                let err = ffi::sqlite3_errmsg(self.sqlite);
                let err = CStr::from_ptr(err).to_string_lossy().into_owned();
                map(err)
            },
            _ => ffi::Error::new(result).into(),
        }
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
    pub fn prepare_v2<S: SqliteStr>(
        &self,
        sql: S,
        ppstmt: &mut *mut ffi::sqlite3_stmt,
        pztail: &mut *const i8,
    ) -> Result<()> {
        let (ptr, len, _) = sql.as_nulstr();
        self.try_ok(
            unsafe { ffi::sqlite3_prepare_v2(self.sqlite, ptr, len, ppstmt, pztail) },
            Error::Prepare,
        )
    }

    /// This routine sets a busy handler that sleeps for a specified amount of time when a table is locked.
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
    pub fn busy_timeout(&mut self, ms: i32) -> Result<()> {
        self.try_ok(unsafe { ffi::sqlite3_busy_timeout(self.sqlite, ms) }, Error::Message)
    }

    /// enables or disables the extended result codes feature of SQLite.
    /// disabled by default
    ///
    /// this is a wrapper for `sqlite3_extended_result_codes()`
    pub fn extended_result_codes(&mut self, onoff: i32) -> Result<()> {
        self.try_ok(unsafe { ffi::sqlite3_extended_result_codes(self.sqlite, onoff)}, Error::Message)
    }
}

impl Drop for SqliteHandle {
    fn drop(&mut self) {
        unsafe { ffi::sqlite3_close(self.sqlite) };
    }
}

