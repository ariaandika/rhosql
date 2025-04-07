use libsqlite3_sys::{self as ffi};
use std::{ptr, sync::Arc};

use crate::{common::FfiExt, handle::SqliteHandle, row_stream::RowStream, Error, Result};

#[derive(Debug)]
pub struct Statement {
    stmt: StatementHandle,
    db: Arc<SqliteHandle>,
}

impl Statement {
    pub(crate) fn prepare(db: Arc<SqliteHandle>, sql: &str) -> Result<Self> {
        let mut stmt = ptr::null_mut();

        let (zsql,nbyte,_) = sql.as_sqlite_cstr()?;

        let result = unsafe { ffi::sqlite3_prepare_v2(**db, zsql, nbyte, &mut stmt, &mut ptr::null()) };

        if result != ffi::SQLITE_OK {
            if result == ffi::SQLITE_ERROR {
                let msg: String = unsafe {
                    let p = ffi::sqlite3_errmsg(**db);
                    std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned()
                };
                return Err(Error::Prepare(msg))
            }
            return Err(ffi::Error::new(result).into());
        }

        debug_assert!(!stmt.is_null(), "we check result above");

        Ok(Self { stmt: StatementHandle { stmt }, db })
    }

    pub fn bind(&mut self) -> RowStream<'_> {
        RowStream::new(self)
    }

    pub(crate) fn db(&self) -> *mut ffi::sqlite3 {
        **self.db
    }
}

impl std::ops::Deref for Statement {
    type Target = *mut ffi::sqlite3_stmt;

    fn deref(&self) -> &Self::Target {
        &self.stmt.stmt
    }
}

#[derive(Debug)]
struct StatementHandle {
    stmt: *mut ffi::sqlite3_stmt,
}

impl Drop for StatementHandle {
    fn drop(&mut self) {
        unsafe { ffi::sqlite3_finalize(self.stmt) };
    }
}

