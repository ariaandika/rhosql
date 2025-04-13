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
        Database, DatabaseExt, OpenFlag, SqliteHandle, StatementHandle,
        error::{OpenError, PrepareError},
    },
    statement::Statement,
};
/// Database connection.
#[derive(Debug)]
pub struct Connection {
    handle: SqliteHandle,
    stmts: LruCache<u64, Statement>,
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

    /// Open a database connection with given flag.
    pub fn open_with<P: SqliteStr>(path: P, flags: OpenFlag) -> Result<Self> {
        if !crate::sqlite::check_threadsafe() {
            Err(OpenError::NotSerializeMode)?;
        }

        let mut handle = SqliteHandle::open_v2(&path.to_nul_string().map_err(OpenError::from)?, flags)
            .map_err(OpenError::from)?;

        handle.extended_result_codes(true)?;
        handle.busy_timeout(std::time::Duration::from_secs(5))?;

        Ok(Self {
            handle,
            stmts: LruCache::new(NonZeroUsize::new(24).unwrap()),
        })
    }

    /// create a prepared statement
    pub fn prepare<S: SqliteStr + Hash>(&self, sql: S) -> Result<&mut Statement, PrepareError> {
        todo!()
        // let mut hash = DefaultHasher::new();
        // sql.hash(&mut hash);
        // let key = hash.finish();
        //
        // let handle = self.handle.clone();
        //
        // if let Some(ok) = self.stmts.get_mut(&key) {
        //     Ok(ok)
        // } else {
        //     let stmt = StatementHandle::prepare(handle, sql)?;
        //     Statement::prepare(handle, sql)
        // }
    }

    /// execute a single statement
    pub fn exec<'a, S: SqliteStr + Hash, R: IntoIterator<Item = ValueRef<'a>>>(
        &self,
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

