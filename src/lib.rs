
// modules structure
//
// DatabaseHandle: raw handle, safe ffi call, close db on drop
// Connection: hold db handle, cache prepared statement
// Pool: hold multiple Connection

// internal utility
mod common;

// low level api
pub mod sqlite;

// high level api
mod connection;
mod statement;
mod row_stream;
mod row;

// error
pub mod error;

// utility api
mod pool;
pub mod query;


// reexports

pub use common::SqliteStr;
pub use connection::Connection;
pub use statement::Statement;
pub use row::{Row, ValueRef, Decode, FromRow};
pub use error::{Result, Error};
pub use rhosql_macros::FromRow;

