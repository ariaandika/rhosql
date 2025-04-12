use libsqlite3_sys::{self as ffi};
use std::marker::PhantomData;

/// represent a held mutex lock from `sqlite3_mutex_enter()`
///
/// when this struct is dropped, lock will be unlock via `sqlite3_mutex_leave()`
///
/// <https://sqlite.org/c3ref/mutex_alloc.html>
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

