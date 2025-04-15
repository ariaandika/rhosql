use std::sync::{Arc, Mutex};

use crate::{
    Connection, Result, SqliteStr,
    query::{Execute, StatementRef},
    sqlite::{OpenFlag, Statement},
};

#[derive(Debug, Clone)]
pub struct SerializeConnection {
    shared: Arc<Mutex<Connection>>,
}

unsafe impl Send for SerializeConnection { }
unsafe impl Sync for SerializeConnection { }

impl SerializeConnection {
    /// Open a database connection with default flag.
    ///
    /// See [`OpenFlag`] for the default value.
    pub fn open<P: SqliteStr>(path: P) -> Result<Self> {
        Self::open_with(path, <_>::default())
    }

    /// Open in memory database.
    pub fn open_in_memory() -> Result<Self> {
        Self::open_with(c":memory:", <_>::default())
    }

    /// Open a database connection with given flag.
    pub fn open_with<P: SqliteStr>(path: P, flags: OpenFlag) -> Result<Self> {
        let conn = Connection::open_with(path, flags)?;
        Ok(Self {
            shared: Arc::new(Mutex::new(conn)),
        })
    }
}

impl<'s> Execute<'s> for &'s SerializeConnection {
    fn prepare<S: SqliteStr>(self, sql: S) -> Result<StatementRef<'s>> {
        let mut me = match self.shared.lock() {
            Ok(ok) => ok,
            Err(err) => err.into_inner(),
        };

        let stmt = <&mut Connection as Execute>::prepare(&mut me, sql)?;

        Ok(StatementRef::Handle(stmt.as_stmt_ptr()))
    }
}

impl<'s> Execute<'s> for &'s mut SerializeConnection {
    fn prepare<S: SqliteStr>(self, sql: S) -> Result<StatementRef<'s>> {
        <&SerializeConnection as Execute>::prepare(self, sql)
    }
}


