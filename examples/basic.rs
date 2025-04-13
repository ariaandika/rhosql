use rhosql::{Connection, FromRow, Result, ValueRef};

#[derive(Debug, PartialEq, Eq, FromRow)]
struct User {
    id: i32,
    name: String,
}

fn main() {
    app().inspect_err(|e|eprintln!("{e}")).ok();
}

fn app() -> Result<()> {
    let mut db = Connection::open(":memory:")?;

    // one liner
    db.exec("create table if not exists users(name)",[])?;
    db.exec(
        "insert into users(name) values(?1)",
        [ValueRef::Text(&format!("john"))],
    )?;


    let stmt = db.prepare("select rowid,name from users")?;
    let mut row_stream = stmt.bind([])?;
    while let Some(row) = row_stream.next()? {
        // iterate and decode each rows
        let _user: User = row.try_row()?;
    }

    Ok(())
}

