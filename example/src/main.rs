#[derive(Debug, PartialEq, Eq, rhosql::FromRow)]
struct User {
    id: i32,
    name: String,
    item: String,
}

fn main() {
    env_logger::init();
    query_api().unwrap();
    low_level().unwrap();
}

fn query_api() -> rhosql::Result<()> {
    use rhosql::Connection;

    // derive macro
    #[derive(rhosql::FromRow)]
    struct Post {
        id: i32,
        name: String,
    }

    let mut db = Connection::open_in_memory()?;

    // execute single statement
    rhosql::query("create table post(name)", &mut db).execute()?;

    let id = rhosql::query("insert into post(name) values(?1)", &mut db)
        .bind("Control")
        .execute()?;

    // using custom struct
    let posts = rhosql::query("select rowid,* from post", &mut db).fetch_all::<Post>()?;

    assert_eq!(posts[0].id as i64, id);
    assert_eq!(posts[0].name, "Control");

    // using tuple
    // also, prepared statement is cached
    let posts = rhosql::query("select rowid,* from post", &mut db).fetch_all::<(i32, String)>()?;

    assert_eq!(posts[0].0 as i64, id);
    assert_eq!(posts[0].1, "Control");

    // iterator based
    let mut posts = rhosql::query("select rowid,* from post", &mut db).fetch()?;

    while let Some(post) = posts.next_row::<Post>()? {
        assert_eq!(post.id as i64, id);
        assert_eq!(post.name, "Control");
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

