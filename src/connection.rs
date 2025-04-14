use lru::LruCache;
use std::{
    hash::{DefaultHasher, Hasher},
    num::NonZeroUsize,
};

use crate::{
    Result,
    common::SqliteStr,
    query::{Execute, StatementRef},
    sqlite::{Database, DatabaseExt, OpenFlag, SqliteHandle, StatementHandle, error::OpenError},
};

/// Database connection.
#[derive(Debug)]
pub struct Connection {
    stmts: LruCache<u64, StatementHandle>,
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
}

impl<'s> Execute<'s> for &'s mut Connection {
    fn prepare<S: SqliteStr>(self, sql: S) -> Result<StatementRef<'s>> {
        let mut hash = DefaultHasher::new();
        sql.hash(&mut hash);
        let key = hash.finish();

        let stmt = self.stmts.try_get_or_insert(key, || {
            StatementHandle::prepare_v2(self.handle.as_ptr(), sql)
        })?;

        Ok(StatementRef::Borrow(stmt))
    }
}

impl Database for Connection {
    fn as_ptr(&self) -> *mut libsqlite3_sys::sqlite3 {
        self.handle.as_ptr()
    }
}

