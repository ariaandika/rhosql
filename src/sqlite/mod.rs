//! A safe interface to sqlite ffi.
//!
//! This is low level interface, user typically does work with it directly.

pub mod error;

mod open_flag;
mod database;

mod raii;
mod statement;
mod mutex;

pub use error::DatabaseError;
pub use database::{Database, DatabaseExt};
pub use raii::SqliteHandle;
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

