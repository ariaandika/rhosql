//! Types for query api.

use crate::{
    FromRow, Result, Row, SqliteStr, ValueRef,
    common::stack::Stack,
    row_stream::RowStream,
    sqlite::{
        Database, DatabaseExt, SqliteHandle, Statement, StatementExt, StatementHandle, StepResult,
    },
};

/// An executor which used in `query` api.
pub trait Execute<'s>: Database {
    fn prepare<S: SqliteStr>(self, sql: S) -> Result<StatementRef<'s>>;
}

impl<'s> Execute<'s> for &'s mut SqliteHandle {
    fn prepare<S: SqliteStr>(self, sql: S) -> Result<StatementRef<'s>> {
        Ok(StatementRef::Owned(StatementHandle::prepare_v2(self.as_ptr(), sql)?))
    }
}

/// Either borrowed or owned prepared statement.
pub enum StatementRef<'a> {
    Borrow(&'a StatementHandle),
    Owned(StatementHandle),
}

impl Statement for StatementRef<'_> {
    fn as_stmt_ptr(&self) -> *mut libsqlite3_sys::sqlite3_stmt {
        match self {
            StatementRef::Borrow(s) => s.as_stmt_ptr(),
            StatementRef::Owned(s) => s.as_stmt_ptr(),
        }
    }
}

/// Query api.
///
/// # Example
///
/// ```
/// # fn main() -> rhosql::Result<()> {
/// # use rhosql::Connection;
/// # let db = Connection::open_in_memory()?;
/// #[derive(rhosql::FromRow)]
/// struct Post {
///     id: i32,
///     name: String,
/// }
///
/// # rhosql::query("create table post(name)", &db).execute()?;
/// rhosql::query("insert into post(name) values(?1)", &db)
///     .bind("Control")
///     .execute()?;
///
/// let posts = rhosql::query("select rowid,* from post", &db).fetch_all::<(i32, String)>()?;
/// #   Ok(())
/// # }
/// ```
///
/// Note that parameter `bind` have hard limit of 16.
pub fn query<'a, 's, S: SqliteStr, E: Execute<'s>>(sql: S, db: E) -> Query<'a, S, E> {
    Query { db, sql, params: Stack::with_size() }
}

/// Query api created by [`query`]
#[derive(Debug)]
pub struct Query<'a, S, E> {
    db: E,
    sql: S,
    params: Stack<ValueRef<'a>,16>,
}

impl<'a, S, E> Query<'a, S, E> {
    /// Bind a parameter.
    ///
    /// Note that parameter `bind` have hard limit of 16.
    pub fn bind<V: Into<ValueRef<'a>>>(mut self, value: V) -> Self {
        self.params.push(value.into());
        self
    }
}

impl<'s, S, E> Query<'_, S, E>
where
    S: SqliteStr,
    E: Execute<'s>
{
    /// Collect result rows to a vector.
    pub fn fetch_all<R: FromRow>(self) -> Result<Vec<R>> {
        let stmt = self.db.prepare(self.sql)?;

        for (param,idx) in self.params.into_iter().zip(1i32..) {
            param.bind(idx, &stmt)?;
        }

        let mut rows = vec![];

        while stmt.step()?.is_row() {
            let row = Row::new(stmt.as_stmt_ptr());
            rows.push(R::from_row(row)?);
        }

        Ok(rows)
    }

    /// Optionally retrieve one row.
    pub fn fetch_optional<R: FromRow>(self) -> Result<Option<R>> {
        let stmt = self.db.prepare(self.sql)?;

        for (param,idx) in self.params.into_iter().zip(1i32..) {
            param.bind(idx, &stmt)?;
        }

        match stmt.step()? {
            StepResult::Row => {
                let row = Row::new(stmt.as_stmt_ptr());
                Ok(Some(R::from_row(row)?))
            }
            StepResult::Done => Ok(None),
        }
    }

    /// Retrieve row by [`Iterator`]
    pub fn fetch(self) -> Result<RowStream<'s>> {
        let stmt = self.db.prepare(self.sql)?;

        for (param,idx) in self.params.into_iter().zip(1i32..) {
            param.bind(idx, &stmt)?;
        }

        Ok(RowStream::new(stmt.as_stmt_ptr()))
    }

    /// Execute statement and return value of `last_insert_rowid`.
    pub fn execute(self) -> Result<i64> {
        let db = self.db.as_ptr();
        let stmt = self.db.prepare(self.sql)?;

        for (param,idx) in self.params.into_iter().zip(1i32..) {
            param.bind(idx, &stmt)?;
        }

        stmt.step()?;

        Ok(db.last_insert_rowid())
    }
}


