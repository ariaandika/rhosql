use crate::{
    Result,
    common::SqliteStr,
    row::ValueRef,
    row_stream::RowStream,
    sqlite::{
        Database, StatementExt, StatementHandle,
        error::{BindError, PrepareError, ResetError},
    },
};

/// sql prepared statement
#[derive(Debug)]
pub struct Statement {
    handle: StatementHandle,
}

impl Statement {
    pub(crate) fn prepare<D: Database, S: SqliteStr>(db: D, sql: S) -> Result<Self, PrepareError> {
        Ok(Self { handle: StatementHandle::prepare_v2(db, sql)? })
    }

    /// bind a value and start iterating row
    pub fn bind<'me, 'input, R: IntoIterator<Item = ValueRef<'input>>>(
        &'me mut self,
        args: R,
    ) -> Result<RowStream<'me>, BindError> {
        RowStream::bind(&self.handle, args)
    }

    pub fn handle(&self) -> &StatementHandle {
        &self.handle
    }

    pub fn reset(&self) -> Result<(), ResetError> {
        self.handle.reset()?;
        self.handle.clear_bindings()
    }
}

