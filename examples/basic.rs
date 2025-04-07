use rhosql::{connection::Connection, Result};

fn main() -> Result<()> {
    let mut db = Connection::open(":memory:")?;

    {
        let mut stmt = db.prepare("create table foo(a,b)")?;
        let mut row_stream = stmt.bind();
        let _row_buffer = row_stream.next()?;
    }

    {
        let mut stmt = db.prepare("insert into foo(a,b) values('deez','foo')")?;
        let mut row_stream = stmt.bind();
        let _row_buffer = row_stream.next()?;
    }

    let mut stmt = db.prepare("select * from foo").unwrap();
    let mut row_stream = stmt.bind();
    let mut row_buffer = row_stream.next().unwrap().unwrap();

    dbg!(row_buffer.try_column(0)).ok();
    dbg!(row_buffer.try_column(1)).ok();

    Ok(())
}

