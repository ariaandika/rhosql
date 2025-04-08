use libsqlite3_sys::{SQLITE_DONE, SQLITE_ROW};

use crate::{row_buffer::RowBuffer, statement::Statement, Error, Result};

/// bounded [`Statement`] and ready for iteration
#[derive(Debug)]
pub struct RowStream<'stmt> {
    stmt: &'stmt mut Statement,
    done: bool,
}

impl<'stmt> RowStream<'stmt> {
    pub(crate) fn new(stmt: &'stmt mut Statement) -> Self {
        Self { stmt, done: false }
    }

    /// fetch the next row
    pub fn next<'me>(&'me mut self) -> Result<Option<RowBuffer<'me,'stmt>>> {
        if self.done {
            return Ok(None);
        }

        match self.stmt.stmt_mut().step() {
            self::SQLITE_ROW => {}
            self::SQLITE_DONE => {
                self.done = true;
                return Ok(None)
            },
            result => {
                self.done = true;
                return Err(self.stmt.db().error(result, Error::Step))
            },
        }

        Ok(Some(RowBuffer::new(self)))
    }

    pub(crate) fn stmt(&self) -> &Statement {
        self.stmt
    }
}

impl Drop for RowStream<'_> {
    fn drop(&mut self) {
        if let Err(err) = self.stmt.clear() {
            eprintln!("{err}");
        }
    }
}

