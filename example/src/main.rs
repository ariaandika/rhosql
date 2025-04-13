#[derive(Debug, PartialEq, Eq, rhosql::FromRow)]
struct User {
    id: i32,
    name: String,
    item: String,
}

fn main() {
    env_logger::builder().filter_level(log::LevelFilter::Debug).init();
    basic().inspect_err(|e|log::error!("{e}")).ok();
    low_level().inspect_err(|e|log::error!("{e}")).ok();
    multithread_mutex().inspect_err(|e|log::error!("{e}")).ok();
}

fn basic() -> rhosql::Result<()> {
    use rhosql::sqlite::DatabaseExt;
    use rhosql::{Connection, ValueRef};

    let mut db = Connection::open(":memory:")?;
    let name = "john".to_string();

    db.exec("create table users(name)",[])?;
    db.exec(
        "insert into users(name) values(?1)",
        [ValueRef::Text(&name)],
    )?;

    let id = db.last_insert_rowid() as _;

    let stmt = db.prepare("select rowid,name,?1 from users")?;

    // scoped for `rows` drop and reset statement
    {
        let item = "Deez".to_string();
        let mut rows = stmt.bind([ValueRef::Text(&item)])?;
        assert_eq!(rows.next_row()?, Some(User { id, name: name.clone(), item }));
        assert_eq!(rows.next_row::<User>()?, None);
    }

    // cached
    let stmt = db.prepare("select rowid,name,?1 from users")?;

    {
        let item = "Cloak".to_string();
        let mut rows = stmt.bind([ValueRef::Text(&item)])?;
        assert_eq!(rows.next_row()?, Some(User { id, name: name.clone(), item }));
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

fn multithread_mutex() -> rhosql::Result<()> {
    use std::{sync::{Arc, Mutex}, thread};
    use rhosql::Connection;

    let db = Arc::new(Mutex::new(Connection::open(":memory:")?));

    fn call(db: Arc<Mutex<Connection>>) -> Result<(), rhosql::Error> {
        let mut lock = db.lock().unwrap();
        lock.exec(c"create table if not exists foo(a)", [])?;
        lock.exec(c"insert into foo(a) values('deez')", [])?;
        Ok(())
    }

    let db1 = db.clone();
    let db2 = db.clone();
    let t1 = thread::spawn(move||call(db1));
    let t2 = thread::spawn(move||call(db2));
    t1.join().unwrap()?;
    t2.join().unwrap()?;

    {
        let mut lock = db.lock().unwrap();
        let stmt = lock.prepare(c"select rowid from foo")?;
        let mut rows = stmt.bind([])?;
        {
            let row = rows.next()?.unwrap();
            let value = row.try_column(0)?.try_decode::<i32>()?;
            assert_eq!(value, 1);
        }
        {
            let row = rows.next()?.unwrap();
            let value = row.try_column(0)?.try_decode::<i32>()?;
            assert_eq!(value, 2);
        }
        assert!(rows.next()?.is_none())
    }

    Ok(())
}

