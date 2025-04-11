
// Connection pooling
//
// `Pool` is the master struct.
//
// # Client Checkout
//
// Lazy checkout is, when cloned, `Pool` will not actually checkout a connection, only when a `query` is performed,
// a connection will checked out
//
// # Client Release
//
// ## Immediate Release
//
// When `Pool` checked out a connection, after query it will immediately release the connection
//
// This prevent footguns of doing heavy work after query, but when used for multiple query,
// connection checkout is performed multiple time.
//
// This is the default behavior
//
// ## Drop Guard Release
//
// When `Pool` checked out a connection, it will be hold until `Pool` is dropped.
//
// This can be efficient when doing multiple query with the same `Pool`,
// but may present footgun when doing heavy work after query.
//
// To use this behavior, after clone, a method must be called for `Pool` and connection will be
// reused until `Pool` dropped
//
// # Pool Checkout
//
// ## Mutex HashMap
//
// On checkout, `Pool` will acquire lock, and try to pick a connection, if none is available,
// `Pool` will wait for `CondVar`. On notified, `Pool` will attempt to pick a connection again.
//
// On release, `Pool` will have to call `notify` for `CondVar`
use std::{
    ffi::CString,
    sync::{Arc, Condvar, Mutex},
};

use crate::{
    Result, SqliteStr,
    common::stack::Stack,
    error::OpenError,
    sqlite::{OpenFlag, SqliteHandle},
};

const MAX_POOL_SIZE: usize = 8;

pub struct Pool {
    inner: Arc<PoolInner>,
    conn: Option<SqliteHandle>,
    /// should connection be release immediately after query
    ///
    /// default to `true`
    // is_single: bool,
    path: CString,
    flags: OpenFlag,
}

struct PoolInner {
    pool: Mutex<PoolStack>,
    cond: Condvar,
}

struct PoolStack {
    stack: Stack<SqliteHandle,MAX_POOL_SIZE>,
    /// the actual connection checked out even its not in the `pool`
    alive: usize,
}

impl Pool {
    #[allow(unused)]
    pub fn setup<S: SqliteStr>(path: S) -> Result<Pool, OpenError> {
        Self::setup_with(path, <_>::default())
    }

    #[allow(unused)]
    pub fn setup_with<S: SqliteStr>(path: S, flags: OpenFlag) -> Result<Pool, OpenError> {
        Ok(Pool {
            path: path.to_nul_string()?.into_owned(),
            flags,
            // is_single: true,
            conn: None,
            inner: Arc::new(PoolInner {
                pool: Mutex::new(PoolStack {
                    stack: Stack::with_size(),
                    alive: 0,
                }),
                cond: Condvar::new(),
            }),
        })
    }

    #[allow(unused)]
    fn checkout(&mut self) -> Result<&mut SqliteHandle, OpenError> {
        if self.conn.is_some() {
            return Ok(self.conn.as_mut().unwrap())
        }

        let mut pool = match self.inner.pool.lock() {
            Ok(ok) => ok,
            Err(err) => err.into_inner(),
        };

        let conn = loop {
            if let Some(ok) = pool.stack.pop() {
                break ok
            }
            if pool.alive == MAX_POOL_SIZE {
                pool = match self.inner.cond.wait(pool) {
                    Ok(ok) => ok,
                    Err(err) => err.into_inner(),
                };
            } else {
                drop(pool);

                let conn = SqliteHandle::open_v2(&self.path, self.flags)?;

                pool = match self.inner.pool.lock() {
                    Ok(ok) => ok,
                    Err(err) => err.into_inner(),
                };

                pool.alive += 1;

                break conn;
            }
        };

        Ok(self.conn.insert(conn))
    }

    #[allow(unused)]
    fn release(&mut self) {
        if self.conn.is_none() {
            return;
        }

        let mut pool = match self.inner.pool.lock() {
            Ok(ok) => ok,
            Err(err) => err.into_inner(),
        };

        pool.stack.push(self.conn.take().unwrap());
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        self.release();
    }
}



