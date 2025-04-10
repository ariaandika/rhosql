use libsqlite3_sys::{self as ffi};

use crate::{error::DecodeError, row_stream::RowStream, Result};

/// unencoded row buffer
#[derive(Debug)]
pub struct RowBuffer<'row,'stmt> {
    // we cannot borrow Statement here, cus mutable reference
    row_stream: &'row RowStream<'stmt>,
    col_count: i32,
}

impl<'row,'stmt> RowBuffer<'row,'stmt> {
    pub(crate) fn new(row_stream: &'row RowStream<'stmt>) -> Self {
        Self { col_count: row_stream.stmt().stmt().data_count(), row_stream }
    }

    /// try get `idx` column
    pub fn try_column(&self, idx: i32) -> Result<ValueRef, DecodeError> {
        if idx >= self.col_count {
            return Err(DecodeError::IndexOutOfBounds)
        }

        let ty = self.stmt().column_type(idx);

        let value = match ty {
            ffi::SQLITE_INTEGER => ValueRef::Int(self.stmt().column_int(idx)),
            ffi::SQLITE_FLOAT => ValueRef::Float(self.stmt().column_double(idx)),
            ffi::SQLITE_TEXT => ValueRef::Text(self.stmt().column_text(idx)?),
            ffi::SQLITE_BLOB => ValueRef::Blob(self.stmt().column_blob(idx)),
            ffi::SQLITE_NULL => ValueRef::Null,
            _ => unreachable!("sqlite return non datatype from `sqlite3_column_type`")
        };

        Ok(value)
    }

    fn stmt(&self) -> &crate::sqlite::StatementHandle {
        self.row_stream.stmt().stmt()
    }
}

pub trait FromColumn {
    /// either SQLITE_INTEGER, SQLITE_FLOAT, SQLITE_TEXT, SQLITE_BLOB, or SQLITE_NULL
    fn type_check(datatype_code: i32) -> bool;
}

impl FromColumn for &str {
    fn type_check(datatype_code: i32) -> bool {
        datatype_code == ffi::SQLITE3_TEXT
    }
}

#[derive(Debug)]
pub enum ValueRef<'a> {
    Null,
    Int(i32),
    Float(f64),
    Text(&'a str),
    Blob(&'a [u8]),
}

impl Clone for ValueRef<'_> {
    fn clone(&self) -> Self {
        match self {
            Self::Null => Self::Null,
            Self::Int(e) => Self::Int(*e),
            Self::Float(e) => Self::Float(*e),
            Self::Text(e) => Self::Text(e),
            Self::Blob(e) => Self::Blob(e),
        }
    }
}

impl Copy for ValueRef<'_> { }

impl<'a> From<i32> for ValueRef<'a> {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

impl<'a> From<f64> for ValueRef<'a> {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl<'a> From<bool> for ValueRef<'a> {
    fn from(value: bool) -> Self {
        Self::Int(value as _)
    }
}

impl<'a> From<&'a str> for ValueRef<'a> {
    fn from(value: &'a str) -> Self {
        Self::Text(value)
    }
}

impl<'a> From<&'a [u8]> for ValueRef<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self::Blob(value)
    }
}

impl<'a> From<&ValueRef<'a>> for ValueRef<'a> {
    fn from(value: &ValueRef<'a>) -> Self {
        *value
    }
}

