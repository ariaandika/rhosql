use libsqlite3_sys::{self as ffi};

use crate::{Error, Result};

// NOTE: destructor implementation
// 1. share Arc and only close when everything is dropped, like prepared_statement
// 2. share Weak Arc and runtime check on Weak reference on any operation, then return error
// for now, option 1 is used as it seems simpler

#[derive(Debug, Clone)]
pub(crate) struct SqliteHandle {
    sqlite: *mut ffi::sqlite3,
}

impl std::ops::Deref for SqliteHandle {
    type Target = *mut ffi::sqlite3;

    fn deref(&self) -> &Self::Target {
        &self.sqlite
    }
}

impl SqliteHandle {
    pub fn setup(sqlite: *mut ffi::sqlite3) -> Result<Self> {
        // https://www.sqlite.org/threadsafe.html#compile_time_selection_of_threading_mode
        let thread_safe = unsafe { ffi::sqlite3_threadsafe() };
        if thread_safe != 1 {
            return Err(Error::NonSerialized)
        }

        Ok(Self { sqlite })
    }
}

impl Drop for SqliteHandle {
    fn drop(&mut self) {
        unsafe { ffi::sqlite3_close(self.sqlite) };
    }
}

