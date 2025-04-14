# SQLite Driver

The existing `rusqlite` crate is just not sufficient for me, so i made my own.

This driver provide:

- prepared statement lru caching
- `FromRow` derive trait
- connection sharing / pooling (WIP)

# Usage

```rust
use rhosql::Connection;

// derive macro
#[derive(rhosql::FromRow)]
struct Post {
    id: i32,
    name: String,
}

fn main() -> rhosql::Result<()> {
    let db = Connection::open_in_memory()?;

    // execute single statement
    rhosql::query("create table post(name)", &db).execute()?;

    let id = rhosql::query("insert into post(name) values(?1)", &db)
        .bind("Control")
        .execute()?;

    // using custom struct
    let posts = rhosql::query("select rowid,* from post", &db).fetch_all::<Post>()?;

    assert_eq!(posts[0].id as i64, id);
    assert_eq!(posts[0].name, "Control");

    // using tuple
    let posts = rhosql::query("select rowid,* from post", &db).fetch_all::<(i32, String)>()?;

    assert_eq!(posts[0].0 as i64, id);
    assert_eq!(posts[0].1, "Control");
    Ok(())
}
```

# Versioning

Currently this library does not follow semver, its api still changing.

