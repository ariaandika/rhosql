use rhosql::{row_buffer::ValueRef, Connection, Result};

fn main() -> Result<()> {
    let db = Connection::open(":memory:")?;
    let db2 = db.clone();
    let db3 = db.clone();
    let h1 = std::thread::spawn(move||run(1,db2));
    let h2 = std::thread::spawn(move||run(2,db3));

    drop(db);
    h1.join().unwrap()?;
    h2.join().unwrap()?;

    Ok(())
}

fn run(id: i32, db: Connection) -> Result<()> {
    // one liner
    db.exec("create table if not exists foo(a,b)",[])?;
    db.exec(
        "insert into foo(a,b) values(?1,?2)",
        [ValueRef::Text("deez"),ValueRef::Text("foo")],
    )?;


    // verbose, high control
    let mut stmt = db.prepare("select rowid,* from foo")?;
    let mut row_stream = stmt.bind([])?;
    while let Some(row) = row_stream.next()? {
        use std::fmt::Write;
        let mut buffer = String::new();
        writeln!(buffer,"--").unwrap();
        writeln!(buffer,"{id}. {:?}",row.try_column(0)).unwrap();
        writeln!(buffer,"{id}. {:?}",row.try_column(1)).unwrap();
        writeln!(buffer,"{id}. {:?}",row.try_column(2)).unwrap();
        println!("{buffer}");
    }

    Ok(())
}

