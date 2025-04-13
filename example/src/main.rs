#[derive(Debug, PartialEq, Eq, rhosql::FromRow)]
struct User {
    id: i32,
    name: String,
    item: String,
}

fn main() {
    env_logger::builder().filter_level(log::LevelFilter::Debug).init();
    basic().inspect_err(|e|eprintln!("{e}")).ok();
    low_level().inspect_err(|e|eprintln!("{e}")).ok();
}

fn basic() -> rhosql::Result<()> {
    use rhosql::{Connection, ValueRef};

    let mut db = Connection::open(":memory:")?;

    db.exec("create table if not exists users(name)",[])?;
    db.exec(
        "insert into users(name) values(?1)",
        [ValueRef::Text(&format!("john"))],
    )?;

    let stmt = db.prepare("select rowid,name,?1 from users")?;

    // scoped for `rows` drop and reset statement
    {
        let mut rows = stmt.bind([ValueRef::Text("Deez")])?;
        assert_eq!(rows.next_row()?, Some(User { id: 1, name: "john".into(), item: "Deez".into() }));
        assert_eq!(rows.next_row::<User>()?, None);
    }

    // cached
    let stmt = db.prepare("select rowid,name,?1 from users")?;

    {
        let mut rows = stmt.bind([ValueRef::Text("Cloak")])?;
        assert_eq!(rows.next_row()?, Some(User { id: 1, name: "john".into(), item: "Cloak".into() }));
        assert_eq!(rows.next_row::<User>()?, None);
    }

    Ok(())
}

fn low_level() -> rhosql::Result<()> {
    use rhosql::sqlite::{DataType, OpenFlag, SqliteHandle, StatementExt, StatementHandle, StepResult};

    // https://sqlite.org/cintro.html#summary

    let db = SqliteHandle::open_v2(c":memory:", OpenFlag::OPEN_READWRITE_CREATE)?;

    let stmt = StatementHandle::prepare_v2(&db, c"select 420,'content',?1")?;

    stmt.bind_text(1, c"GG")?;

    assert_eq!(stmt.step()?, StepResult::Row);

    assert_eq!(stmt.data_count(), 3);

    assert_eq!(stmt.column_type(0), DataType::Int);
    assert_eq!(stmt.column_type(1), DataType::Text);
    assert_eq!(stmt.column_type(2), DataType::Text);

    assert_eq!(stmt.column_int(0), 420);
    assert_eq!(stmt.column_text(1)?, "content");
    assert_eq!(stmt.column_text(2)?, "GG");

    assert_eq!(stmt.step()?, StepResult::Done);

    Ok(())
}

