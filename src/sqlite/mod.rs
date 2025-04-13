//! A safe interface to sqlite ffi.

pub mod error;

mod open_flag;
mod database;
mod statement;
mod raii;

pub use error::DatabaseError;
pub use open_flag::OpenFlag;
pub use database::{Database, DatabaseExt};
pub use statement::{Statement, StatementExt};
pub use raii::{SqliteHandle, StatementHandle, SqliteMutexGuard};

macro_rules! flags {
    ($(#[$doc:meta])* $id:ident, $($(#[$doc2:meta])* $fl:ident => $name:ident),* $(,)?) => {
        $(#[$doc])*
        #[derive(Debug, PartialEq, Eq)]
        pub enum $id {
            $($(#[$doc2])* $name),*
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
    /// Return type for [`sqlite3_column_type`][ty].
    ///
    /// Sqlite fundamental datatype.
    ///
    /// <https://sqlite.org/c3ref/c_blob.html>
    ///
    /// [ty]: StatementExt::column_type
    DataType,
    SQLITE_NULL => Null,
    SQLITE_INTEGER => Int,
    SQLITE_FLOAT => Float,
    SQLITE3_TEXT => Text,
    SQLITE_BLOB => Blob,
}

flags! {
    /// Return type for [`sqlite3_step`][ty].
    ///
    /// <https://sqlite.org/c3ref/step.html>
    ///
    /// [ty]: StatementExt::step
    StepResult,
    /// The statement has finished executing successfully.
    SQLITE_DONE => Done,
    /// A new row of data is ready for processing by the caller.
    SQLITE_ROW => Row,
}

impl StepResult {
    /// Return `true` if statement has finished executing successfully.
    pub fn is_done(&self) -> bool {
        matches!(self, StepResult::Done)
    }

    /// Return `true` if a new row of data is ready for processing.
    pub fn is_row(&self) -> bool {
        matches!(self, StepResult::Row)
    }
}

/// Returns `true` if sqlite is compiled with serialize mode enabled
///
/// <https://www.sqlite.org/threadsafe.html#compile_time_selection_of_threading_mode>
pub fn is_threadsafe() -> bool {
    const SERIALIZE_MODE: i32 = 1;
    unsafe { libsqlite3_sys::sqlite3_threadsafe() == SERIALIZE_MODE }
}

