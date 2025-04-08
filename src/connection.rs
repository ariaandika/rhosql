use libsqlite3_sys::{self as ffi};
use std::{path::Path, sync::Arc};

use crate::{handle::SqliteHandle, row_buffer::ValueRef, statement::Statement, Result};

/// database connection
#[derive(Clone)]
pub struct Connection {
    handle: Arc<SqliteHandle>,
}

impl Connection {
    /// open a database connection
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {

        let flags = ffi::SQLITE_OPEN_READWRITE | ffi::SQLITE_OPEN_CREATE;
        let mut db = SqliteHandle::open_v2(path, flags)?;

        db.extended_result_codes(1)?;
        db.busy_timeout(5000)?;

        Ok(Self { handle: Arc::new(db) })
    }

    /// execute a single statement
    pub fn exec(&self, sql: &str, args: &[ValueRef]) -> Result<()> {
        let mut stmt = self.prepare(sql)?;
        let mut rows = stmt.bind(args)?;
        while let Some(_) = rows.next()? { }
        Ok(())
    }

    /// create a prepared statement
    pub fn prepare(&self, sql: &str) -> Result<Statement> {
        Statement::prepare(self.handle.clone(),sql)
    }
}

