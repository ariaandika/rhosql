
// modules structure
//
// DatabaseHandle: raw handle, safe ffi call, close db on drop
// Connection: hold db handle, cache prepared statement
// Pool: hold multiple Connection

mod common;
mod pool;

pub mod sqlite;

mod connection;

pub mod statement;
pub mod row_stream;
pub mod row;
pub mod from_row;
pub mod query;

pub mod error;

pub use common::SqliteStr;
pub use connection::Connection;
pub use row::Row;
pub use from_row::FromRow;
pub use error::{Result, Error};

pub use rhosql_macros::FromRow;

