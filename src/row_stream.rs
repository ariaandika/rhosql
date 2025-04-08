use libsqlite3_sys::{SQLITE_DONE, SQLITE_ROW};

use crate::{row_buffer::{RowBuffer, ValueRef}, statement::Statement, Error, Result};

/// bounded [`Statement`] and ready for iteration
#[derive(Debug)]
pub struct RowStream<'stmt> {
    stmt: &'stmt mut Statement,
    done: bool,
}

impl<'stmt> RowStream<'stmt> {
    pub(crate) fn setup(stmt: &'stmt mut Statement, args: &[ValueRef]) -> Result<Self> {
        let me = Self { stmt, done: false };
        let iter = args.iter().enumerate().map(|e|(e.0 as i32 + 1,e.1));

        for (i,value) in iter {
            match *value {
                ValueRef::Null => me.stmt.stmt_mut().bind_null(i)?,
                ValueRef::Int(int) => me.stmt.stmt_mut().bind_int(i, int)?,
                ValueRef::Float(fl) => me.stmt.stmt_mut().bind_double(i, fl)?,
                ValueRef::Text(t) => me.stmt.stmt_mut().bind_text(i, t)?,
                ValueRef::Blob(b) => me.stmt.stmt_mut().bind_blob(i, b)?,
            }
        }
        Ok(me)
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

