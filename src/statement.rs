use crate::{
    common::SqliteStr,
    sqlite::{SqliteHandle, StatementHandle},
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
        Ok(Self { stmt: db.prepare_v2(sql)?, })
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

