//! # Rhosql
//!
//! SQLite driver.
//!
//! The existing `rusqlite` crate is just not sufficient for me, so i made my own.
//!
//! ## Usage
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
//! let mut db = Connection::open_in_memory()?;
//!
//! // execute single statement
//! rhosql::query("create table post(name)", &mut db).execute()?;
//!
//! let id = rhosql::query("insert into post(name) values(?1)", &mut db)
//!     .bind("Control")
//!     .execute()?;
//!
//! // using custom struct
//! let posts = rhosql::query("select rowid,* from post", &mut db).fetch_all::<Post>()?;
//!
//! assert_eq!(posts[0].id as i64, id);
//! assert_eq!(posts[0].name, "Control");
//!
//! // using tuple
//! let posts = rhosql::query("select rowid,* from post", &mut db).fetch_all::<(i32, String)>()?;
//!
//! assert_eq!(posts[0].0 as i64, id);
//! assert_eq!(posts[0].1, "Control");
//!
//! // iterator based
//! let mut posts = rhosql::query("select rowid,* from post", &mut db).fetch()?;
//!
//! while let Some(post) = posts.next_row::<Post>()? {
//!     assert_eq!(post.id as i64, id);
//!     assert_eq!(post.name, "Control");
//! }
//! #   Ok(())
//! # }
//! ```

// internal utility
mod common;

// low level api
pub mod sqlite;

// query api
pub mod query;

// shared state
mod connection;
mod pool;

// subtypes
mod row_stream;
mod row;

// error
pub mod error;


// reexports
pub use common::SqliteStr;
pub use query::query;
pub use connection::Connection;
pub use row_stream::RowStream;
pub use row::{Decode, FromRow, Row, ValueRef};
pub use rhosql_macros::FromRow;
pub use error::{Result, Error};

