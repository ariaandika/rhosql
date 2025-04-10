use crate::{error::DecodeError, from_row::FromRow, row_stream::RowStream, sqlite::DataType, Result};

/// unencoded row buffer
#[derive(Debug)]
pub struct Row<'row,'stmt> {
    // we cannot borrow Statement here, cus mutable reference
    row_stream: &'row RowStream<'stmt>,
    col_count: i32,
}

impl<'row,'stmt> Row<'row,'stmt> {
    pub(crate) fn new(row_stream: &'row RowStream<'stmt>) -> Self {
        Self { col_count: row_stream.stmt().handle().data_count(), row_stream }
    }

    /// try get single column from given index
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

    fn stmt(&self) -> &crate::sqlite::StatementHandle {
        self.row_stream.stmt().handle()
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

