use crate::{Result, error::DecodeError, row_stream::RowStream, sqlite::DataType};

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

        let value = match self.stmt().column_type(idx) {
            DataType::Null => ValueRef::Null,
            DataType::Int => ValueRef::Int(self.stmt().column_int(idx)),
            DataType::Float => ValueRef::Float(self.stmt().column_double(idx)),
            DataType::Text => ValueRef::Text(self.stmt().column_text(idx)?),
            DataType::Blob => ValueRef::Blob(self.stmt().column_blob(idx)),
        };

        Ok(value)
    }

    fn stmt(&self) -> &crate::sqlite::StatementHandle {
        self.row_stream.stmt().stmt()
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
        *self
    }
}

impl Copy for ValueRef<'_> { }

impl From<i32> for ValueRef<'_> {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

impl From<f64> for ValueRef<'_> {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<bool> for ValueRef<'_> {
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

