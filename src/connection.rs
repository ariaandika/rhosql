use libsqlite3_sys::{self as ffi};
use std::{path::Path, sync::Arc};

use crate::{handle::SqliteHandle, statement::Statement, Error, Result};

/// database connection
#[derive(Clone)]
pub struct Connection {
    handle: Arc<SqliteHandle>,
}

// we checked that sqlite in Serialize mode
unsafe impl Send for Connection { }

impl Connection {
    /// open a database connection
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        // for unsafe `Send` impl
        // https://www.sqlite.org/threadsafe.html#compile_time_selection_of_threading_mode
        const SERIALIZE_MODE: i32 = 1;
        let thread_safe = unsafe { ffi::sqlite3_threadsafe() };
        if thread_safe != SERIALIZE_MODE {
            return Err(Error::NonSerialized)
        }

        let flags = ffi::SQLITE_OPEN_READWRITE | ffi::SQLITE_OPEN_CREATE;
        let mut db = SqliteHandle::open_v2(path, flags)?;

        db.extended_result_codes(1)?;
        db.busy_timeout(5000)?;

        Ok(Self { handle: Arc::new(db) })
    }

    /// execute a single statement
    pub fn exec(&self, sql: &str) -> Result<()> {
        let mut stmt = self.prepare(sql)?;
        let mut rows = stmt.bind();
        while let Some(_) = rows.next()? { }
        Ok(())
    }

    /// create a prepared statement
    pub fn prepare(&self, sql: &str) -> Result<Statement> {
        Statement::prepare(self.handle.clone(),sql)
    }
}

