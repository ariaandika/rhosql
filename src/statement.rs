use std::ptr;

use crate::{
    common::SqliteStr,
    handle::{SqliteHandle, StatementHandle},
    row_buffer::ValueRef,
    row_stream::RowStream,
    Result,
};

/// sql prepared statement
#[derive(Debug)]
pub struct Statement {
    stmt: StatementHandle,
}

impl Statement {
    pub(crate) fn prepare<S: SqliteStr>(db: SqliteHandle, sql: S) -> Result<Self> {
        let mut stmt = ptr::null_mut();

        db.prepare_v2(sql, &mut stmt, &mut ptr::null())?;

        debug_assert!(!stmt.is_null(), "we check result above");

        Ok(Self { stmt: StatementHandle::new(stmt, db) })
    }

    /// bind a value and start iterating row
    pub fn bind<'me>(&'me mut self, args: &[ValueRef]) -> Result<RowStream<'me>> {
        RowStream::setup(self, args)
    }

    // we keep it private instead of Deref so that methods from
    // handles does not leak

    pub(crate) fn db(&self) -> &SqliteHandle {
        self.stmt.db()
    }

    pub(crate) fn stmt(&self) -> &StatementHandle {
        &self.stmt
    }

    pub(crate) fn stmt_mut(&mut self) -> &mut StatementHandle {
        &mut self.stmt
    }

    pub(crate) fn clear(&mut self) -> Result<()> {
        self.stmt.reset()?;
        self.stmt.clear_bindings()
    }
}

