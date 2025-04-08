use libsqlite3_sys::{self as ffi};
use std::{
    ffi::{c_char, c_int},
    ptr,
    sync::Arc,
};

use crate::{
    handle::{SqliteHandle, StatementHandle},
    row_buffer::ValueRef,
    row_stream::RowStream,
    Error, Result,
};

/// sql prepared statement
#[derive(Debug)]
pub struct Statement {
    stmt: StatementHandle,
}

impl Statement {
    pub(crate) fn prepare(db: Arc<SqliteHandle>, sql: &str) -> Result<Self> {
        let mut stmt = ptr::null_mut();

        let (zsql,nbyte,_) = as_sqlite_cstr(sql)?;

        db.prepare_v2(zsql, nbyte, &mut stmt, ptr::null_mut())?;

        debug_assert!(!stmt.is_null(), "we check result above");

        Ok(Self { stmt: StatementHandle::new(stmt, db) })
    }

    /// bind a value and start iterating row
    pub fn bind<'me>(&'me mut self, args: &[ValueRef]) -> Result<RowStream<'me>> {
        RowStream::setup(self, args)
    }

    // we keep it private instead of Deref so that methods from
    // handles does not leak

    pub(crate) fn db(&self) -> &SqliteHandle {
        self.stmt.db()
    }

    pub(crate) fn stmt(&self) -> &StatementHandle {
        &self.stmt
    }

    pub(crate) fn stmt_mut(&mut self) -> &mut StatementHandle {
        &mut self.stmt
    }

    pub(crate) fn clear(&mut self) -> Result<()> {
        self.stmt.reset()?;
        self.stmt.clear_bindings()
    }
}

/// Returns `Ok((string ptr, len as c_int, SQLITE_STATIC | SQLITE_TRANSIENT))` normally.
///
/// Returns error if the string is too large for sqlite. (c_int::MAX = 2147483647)
///
/// The `sqlite3_destructor_type` item is always `SQLITE_TRANSIENT` unless
/// the string was empty (in which case it's `SQLITE_STATIC`, and the ptr is static).
fn as_sqlite_cstr(me: &str) -> Result<(*const c_char, c_int, ffi::sqlite3_destructor_type)> {
    let len = c_int::try_from(me.len()).map_err(|_|Error::StringTooLarge)?;
    let (ptr, dtor_info) = if len != 0 {
        (me.as_ptr().cast::<c_char>(), ffi::SQLITE_TRANSIENT())
    } else {
        // Return a pointer guaranteed to live forever
        ("".as_ptr().cast::<c_char>(), ffi::SQLITE_STATIC())
    };
    Ok((ptr, len, dtor_info))
}

