use rhosql::{
    Connection, Result,
    from_row::FromRow,
    row::{Row, ValueRef},
};

#[derive(Debug, PartialEq, Eq)]
#[allow(unused)]
struct User {
    id: i32,
    name: String,
}

impl FromRow for User {
    fn from_row(row: Row) -> Result<Self> {
        Ok(Self {
            id: row.try_decode(0)?,
            name: row.try_decode(1)?,
        })
    }
}

fn main() {
    app().inspect_err(|e|eprintln!("{e}")).ok();
}

fn app() -> Result<()> {
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
    db.exec("create table if not exists users(name)",[])?;
    db.exec(
        "insert into users(name) values(?1)",
        [ValueRef::Text(&format!("john {id}"))],
    )?;


    let mut stmt = db.prepare("select rowid,name from users")?;
    let mut row_stream = stmt.bind([])?;
    while let Some(row) = row_stream.next()? {
        // iterate and decode each rows
        let _user: User = row.try_row()?;
    }

    Ok(())
}

