use libsqlite3_sys::{self as ffi};
use std::{
    ffi::{CStr, CString},
    path::Path,
    ptr,
};

use crate::{common::{general, FfiExt}, Error, Result};

pub struct Connection {
    db: *mut ffi::sqlite3,
}

impl Connection {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut db = ptr::null_mut();

        // The filename argument is interpreted as UTF-8 for `sqlite3_open_v2()`
        let c_path = path_to_cstring(path.as_ref())?;

        unsafe {
            let flags = ffi::SQLITE_OPEN_READWRITE | ffi::SQLITE_OPEN_CREATE;

            // This routine opens a connection to an SQLite database file and returns a database connection object.
            let result = ffi::sqlite3_open_v2(c_path.as_ptr(), &mut db, flags, ptr::null_mut());

            if result != ffi::SQLITE_OK {
                if db.is_null() {
                    Err(ffi::Error::new(result))?;
                } else {
                    let err = ffi::sqlite3_errmsg(db);
                    let err = CStr::from_ptr(err).to_string_lossy();
                    ffi::sqlite3_close(db);
                    Err(Error::Open(general!("{err}")))?;
                }
            }

            // NOTE: currently copied from rusqlite, idk what it does yet
            ffi::sqlite3_extended_result_codes(db, 1);

            let result = ffi::sqlite3_busy_timeout(db, 5000);

            if result != ffi::SQLITE_OK {
                let err = ffi::sqlite3_errmsg(db);
                let err = CStr::from_ptr(err).to_string_lossy();
                ffi::sqlite3_close(db);
                Err(Error::Open(general!("{err}")))?;
            }
        }

        Ok(Self { db })
    }

    pub fn query(&mut self, sql: &str) -> Result<()> {
        // To run an SQL statement, the application follows these steps:

        // - Create a prepared statement using sqlite3_prepare().

        let mut stmt = ptr::null_mut();

        unsafe {
            let (zsql,len,_) = sql.as_sqlite_cstr()?;

            let result = ffi::sqlite3_prepare_v2(self.db, zsql, len, &mut stmt, &mut ptr::null());
            if result != ffi::SQLITE_OK {
                todo!("failed to create prepare")
            }
            if stmt.is_null() {
                todo!("pp_stmt is null")
            }
        }

        // - Evaluate the prepared statement by calling sqlite3_step() one or more times.
        unsafe {
            loop {
                let result = ffi::sqlite3_step(stmt);
                match result {
                    ffi::SQLITE_ERROR => todo!("failed to step prepare"),
                    ffi::SQLITE_ROW => { }
                    ffi::SQLITE_DONE => break,
                    _ => unreachable!(),
                }

                // - For queries, extract results by calling sqlite3_column() in between two calls to sqlite3_step().
                match ffi::sqlite3_column_type(stmt, 0) {
                    ffi::SQLITE_INTEGER => println!{"SQLITE_INTEGER"},
                    ffi::SQLITE_FLOAT => println!{"SQLITE_FLOAT"},
                    ffi::SQLITE_TEXT => println!{"SQLITE_TEXT"},
                    ffi::SQLITE_BLOB => println!{"SQLITE_BLOB"},
                    ffi::SQLITE_NULL => println!{"SQLITE_NULL"},
                    _ => unreachable!()
                }
                let text = ffi::sqlite3_column_text(stmt, 0).cast::<std::ffi::c_char>();
                let gg = CStr::from_ptr(text);
                println!("{gg:?}");
            }
        }

        // - Destroy the prepared statement using sqlite3_finalize().
        unsafe {
            let _result = ffi::sqlite3_finalize(stmt);
        }

        // The foregoing is all one really needs to know in order to use SQLite effectively.
        // All the rest is optimization and detail.

        Ok(())
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        // Many applications destroy their database connections using calls to sqlite3_close() at shutdown.
        unsafe { ffi::sqlite3_close(self.db) };
    }
}


#[cfg(unix)]
fn path_to_cstring(path: &Path) -> Result<CString> {
    use std::os::unix::ffi::OsStrExt;
    CString::new(path.as_os_str().as_bytes())
        .map_err(|_|Error::NulStringOpen(path.to_owned()))
}

#[cfg(not(unix))]
/// The filename argument is interpreted as UTF-8 for `sqlite3_open_v2()`
fn path_to_cstring(path: &Path) -> Result<CString> {
    path.to_str()
        .ok_or_else(|| Error::NonUtf8Open(path.to_owned()))
        .and_then(|ok| CString::new(ok).map_err(|_| Error::NulStringOpen(path.to_owned())))
}


#[test]
fn test() {
    let mut db = Connection::open("db.sqlite").unwrap();
    db.query("select 'foobar'").unwrap();
}

