use libsqlite3_sys::{self as ffi};

pub type Result<T,E = Error> = std::result::Result<T,E>;

pub enum Error {
    /// Path is non UTF-8
    ///
    /// The filename argument is interpreted as UTF-8 for `sqlite3_open_v2()`
    NonUtf8Open(std::path::PathBuf),
    /// Path contains 0 bytes
    ///
    /// For conversion to `CString`, path should *not* contain any 0 bytes in it.
    NulStringOpen(std::path::PathBuf),
    /// An English language description of the error following a failure of any of the `sqlite3_open()` routines.
    ///
    /// catured from `sqlite3_errmsg()`
    Open(String),
    /// Failure of calling `sqlite3_prepare()`
    ///
    /// catured from `sqlite3_errmsg()`
    Prepare(String),
    /// Failure of calling `sqlite3_step()`
    ///
    /// catured from `sqlite3_errmsg()`
    Step(String),
    /// Sqlite Error Code
    Code(ffi::Error),
    /// string too large for sqlite (c_int::MAX)
    ///
    /// this error usually returned when performing a query with rust string
    StringTooLarge,
    /// runtime check error that database already closed
    AlreadyClosed,
    /// setup error that sqlite is not in Serialized mode
    ///
    /// <https://www.sqlite.org/threadsafe.html>
    NonSerialized,
    /// the database engine was unable to acquire the database locks it needs to do its job
    ///
    ///  If the statement is a COMMIT or occurs outside of an explicit transaction, then you can retry the statement.
    ///  If the statement is not a COMMIT and occurs within an explicit transaction then you should rollback the
    ///  transaction before continuing.
    SqliteBusy,
    /// [`RowBuffer::try_column`] given index is out of bounds
    ///
    /// [`RowBuffer::try_column`]: crate::row_buffer::RowBuffer::try_column
    IndexOutOfBounds,
    /// [`RowBuffer::try_column`] given data type is mismatch
    ///
    /// [`RowBuffer::try_column`]: crate::row_buffer::RowBuffer::try_column
    InvalidDataType,
    /// [`RowBuffer::try_column`] sqlite return non UTF-8 for text
    ///
    /// [`RowBuffer::try_column`]: crate::row_buffer::RowBuffer::try_column
    NonUtf8Sqlite(std::str::Utf8Error)
}

macro_rules! from {
    ($($to:ty => $id:ident),* , $(<$t2:ty> $id2:pat => $b2:expr),*) => {
        $(
            impl From<$to> for Error {
                fn from(value: $to) -> Self {
                    Self::$id(value)
                }
            }
        )*
        $(
            impl From<$($t2)*> for Error {
                fn from($id2: $($t2)*) -> Self {
                    $b2
                }
            }
        )*
    };
}

from! {
    ffi::Error => Code,
}

impl std::error::Error for Error { }

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        macro_rules! foo {
            ($($id:ident)* , $($id2:pat => ($fmt:literal$($tt:tt)*)),* $(,)?) => {
                match self {
                    $(Self::$id(e) => std::fmt::Display::fmt(e, f),)*
                    $($id2 => write!(f, $fmt $($tt)*)),*
                }
            };
        }
        foo! {
            Code,
            Self::Open(m) => ("Failed to open database: {m:?}"),
            Self::Prepare(m) => ("Failed to prepare statement: {m:?}"),
            Self::Step(m) => ("Failed to read row: {m:?}"),
            Self::NonUtf8Open(p) => ("Path is non UTF-8: {:?}", p.to_string_lossy()),
            Self::NulStringOpen(p) => ("Path contains nul string: {:?}", p.to_string_lossy()),
            Self::StringTooLarge => ("String too large for sqlite"),
            Self::AlreadyClosed => ("Database already closed"),
            Self::NonSerialized => ("Sqlite is not in Serialized mode"),
            Self::SqliteBusy => ("SQLITE_BUSY, the database engine was unable to acquire the database locks"),
            Self::IndexOutOfBounds => ("Row index out of bounds"),
            Self::InvalidDataType => ("Datatype requested invalid"),
            Self::NonUtf8Sqlite(err) => ("Sqlite returns non UTF-8 text: {err}"),
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        macro_rules! foo {
            ($($id:ident)*) => {
                match self {
                    $(Self::$id(e) => std::fmt::Debug::fmt(e, f),)*
                    me => std::fmt::Display::fmt(me, f),
                }
            };
        }
        foo! { Code }
    }
}


