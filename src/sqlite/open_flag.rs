/// Database open flag.
///
/// The default is [`SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE`][OpenFlag::OPEN_READWRITE_CREATE].
///
/// <https://sqlite.org/c3ref/open.html>
#[derive(Clone, Copy)]
pub struct OpenFlag(pub(crate) i32);

/// The default is [`OpenFlag::OPEN_READWRITE_CREATE`]
impl Default for OpenFlag {
    fn default() -> Self {
        Self::OPEN_READWRITE_CREATE
    }
}

impl std::ops::BitOr for OpenFlag {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0.bitor(rhs.0))
    }
}

macro_rules! consts {
    ($($(#[$m:meta])* $id:ident => $($sq:ident)|*);* $(;)?) => {
        impl OpenFlag {
            $($(#[$m])*pub const $id: Self = Self($(libsqlite3_sys::$sq)|*);)*
        }
    };
}

consts! {
    /// The database is opened in read-only mode. If the database does not already exist, an error is returned.
    OPEN_READONLY => SQLITE_OPEN_READONLY;
    /// The database is opened for reading and writing if possible,
    /// or reading only if the file is write protected by the operating system.
    OPEN_READWRITE => SQLITE_OPEN_READWRITE;
    /// The database is opened for reading and writing, and is created if it does not already exist.
    OPEN_READWRITE_CREATE => SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE;
    /// The filename can be interpreted as a URI if this flag is set.
    OPEN_URI => SQLITE_OPEN_URI;
    /// The database will be opened as an in-memory database.
    ///
    /// The database is named by the "filename" argument for the purposes of cache-sharing,
    /// if shared cache mode is enabled, but the "filename" is otherwise ignored.
    OPEN_MEMORY => SQLITE_OPEN_MEMORY;
    /// The new database connection will use the "multi-thread" threading mode.
    ///
    /// This means that separate threads are allowed to use SQLite at the same time,
    /// as long as each thread is using a different database connection.
    OPEN_NOMUTEX => SQLITE_OPEN_NOMUTEX;
    /// The new database connection will use the "serialized" threading mode.
    ///
    /// This means the multiple threads can safely attempt to use the same database connection at the same time.
    ///
    /// (Mutexes will block any actual concurrency, but in this mode there is no harm in trying.)
    OPEN_FULLMUTEX => SQLITE_OPEN_FULLMUTEX;
    /// The database is opened shared cache enabled,
    /// overriding the default shared cache setting provided by sqlite3_enable_shared_cache().
    ///
    /// The use of shared cache mode is discouraged and hence shared cache capabilities may be
    /// omitted from many builds of SQLite. In such cases, this option is a no-op.
    OPEN_SHAREDCACHE => SQLITE_OPEN_SHAREDCACHE;
    /// The database is opened shared cache disabled,
    /// overriding the default shared cache setting provided by sqlite3_enable_shared_cache().
    OPEN_PRIVATECACHE => SQLITE_OPEN_PRIVATECACHE;
}

