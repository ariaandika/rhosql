use crate::common::General;
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
    Open(General),
    /// Sqlite Error Code
    Code(ffi::Error),
    /// string too large for sqlite (c_int::MAX)
    ///
    /// this error usually returned when performing a query with rust string
    StringTooLarge,
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
            ($($id:ident)* , $($tt:tt)*) => {
                match self {
                    $(Self::$id(e) => std::fmt::Display::fmt(e, f),)*
                    $($tt)*
                }
            };
        }
        foo! {
            Open Code,
            Self::NonUtf8Open(p) => write!(f, "Path is non UTF-8: {:?}", p.to_string_lossy()),
            Self::NulStringOpen(p) => write!(f, "Path contains nul string: {:?}", p.to_string_lossy()),
            Self::StringTooLarge => write!(f, "String too large for sqlite"),
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
        foo! { Open Code }
    }
}


