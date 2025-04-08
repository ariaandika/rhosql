use rhosql::{Connection, Result};

fn main() -> Result<()> {
    let db = Connection::open(":memory:")?;
    let db2 = db.clone();
    let h1 = std::thread::spawn(move||run(1,db));
    let h2 = std::thread::spawn(move||run(2,db2));

    h1.join().unwrap()?;
    h2.join().unwrap()?;

    Ok(())
}

fn run(id: i32, db: Connection) -> Result<()> {
    db.exec("create table if not exists foo(a,b)")?;
    db.exec("insert into foo(a,b) values('deez','foo')")?;

    let mut stmt = db.prepare("select rowid,* from foo")?;
    let mut row_stream = stmt.bind();

    while let Some(row) = row_stream.next()? {
        println!("{id}. {:?}",row.try_column(0));
        println!("{id}. {:?}",row.try_column(1));
    }

    Ok(())
}

