//! Operation specific error
use libsqlite3_sys::{self as ffi};
use std::{
    ffi::{CStr, NulError},
    str::Utf8Error,
};

macro_rules! display_error {
    (@@
        $self:ident, $me:ident, $f:ident,
        $(, #prefix $s:literal)?
        $(, #delegate $($id:ident)*)?
        $(, $($id2:pat => ($fmt:literal$($tt:tt)*)),* )? $(,)?
    ) => {
        $(write!($f, $s)?;)?
        return match $me {
            $($(Self::$id(e) => std::fmt::Display::fmt(e,$f),)*)?
            $(
                $($id2 => write!($f, $fmt $($tt)*)),*
            )?
        }
    };
    (@@ $self:ident, $me:ident, $f:ident, , $pat:pat => write!($($tt:tt)*)) => {{
        let $pat = $me;
        return write!($f, $($tt)*);
    }};
    ($self:ident $($tt:tt)*) => {
        impl std::error::Error for $self { }
        impl std::fmt::Display for $self {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                display_error!(@@ $self, self, f, $($tt)*);
            }
        }
    };
}

macro_rules! is_busy {
    ($me:ty, $pr:pat => $($tt:tt)*) => {
        impl $me {
            /// Returns `true` if the error is a `SQLITE_BUSY`.
            ///
            /// `SQLITE_BUSY` occur when the database engine was unable to acquire the database locks it needs to do its job.
            ///
            /// If the statement is a COMMIT or occurs outside of an explicit transaction,
            /// then you can retry the statement. If the statement is not a COMMIT and occurs
            /// within an explicit transaction then you should rollback the transaction before continuing.
            pub fn is_busy(&self) -> bool {
                let $pr = self;
                $($tt)*
            }
        }
    };
}

macro_rules! from {
    (
        $me:ty,
        $(for $to:ty => $id:ident),* $(,)?
        $(<$t2:ty> $id2:pat => $b2:expr),* $(,)?
    ) => {
        $(
            impl From<$to> for $me {
                fn from(value: $to) -> Self {
                    Self::$id(value.into())
                }
            }
        )*
        $(
            impl From<$t2> for $me {
                fn from($id2: $t2) -> Self {
                    $b2
                }
            }
        )*
    };
}

macro_rules! opaque_error {
    (
        $me:ident,
        #failedto $display:literal $(,)?
    ) => {
        #[derive(Debug)]
        #[doc = concat!("An error when failed to ",$display)]
        pub struct $me(DatabaseError);

        from!($me, <DatabaseError> err => Self(err));

        is_busy!($me, me => me.0.is_busy());

        impl std::error::Error for $me { }
        impl std::fmt::Display for $me {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Failed to {}: {}", $display, self.0)
            }
        }
    };
}

pub(crate) use display_error;
pub(crate) use is_busy;
pub(crate) use from;
pub(crate) use opaque_error;

/// An error returned from database.
#[derive(Debug)]
pub struct DatabaseError {
    /// Error message via `sqlite3_errmsg()`.
    pub message: String,
    /// Error result code.
    pub code: i32,
}

impl DatabaseError {
    /// Convert result code into [`DatabaseError`].
    pub(crate) fn from_code(result: i32, db: *mut ffi::sqlite3) -> Self {
        let data = unsafe { ffi::sqlite3_errmsg(db) };
        if data.is_null() {
            return Self { message: ffi::code_to_str(result).into(), code: result }
        }

        let msg = match unsafe { CStr::from_ptr(data) }.to_str() {
            Ok(ok) => ok.into(),
            Err(_) => ffi::code_to_str(result).into(),
        };

        Self { message: msg, code: result }
    }
}

impl DatabaseError {
    /// Returns `true` if the error is a `SQLITE_BUSY`.
    ///
    /// `SQLITE_BUSY` occur when the database engine was unable to acquire the database locks it needs to do its job.
    ///
    /// If the statement is a COMMIT or occurs outside of an explicit transaction,
    /// then you can retry the statement. If the statement is not a COMMIT and occurs
    /// within an explicit transaction then you should rollback the transaction before continuing.
    pub fn is_busy(&self) -> bool {
        self.code == ffi::SQLITE_BUSY
    }
}

display_error!(DatabaseError, me => write!("{}", me.message));

/// An error when failed to convert rust to sqlite string.
#[derive(Debug)]
pub enum StringError {
    /// String too large, sqlite max string is i32::MAX.
    TooLarge,
    /// Value is not UTF-8.
    Utf8(Utf8Error),
    /// The supplied bytes contain an internal 0 byte.
    NulError(NulError),
}

from! {
    StringError,
    for NulError => NulError,
    for Utf8Error => Utf8
}

display_error! {
    StringError,
    #prefix "Failed to convert rust to sqlite string: ",
    #delegate NulError Utf8,
    Self::TooLarge => ("string too large, sqlite max string is i32::MAX"),
}

/// An error when failed to open a database connection
#[derive(Debug)]
pub enum OpenError {
    /// Sqlite is not in [Serialized][1] mode
    ///
    /// [1]: <https://www.sqlite.org/threadsafe.html>
    NotSerializeMode,
    /// An error when failed to convert rust to sqlite string
    String(StringError),
    /// An error returned from database
    Database(DatabaseError),
}

impl From<std::ffi::NulError> for OpenError {
    fn from(value: std::ffi::NulError) -> Self {
        Self::String(value.into())
    }
}

from! {
    OpenError,
    for DatabaseError => Database,
    for StringError => String,
    for Utf8Error => String
}

is_busy!(OpenError, me => matches!(me,OpenError::Database(d) if d.is_busy()));

display_error! {
    OpenError,
    #prefix "Failed to open a database: ",
    #delegate Database String,
    Self::NotSerializeMode => ("sqlite is not in serialize mode"),
}

opaque_error!(ConfigureError, #failedto "configure database");
opaque_error!(PrepareError, #failedto "create prepared statement");
opaque_error!(StepError, #failedto "get the next row");
opaque_error!(ResetError, #failedto "reset or clear binding prepared statement");

/// An error when failed to decode value
#[derive(Debug)]
pub enum BindError {
    String(StringError),
    Database(DatabaseError),
}

from! {
    BindError,
    for StringError => String,
    for DatabaseError => Database
}

display_error! {
    BindError,
    #prefix "Failed to bind value: ",
    #delegate String Database
}

/// An error when failed to decode value
#[derive(Debug)]
pub enum DecodeError {
    IndexOutOfBounds,
    InvalidDataType,
    Utf8(Utf8Error),
}

display_error! {
    DecodeError,
    #prefix "Failed to decode value: ",
    #delegate Utf8,
    Self::IndexOutOfBounds => ("row index out of bounds"),
    Self::InvalidDataType => ("datatype requested missmatch"),
}

