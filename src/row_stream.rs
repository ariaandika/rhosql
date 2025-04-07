use libsqlite3_sys::{self as ffi};

use crate::{row_buffer::RowBuffer, statement::Statement, Error, Result};

#[derive(Debug)]
pub struct RowStream<'stmt> {
    stmt: &'stmt mut Statement,
    done: bool,
}

impl<'stmt> RowStream<'stmt> {
    pub fn new(stmt: &'stmt mut Statement) -> Self {
        Self { stmt, done: false }
    }

    pub fn next<'me>(&'me mut self) -> Result<Option<RowBuffer<'me,'stmt>>> {
        if self.done {
            return Ok(None);
        }

        let result = unsafe { ffi::sqlite3_step(**self.stmt) };
        match result {
            ffi::SQLITE_ROW => {}
            ffi::SQLITE_DONE => {
                self.done = true;
                return Ok(None)
            },
            ffi::SQLITE_BUSY => return Err(Error::SqliteBusy),
            ffi::SQLITE_MISUSE => {
                self.done = true;
                panic!("(bug) sqlite returns SQLITE_MISUSE")
            },
            ffi::SQLITE_ERROR => {
                self.done = true;
                return unsafe {
                    let err = ffi::sqlite3_errmsg(self.stmt.db());
                    let err = std::ffi::CStr::from_ptr(err).to_string_lossy().into_owned();
                    Err(Error::Step(err))
                }
            },
            code => {
                self.done = true;
                return Err(ffi::Error::new(code).into())
            },
        }

        Ok(Some(RowBuffer::new(self)))
    }

    pub(crate) fn stmt(&self) -> *mut ffi::sqlite3_stmt {
        **self.stmt
    }
}

impl Drop for RowStream<'_> {
    fn drop(&mut self) {
        unsafe { ffi::sqlite3_reset(**self.stmt) };
    }
}


