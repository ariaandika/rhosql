use lru::LruCache;
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    num::NonZeroUsize,
};

use crate::{
    Result,
    common::SqliteStr,
    row::ValueRef,
    sqlite::{
        Database, DatabaseExt, OpenFlag, SqliteHandle,
        error::{OpenError, PrepareError},
    },
    statement::Statement,
};

/// Database connection.
#[derive(Debug)]
pub struct Connection {
    stmts: LruCache<u64, Statement>,
    handle: SqliteHandle,
}

/// SAFETY: Checked that sqlite compiled with `SERIALIZE_MODE`
/// thus synchronization is handled by sqlite
unsafe impl Send for Connection {}

/// SAFETY: Checked that sqlite compiled with `SERIALIZE_MODE`
/// thus synchronization is handled by sqlite
unsafe impl Sync for Connection {}

impl Connection {
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
        if !crate::sqlite::is_threadsafe() {
            Err(OpenError::NotSerializeMode)?;
        }

        let path = path.to_nul_string().map_err(OpenError::from)?;
        let mut handle = SqliteHandle::open_v2(&path, flags)?;

        handle.extended_result_codes(true)?;
        handle.busy_timeout(std::time::Duration::from_secs(5))?;

        Ok(Self {
            handle,
            stmts: LruCache::new(NonZeroUsize::new(24).unwrap()),
        })
    }

    /// Create a prepared statement.
    ///
    /// Prepared statement is cached internally.
    pub fn prepare<S: SqliteStr + Hash>(&mut self, sql: S) -> Result<&mut Statement, PrepareError> {
        let mut hash = DefaultHasher::new();
        sql.hash(&mut hash);
        let key = hash.finish();

        self.stmts
            .try_get_or_insert_mut(key, || Statement::prepare(self.handle.as_ptr(), sql))
    }

    /// Execute a single statement.
    ///
    /// Prepared statement is cached internally.
    pub fn exec<'a, S: SqliteStr + Hash, R: IntoIterator<Item = ValueRef<'a>>>(
        &mut self,
        sql: S,
        args: R,
    ) -> Result<()> {
        let stmt = self.prepare(sql)?;
        let mut rows = stmt.bind(args)?;
        while rows.next()?.is_some() {}
        Ok(())
    }
}

impl Database for Connection {
    fn as_ptr(&self) -> *mut libsqlite3_sys::sqlite3 {
        self.handle.as_ptr()
    }
}

