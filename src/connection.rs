use crate::{
    Result,
    common::SqliteStr,
    error::PrepareError,
    row_buffer::ValueRef,
    sqlite::{OpenFlag, SqliteHandle},
    statement::Statement,
};

/// database connection
#[derive(Clone)]
pub struct Connection {
    handle: SqliteHandle,
}

impl Connection {
    /// open a database connection with default flag
    ///
    /// see [`OpenFlag`] for the default value
    pub fn open<P: SqliteStr>(path: P) -> Result<Self> {
        Self::open_with(path, <_>::default())
    }

    /// open a database connection with given flag
    pub fn open_with<P: SqliteStr>(path: P, flags: OpenFlag) -> Result<Self> {
        let mut handle = SqliteHandle::open_v2(path, flags)?;

        handle.extended_result_codes(true)?;
        handle.busy_timeout(std::time::Duration::from_secs(5))?;

        Ok(Self { handle })
    }

    /// create a prepared statement
    pub fn prepare<S: SqliteStr>(&self, sql: S) -> Result<Statement, PrepareError> {
        Statement::prepare(self.handle.clone(), sql)
    }

    /// execute a single statement
    pub fn exec<S: SqliteStr>(&self, sql: S, args: &[ValueRef]) -> Result<()> {
        let mut stmt = self.prepare(sql)?;
        let mut rows = stmt.bind(args)?;
        while rows.next()?.is_some() { }
        Ok(())
    }
}

