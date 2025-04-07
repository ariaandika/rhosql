use libsqlite3_sys::{self as ffi};
use std::rc::{Rc, Weak};

use crate::{Error, Result};

/// a wrapper to holds sqlite pointer
///
/// handle implement `Clone` on which create a weak reference
///
/// thus any database operation should check is it still open
pub struct SqliteHandle {
    sqlite: *mut ffi::sqlite3,
    kind: HandleKind,
}

impl SqliteHandle {
    pub fn new(sqlite: *mut ffi::sqlite3) -> Self {
        Self { sqlite, kind: HandleKind::Strong(Rc::new(())) }
    }

    pub fn master_db(&self) -> *mut ffi::sqlite3 {
        self.try_db().expect("(bug) weak reference should not call SqliteHandle::master_db")
    }

    pub fn try_db(&self) -> Result<*mut ffi::sqlite3> {
        if let HandleKind::Weak(weak) = &self.kind {
            if weak.strong_count() == 0 {
                return Err(Error::AlreadyClosed)
            }
        }
        Ok(self.sqlite)
    }
}

impl Drop for SqliteHandle {
    fn drop(&mut self) {
        // Many applications destroy their database connections using calls to sqlite3_close() at shutdown.
        unsafe { ffi::sqlite3_close(self.sqlite) };
    }
}

impl Clone for SqliteHandle {
    fn clone(&self) -> Self {
        let weak = match &self.kind {
            HandleKind::Strong(rc) => Rc::downgrade(&rc),
            HandleKind::Weak(weak) => weak.clone(),
        };
        Self {
            sqlite: self.sqlite,
            kind: HandleKind::Weak(weak),
        }
    }
}

enum HandleKind {
    Strong(Rc<()>),
    Weak(Weak<()>),
}

