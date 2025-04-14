//! # SQLite Driver
//!
//! The existing `rusqlite` crate is just not sufficient for me, so i made my own.
//!
//! # Usage
//!
//! ```
//! # fn main() -> rhosql::Result<()> {
//! use rhosql::Connection;
//!
//! // derive macro
//! #[derive(rhosql::FromRow)]
//! struct Post {
//!     id: i32,
//!     name: String,
//! }
//!
//! let db = Connection::open_in_memory()?;
//!
//! // execute single statement
//! rhosql::query("create table post(name)", &db).execute()?;
//!
//! let id = rhosql::query("insert into post(name) values(?1)", &db)
//!     .bind("Control")
//!     .execute()?;
//!
//! // using custom struct
//! let posts = rhosql::query("select rowid,* from post", &db).fetch_all::<Post>()?;
//!
//! assert_eq!(posts[0].id as i64, id);
//! assert_eq!(posts[0].name, "Control");
//!
//! // using tuple
//! let posts = rhosql::query("select rowid,* from post", &db).fetch_all::<(i32, String)>()?;
//!
//! assert_eq!(posts[0].0 as i64, id);
//! assert_eq!(posts[0].1, "Control");
//! #   Ok(())
//! # }
//! ```

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
pub use query::query;
pub use error::{Result, Error};
pub use rhosql_macros::FromRow;

