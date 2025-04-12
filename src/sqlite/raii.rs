use libsqlite3_sys::{self as ffi};
use std::ffi::CStr;

use super::{Database, DatabaseError, OpenFlag, database::ffi_db};

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
        if let Err(_err) = ffi_db!(sqlite3_close(self.sqlite) as _) {
            #[cfg(feature = "log")]
            log::error!("Failed to close db on drop: {_err}")
        }
    }
}

