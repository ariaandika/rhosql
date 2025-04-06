use libsqlite3_sys::{self as ffi};
use std::{
    ffi::{CStr, CString},
    path::Path,
    ptr,
};

use crate::{error::general, BoxError};

pub struct Connection {
    db: *mut ffi::sqlite3,
}

impl Connection {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, BoxError> {
        let mut db = ptr::null_mut();
        let c_path = path_to_cstring(path.as_ref())?;

        unsafe {
            let flags = ffi::SQLITE_OPEN_READWRITE | ffi::SQLITE_OPEN_CREATE;

            // This routine opens a connection to an SQLite database file and returns a database connection object.
            let result = ffi::sqlite3_open_v2(c_path.as_ptr(), &mut db, flags, ptr::null_mut());

            if result != ffi::SQLITE_OK {
                if db.is_null() {
                    Err(ffi::Error::new(result))?
                } else {
                    let err = ffi::sqlite3_errmsg(db);
                    let err = CStr::from_ptr(err).to_string_lossy();
                    ffi::sqlite3_close(db);
                    Err(general!("{err}"))?
                }
            }

            // NOTE: currently copied from rusqlite, idk what it does yet
            ffi::sqlite3_extended_result_codes(db, 1);

            let result = ffi::sqlite3_busy_timeout(db, 5000);

            if result != ffi::SQLITE_OK {
                let err = ffi::sqlite3_errmsg(db);
                let err = CStr::from_ptr(err).to_string_lossy();
                ffi::sqlite3_close(db);
                Err(general!("{err}"))?
            }
        }

        Ok(Self { db })
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        // Many applications destroy their database connections using calls to sqlite3_close() at shutdown.
        unsafe { ffi::sqlite3_close(self.db) };
    }
}


#[cfg(unix)]
fn path_to_cstring(path: &Path) -> Result<CString, std::ffi::NulError> {
    use std::os::unix::ffi::OsStrExt;
    CString::new(path.as_os_str().as_bytes())
}

#[cfg(not(unix))]
fn path_to_cstring(path: &Path) -> Result<CString, std::ffi::NulError>  {
    todo!()
}


#[test]
fn test() {
    let _db = Connection::open("db.sqlite");
}

