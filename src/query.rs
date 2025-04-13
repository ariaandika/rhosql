use crate::{
    FromRow, Result, SqliteStr,
    common::stack::Stack,
    row::ValueRef,
    sqlite::{Database, StatementExt, StatementHandle},
};

pub trait Execute {
    fn prepare<S: SqliteStr>(&self, sql: S) -> Result<StatementHandle>;
}

impl Execute for &crate::sqlite::SqliteHandle {
    fn prepare<S: SqliteStr>(&self, sql: S) -> Result<StatementHandle> {
        StatementHandle::prepare_v2(self.as_ptr(), sql).map_err(Into::into)
    }
}

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
    pub fn bind<V: Into<ValueRef<'a>>>(mut self, value: V) -> Self {
        self.params.push(value.into());
        self
    }
}

impl<'a, S, E> Query<'a, S, E>
where
    S: SqliteStr,
    E: Execute
{
    pub fn fetch_all<R: FromRow>(self) -> Result<Vec<R>> {
        let stmt = self.db.prepare(self.sql)?;
        for (param,i) in self.params.iter().zip(0i32..) {
            match *param {
                ValueRef::Null => stmt.bind_null(i)?,
                ValueRef::Int(val) => stmt.bind_int(i, val)?,
                ValueRef::Float(val) => stmt.bind_double(i, val)?,
                ValueRef::Text(val) => stmt.bind_text(i, val)?,
                ValueRef::Blob(val) => stmt.bind_blob(i, val)?,
            }
        }
        // let mut rows = vec![];
        let len = stmt.data_count();
        for _i in 0..len {
            // FromRow trait cannot be used with low level api
        }

        todo!()
    }
}


