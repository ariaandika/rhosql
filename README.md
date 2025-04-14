# SQLite Driver

The existing `rusqlite` crate is just not sufficient for me, so i made my own.

This driver provide:

- prepared statement lru caching
- `FromRow` derive trait
- connection sharing / pooling (WIP)

# Versioning

Currently this library does not follow semver, its api still changing.

