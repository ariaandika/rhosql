//! A safe interface to sqltite ffi
//!
//! this is low level interface that mimic how sqlite3 api are formed

mod error;
mod ffi;

mod database;
mod statement;
mod open_flag;
mod mutex;

pub use ffi::{Database, DatabaseExt};
pub use database::SqliteHandle;
pub use statement::StatementHandle;
pub use open_flag::OpenFlag;
pub use mutex::SqliteMutexGuard;

macro_rules! flags {
    ($id:ident, $($fl:ident => $name:ident),* $(,)?) => {
        pub enum $id {
            $($name),*
        }

        impl $id {
            pub fn from_code(code: i32) -> Option<Self> {
                match code {
                    $(libsqlite3_sys::$fl => Some(Self::$name)),*,
                    _ => None
                }
            }
        }
    };
}

flags! {
    DataType,
    SQLITE_NULL => Null,
    SQLITE_INTEGER => Int,
    SQLITE_FLOAT => Float,
    SQLITE_TEXT => Text,
    SQLITE_BLOB => Blob,
}

flags! {
    StepResult,
    SQLITE_DONE => Done,
    SQLITE_ROW => Row,
}

