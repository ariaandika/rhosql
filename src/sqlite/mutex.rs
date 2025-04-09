use libsqlite3_sys::{self as ffi};

use super::SqliteHandle;

/// represent a held mutex lock from `sqlite3_mutex_enter()`
///
/// when this struct is dropped, lock will be unlock via `sqlite3_mutex_leave()`
///
/// <https://sqlite.org/c3ref/mutex_alloc.html>
pub struct SqliteMutexGuard<'a> {
    _db: &'a SqliteHandle,
    lock: *mut ffi::sqlite3_mutex,
}

impl<'a> SqliteMutexGuard<'a> {
    pub(crate) fn new(db: &'a SqliteHandle, lock: *mut ffi::sqlite3_mutex) -> Self {
        Self { _db: db, lock }
    }
}

impl Drop for SqliteMutexGuard<'_> {
    fn drop(&mut self) {
        unsafe { ffi::sqlite3_mutex_leave(self.lock) }
    }
}


