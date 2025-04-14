use std::marker::PhantomData;

use crate::{
    FromRow, Result,
    row::Row,
    sqlite::{StatementExt, error::StepError},
};

/// Bounded prepared statement and ready for iteration.
#[derive(Debug)]
pub struct RowStream<'stmt> {
    handle: *mut libsqlite3_sys::sqlite3_stmt,
    done: bool,
    _p: PhantomData<&'stmt mut ()>,
}

impl RowStream<'_> {
    /// the statement parameter should already bound and ready to step.
    pub(crate) fn new(handle: *mut libsqlite3_sys::sqlite3_stmt) -> Self {
        Self {
            handle,
            done: false,
            _p: PhantomData,
        }
    }

    /// fetch the next row
    pub fn next(&mut self) -> Result<Option<Row<'_>>, StepError> {
        if self.done {
            return Ok(None);
        }

        if self.handle.step()?.is_done() {
            self.done = true;
            return Ok(None);
        }

        Ok(Some(Row::new(self.handle)))
    }

    pub fn next_row<R: FromRow>(&mut self) -> Result<Option<R>> {
        Ok(match self.next()? {
            Some(ok) => Some(R::from_row(ok)?),
            None => None,
        })
    }
}

impl Drop for RowStream<'_> {
    fn drop(&mut self) {
        if let Err(err) = self.handle.reset() {
            eprintln!("{err}");
        }
    }
}

