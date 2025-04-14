use crate::{
    FromRow, Result, Row, SqliteStr,
    common::stack::Stack,
    row::ValueRef,
    sqlite::{Database, Statement, StatementExt, StatementHandle, StepResult},
};

pub trait Execute: Database {
    fn prepare<S: SqliteStr>(&self, sql: S) -> Result<StatementHandle>;
}

impl Execute for &crate::sqlite::SqliteHandle {
    fn prepare<S: SqliteStr>(&self, sql: S) -> Result<StatementHandle> {
        StatementHandle::prepare_v2(self.as_ptr(), sql).map_err(Into::into)
    }
}

impl Execute for &crate::Connection {
    fn prepare<S: SqliteStr>(&self, sql: S) -> Result<StatementHandle> {
        StatementHandle::prepare_v2(self.as_ptr(), sql).map_err(Into::into)
    }
}

/// Query api.
///
/// Note that currently parameter `bind` have hard limit of 16.
pub fn query<'a, S: SqliteStr, E: Execute>(sql: S, db: E) -> Query<'a, S, E> {
    Query { db, sql, params: Stack::with_size() }
}

#[derive(Debug)]
pub struct Query<'a, S, E> {
    db: E,
    sql: S,
    params: Stack<ValueRef<'a>,16>,
}

impl<'a, S, E> Query<'a, S, E> {
    /// Bind a parameter.
    ///
    /// Note that currently parameter `bind` have hard limit of 16.
    pub fn bind<V: Into<ValueRef<'a>>>(mut self, value: V) -> Self {
        self.params.push(value.into());
        self
    }
}

impl<S, E> Query<'_, S, E>
where
    S: SqliteStr,
    E: Execute
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

    /// Retrieve one row optionally.
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

    /// Execute statement and return value of `last_insert_rowid`
    pub fn execute(self) -> Result<i64> {
        use crate::sqlite::DatabaseExt;
        let stmt = self.db.prepare(self.sql)?;

        for (param,idx) in self.params.into_iter().zip(1i32..) {
            param.bind(idx, &stmt)?;
        }

        stmt.step()?;

        Ok(self.db.as_ptr().last_insert_rowid())
    }
}


