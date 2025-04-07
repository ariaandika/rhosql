use std::ffi::CStr;

use libsqlite3_sys::{self as ffi};

use crate::{row_stream::RowStream, Error, Result};

#[derive(Debug)]
pub struct RowBuffer<'row,'stmt> {
    row_stream: &'row mut RowStream<'stmt>,
    col_count: i32,
}

impl<'row, 'stmt> RowBuffer<'row, 'stmt> {
    pub(crate) fn new(row_stream: &'row mut RowStream<'stmt>) -> Self {
        let col_count = unsafe { ffi::sqlite3_data_count(row_stream.stmt()) };
        Self { row_stream, col_count }
    }

    pub fn try_column(&mut self, idx: i32) -> Result<ValueRef> {
        if idx >= self.col_count {
            return Err(Error::IndexOutOfBounds)
        }

        let ty = unsafe { ffi::sqlite3_column_type(self.stmt(), idx) };

        let value = match ty {
            ffi::SQLITE_INTEGER => ValueRef::Int(unsafe { ffi::sqlite3_column_int(self.stmt(), idx) }),
            ffi::SQLITE_FLOAT => ValueRef::Float(unsafe { ffi::sqlite3_column_double(self.stmt(), idx) }),
            ffi::SQLITE_TEXT => {
                let value = unsafe {
                    let value = ffi::sqlite3_column_text(self.stmt(), idx).cast::<std::ffi::c_char>();
                    CStr::from_ptr(value)
                };
                ValueRef::Text(value.to_str().map_err(Error::NonUtf8Sqlite)?)
            },
            ffi::SQLITE_BLOB => {
                let len = unsafe { ffi::sqlite3_column_bytes(self.stmt(), idx) };
                let data = unsafe { ffi::sqlite3_column_blob(self.stmt(), idx) }.cast::<u8>();
                let blob = unsafe { std::slice::from_raw_parts(data, len as _) };
                ValueRef::Blob(blob)
            }
            ffi::SQLITE_NULL => ValueRef::Null,
            _ => unreachable!("sqlite return non datatype from `sqlite3_column_type`")
        };

        Ok(value)
    }

    pub(crate) fn stmt(&self) -> *mut ffi::sqlite3_stmt {
        self.row_stream.stmt()
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

