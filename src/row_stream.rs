use std::marker::PhantomData;

use crate::{
    Result,
    row::{Row, ValueRef},
    sqlite::{
        Database, Statement, StatementExt, StatementHandle,
        error::{BindError, StepError},
    },
};

/// Bounded prepared statement and ready for iteration.
#[derive(Debug)]
pub struct RowStream<'stmt> {
    handle: (
        *mut libsqlite3_sys::sqlite3,
        *mut libsqlite3_sys::sqlite3_stmt,
    ),
    done: bool,
    _p: PhantomData<&'stmt mut ()>,
}

impl<'stmt> RowStream<'stmt> {
    pub(crate) fn bind<'input, R: IntoIterator<Item = ValueRef<'input>>>(
        stmt: &StatementHandle,
        args: R,
    ) -> Result<Self, BindError> {
        let me = Self {
            handle: (stmt.as_ptr(), stmt.as_stmt_ptr()),
            done: false,
            _p: PhantomData,
        };

        for (i, value) in (1i32..).zip(args) {
            value.bind(i, &me.handle)?;
        }

        Ok(me)
    }

    /// fetch the next row
    pub fn next<'me>(&'me mut self) -> Result<Option<Row<'me>>, StepError> {
        if self.done {
            return Ok(None);
        }

        if self.handle.step()?.is_done() {
            self.done = true;
            return Ok(None);
        }

        Ok(Some(Row::new(self.handle)))
    }
}

impl Drop for RowStream<'_> {
    fn drop(&mut self) {
        if let Err(err) = self.handle.reset() {
            eprintln!("{err}");
        }
    }
}

