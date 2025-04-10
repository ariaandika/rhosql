//! an error which can occur in sqlite operation
use libsqlite3_sys::{self as ffi};
use std::{ffi::NulError, str::Utf8Error};

macro_rules! display_error {
    (@@
        $self:ident, $me:ident, $f:ident,
        $(, #prefix $s:literal)?
        $(, #delegate $($id:ident)*)?
        $(, $($id2:pat => ($fmt:literal$($tt:tt)*)),* )? $(,)?
    ) => {
        $(
            if let Err(err) = write!($f, $s) {
                return Err(err);
            }
        )?
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
            #[doc = "return is this error is a SQLITE_BUSY"]
            ///
            /// the database engine was unable to acquire the database locks it needs to do its job
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
                    Self::$id(value)
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
        #[doc = concat!("an error when failed to ",$display)]
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

pub type Result<T,E = Error> = std::result::Result<T,E>;

/// an error code returned as result code and its message via `sqlite3_errmsg()`
#[derive(Debug)]
pub struct DatabaseError {
    pub message: String,
    pub code: i32,
}

impl DatabaseError {
    /// create new [`DatabaseError`]
    pub fn new(message: String, code: i32) -> Self {
        Self { message, code }
    }
}

is_busy!(DatabaseError, me => me.code == ffi::SQLITE_BUSY);
display_error!(DatabaseError, me => write!("{} ({})", me.message, me.code));

/// an error when failed to convert rust to sqlite string
#[derive(Debug)]
pub enum StringError {
    /// string too large, sqlite max string is i32::MAX
    TooLarge,
    /// value is not UTF-8
    Utf8,
    /// the supplied bytes contain an internal 0 byte
    NulError(NulError),
}

from!(StringError, for NulError => NulError);

display_error! {
    StringError,
    #prefix "Failed to convert rust to sqlite string: ",
    #delegate NulError,
    Self::TooLarge => ("string too large, sqlite max string is i32::MAX"),
    Self::Utf8 => ("value is not UTF-8")
}

/// an error when failed to open a database connection
#[derive(Debug)]
pub enum OpenError {
    /// setup error that sqlite is not in Serialized mode
    ///
    /// <https://www.sqlite.org/threadsafe.html>
    NotSerializeMode,
    /// an error when failed to convert rust to sqlite string
    String(StringError),
    /// an error returned from database
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
    for StringError => String
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

/// an error when failed to decode value
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
    #prefix "Failed to decode value: ",
    #delegate String Database
}

/// an error when failed to decode value
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

#[derive(Debug)]
pub enum Error {
    /// an error when failed to open a database
    Open(OpenError),
    /// an error when failed to configure database
    Configure(ConfigureError),
    /// an error when failed to create prepared statement
    Prepare(PrepareError),
    /// an error when failed to bind value to parameter
    Bind(BindError),
    /// an error when failed to get the next row
    Step(StepError),
    /// an error when failed to decode value
    Decode(DecodeError),
    /// an error when failed to reset or clear binding prepared statement
    Reset(ResetError),
}

from! {
    Error,
    for OpenError => Open,
    for ConfigureError => Configure,
    for PrepareError => Prepare,
    for BindError => Bind,
    for StepError => Step,
    for DecodeError => Decode,
    for ResetError => Reset
}

display_error! {
    Error,
    #delegate Open Configure Prepare Bind Step Decode Reset
}

