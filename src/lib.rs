
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
pub mod connection;
pub mod statement;
pub mod row_stream;
pub mod row;
pub mod from_row;

// error
pub mod error;

// utility api
mod pool;
pub mod query;



// reexports

pub use common::SqliteStr;
pub use connection::Connection;
pub use row::{Row, ValueRef};
pub use from_row::FromRow;
pub use error::{Result, Error};
pub use rhosql_macros::FromRow;

