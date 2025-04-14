# Rhosql

Simple SQLite driver.

The `rusqlite` crate is just not sufficient for me, so i made my own.

## Usage

```rust
use rhosql::Connection;

// derive macro
#[derive(rhosql::FromRow)]
struct Post {
    id: i32,
    name: String,
}

fn main() -> rhosql::Result<()> {
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
```

## License
This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

