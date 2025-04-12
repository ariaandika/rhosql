use libsqlite3_sys::{self as ffi};

use super::{OpenFlag, ffi::Database};
use crate::{SqliteStr, error::OpenError};

/// Represent the `sqlite3` object.
///
/// It automatically close the connection on drop.
///
/// Use [`DatabaseExt`][1] for operation.
///
/// [1]: super::DatabaseExt
#[derive(Debug)]
pub struct SqliteHandle {
    sqlite: *mut ffi::sqlite3,
}

impl SqliteHandle {
    /// Open new sqlite database.
    ///
    /// # Errors
    ///
    /// Returns `Err` if path is not UTF-8 or sqlite returns error code.
    ///
    /// > The filename argument is interpreted as UTF-8 for sqlite3_open() and sqlite3_open_v2()
    ///
    /// <https://sqlite.org/c3ref/open.html>
    pub fn open_v2<P: SqliteStr>(path: P, flags: OpenFlag) -> Result<Self, OpenError> {
        Ok(Self {
            sqlite: super::ffi::open_v2(&path.to_nul_string()?, flags)?,
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
        unsafe {
            ffi::sqlite3_close(self.sqlite);
        };
    }
}
