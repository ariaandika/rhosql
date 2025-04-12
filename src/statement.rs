use crate::sqlite::error::{BindError, PrepareError, ResetError};
use crate::sqlite::StatementExt;
use crate::{
    Result,
    common::SqliteStr,
    row::ValueRef,
    row_stream::RowStream,
    sqlite::{SqliteHandle, StatementHandle},
};

/// sql prepared statement
#[derive(Debug)]
pub struct Statement {
    handle: StatementHandle,
}

impl Statement {
    pub(crate) fn prepare<S: SqliteStr>(db: SqliteHandle, sql: S) -> Result<Self, PrepareError> {
        todo!()
        // Ok(Self { handle: db.prepare_v2(sql)?, })
    }

    /// bind a value and start iterating row
    pub fn bind<'me, 'input, R: IntoIterator<Item = ValueRef<'input>>>(
        &'me mut self,
        args: R,
    ) -> Result<RowStream<'me>, BindError> {
        RowStream::setup(self, args)
    }

    // we keep it private instead of Deref so that methods from
    // handles does not leak

    pub(crate) fn handle(&self) -> &StatementHandle {
        &self.handle
    }

    pub(crate) fn handle_mut(&mut self) -> &mut StatementHandle {
        &mut self.handle
    }

    pub(crate) fn reset(&mut self) -> Result<(), ResetError> {
        self.handle.reset()?;
        self.handle.clear_bindings()
    }
}

