

mod common;

mod handle;

pub mod connection;
pub mod statement;
pub mod row_stream;
pub mod row_buffer;
pub mod value_ref;

pub mod error;

pub use error::{Result, Error};



#[cfg(test)]
mod test {
    use crate::connection::Connection;

    #[test]
    fn test() {
        let mut db = Connection::open("db.sqlite").unwrap();
        let mut stmt = db.prepare("select 'foobar',420,true,null").unwrap();
        let mut row_stream = stmt.bind();
        let mut row_buffer = row_stream.next().unwrap().unwrap();
        dbg!(row_buffer.try_column(0)).ok();
        dbg!(row_buffer.try_column(1)).ok();
        dbg!(row_buffer.try_column(2)).ok();
        dbg!(row_buffer.try_column(3)).ok();
        dbg!(row_stream.next()).ok();
    }
}

