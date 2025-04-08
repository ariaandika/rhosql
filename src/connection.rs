use libsqlite3_sys::{self as ffi};

use crate::{
    common::SqliteStr, handle::SqliteHandle, row_buffer::ValueRef, statement::Statement, Result,
};

/// database connection
#[derive(Clone)]
pub struct Connection {
    handle: SqliteHandle,
}

impl Connection {
    /// open a database connection
    pub fn open<P: SqliteStr>(path: P) -> Result<Self> {
        let flags = ffi::SQLITE_OPEN_READWRITE | ffi::SQLITE_OPEN_CREATE;
        let mut handle = SqliteHandle::open_v2(path, flags)?;

        handle.extended_result_codes(1)?;
        handle.busy_timeout(5000)?;

        Ok(Self { handle })
    }

    /// execute a single statement
    pub fn exec<S: SqliteStr>(&self, sql: S, args: &[ValueRef]) -> Result<()> {
        let mut stmt = self.prepare(sql)?;
        let mut rows = stmt.bind(args)?;
        while let Some(_) = rows.next()? { }
        Ok(())
    }

    /// create a prepared statement
    pub fn prepare<S: SqliteStr>(&self, sql: S) -> Result<Statement> {
        Statement::prepare(self.handle.clone(), sql)
    }
}

