use std::marker::PhantomData;

use crate::{
    sqlite::{
        error::{BindError, DecodeError}, DataType, Statement, StatementExt
    }, Result
};

/// Row buffer.
#[derive(Debug)]
pub struct Row<'row> {
    handle: *mut libsqlite3_sys::sqlite3_stmt,
    col_count: i32,
    _p: PhantomData<&'row mut ()>,
}

impl Row<'_> {
    pub fn new(handle: *mut libsqlite3_sys::sqlite3_stmt) -> Self {
        Self {
            col_count: handle.data_count(),
            handle,
            _p: PhantomData,
        }
    }

    /// try get single column from given index
    pub fn try_column(&self, idx: i32) -> Result<ValueRef, DecodeError> {
        if idx >= self.col_count {
            return Err(DecodeError::IndexOutOfBounds);
        }
        ValueRef::decode(idx, &self.handle)
    }

    /// try decode single column from given index
    pub fn try_decode<'a, D: Decode<'a>>(&'a self, idx: i32) -> Result<D> {
        self.try_column(idx)?.try_decode()
    }

    /// try decode current row
    pub fn try_row<D: FromRow>(self) -> Result<D> {
        D::from_row(self)
    }

    /// return the column count
    pub fn len(&self) -> usize {
        self.col_count as _
    }

    /// returns is no column returned
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A borrowed sqlite value.
#[derive(Debug)]
pub enum ValueRef<'a> {
    Null,
    Int(i32),
    Float(f64),
    Text(&'a str),
    Blob(&'a [u8]),
}

impl ValueRef<'_> {
    /// Bind current value to parameter at given index.
    ///
    /// Note that parameter index is one based.
    pub fn bind<S: Statement>(&self, idx: i32, handle: S) -> Result<(), BindError> {
        match *self {
            ValueRef::Null => handle.bind_null(idx)?,
            ValueRef::Int(int) => handle.bind_int(idx, int)?,
            ValueRef::Float(fl) => handle.bind_double(idx, fl)?,
            ValueRef::Text(t) => handle.bind_text(idx, t)?,
            ValueRef::Blob(b) => handle.bind_blob(idx, b)?,
        }
        Ok(())
    }

    pub fn decode<S: Statement>(idx: i32, handle: &S) -> Result<ValueRef<'_>, DecodeError> {
        let value = match handle.column_type(idx) {
            DataType::Null => ValueRef::Null,
            DataType::Int => ValueRef::Int(handle.column_int(idx)),
            DataType::Float => ValueRef::Float(handle.column_double(idx)),
            DataType::Text => ValueRef::Text(handle.column_text(idx)?),
            DataType::Blob => ValueRef::Blob(handle.column_blob(idx)),
        };
        Ok(value)
    }
}

impl Clone for ValueRef<'_> {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for ValueRef<'_> { }

impl From<()> for ValueRef<'_> {
    fn from(_: ()) -> Self {
        Self::Null
    }
}

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


macro_rules! decode {
    ($fr:ty as $ty:ty) => {
        impl Decode<'_> for $ty {
            fn decode(value: ValueRef) -> Result<Self> {
                <$fr as Decode>::decode(value).map(Into::into)
            }
        }
    };
    ($ty:ty, $pat:pat => $expr:expr) => {
        impl Decode<'_> for $ty {
            fn decode(value: ValueRef) -> Result<Self> {
                match value {
                    $pat => Ok($expr),
                    _ => Err(DecodeError::InvalidDataType.into()),
                }
            }
        }
    };
}

/// A type that can be construced from sqlite value.
pub trait Decode<'a>: Sized {
    fn decode(value: ValueRef<'a>) -> Result<Self>;
}

impl<'a> Decode<'a> for &'a str {
    fn decode(value: ValueRef<'a>) -> Result<Self> {
        match value {
            ValueRef::Text(t) => Ok(t),
            _ => Err(DecodeError::InvalidDataType.into()),
        }
    }
}

impl<'a> Decode<'a> for &'a [u8] {
    fn decode(value: ValueRef<'a>) -> Result<Self> {
        match value {
            ValueRef::Blob(t) => Ok(t),
            _ => Err(DecodeError::InvalidDataType.into()),
        }
    }
}

decode!((), ValueRef::Null => ());
decode!(i32, ValueRef::Int(i) => i);
decode!(f64, ValueRef::Float(i) => i);
decode!(&str as String);
decode!(&[u8] as Vec<u8>);

impl<'a> ValueRef<'a> {
    pub fn try_decode<D: Decode<'a>>(&self) -> Result<D> {
        D::decode(*self)
    }
}


/// A type that can be construced from sqlite row.
pub trait FromRow: Sized {
    fn from_row(row: Row) -> Result<Self>;
}

macro_rules! from_tuple {
    ($($id:ident $i:literal),*) => {
        impl<$($id),*> FromRow for ($($id),*,)
        where
            $($id: for<'a> Decode<'a>),*
        {
            fn from_row(row: Row) -> Result<Self> {
                Ok((
                    $(row.try_column($i)?.try_decode::<$id>()?),*,
                ))
            }
        }
    };
}

from_tuple!(R1 0);
from_tuple!(R1 0,R2 1);
from_tuple!(R1 0,R2 1,R3 2);
from_tuple!(R1 0,R2 1,R3 2,R4 3);
from_tuple!(R1 0,R2 1,R3 2,R4 3,R5 4);
from_tuple!(R1 0,R2 1,R3 2,R4 3,R5 4,R6 5);
from_tuple!(R1 0,R2 1,R3 2,R4 3,R5 4,R6 5,R7 6);
