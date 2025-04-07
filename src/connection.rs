use libsqlite3_sys::{self as ffi};
use std::{
    ffi::{CStr, CString},
    path::Path,
    ptr, sync::Arc,
};

use crate::{handle::SqliteHandle, statement::Statement, Error, Result};

/// database connection
#[derive(Clone)]
pub struct Connection {
    handle: Arc<SqliteHandle>,
}

// we checked that sqlite in Serialize mode
unsafe impl Send for Connection { }

impl Connection {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut db = ptr::null_mut();

        // The filename argument is interpreted as UTF-8 for `sqlite3_open_v2()`
        let c_path = path_to_cstring(path.as_ref())?;
        let flags = ffi::SQLITE_OPEN_READWRITE | ffi::SQLITE_OPEN_CREATE;

        let result = unsafe { ffi::sqlite3_open_v2(c_path.as_ptr(), &mut db, flags, ptr::null_mut()) };
        if result != ffi::SQLITE_OK {
            return match db.is_null() {
                true => Err(ffi::Error::new(result).into()),
                false => unsafe {
                    let err = ffi::sqlite3_errmsg(db);
                    let err = CStr::from_ptr(err).to_string_lossy().into_owned();
                    ffi::sqlite3_close(db);
                    Err(Error::Open(err))
                }
            }
        }

        // NOTE: currently copied from rusqlite, idk what it does yet
        unsafe {
            ffi::sqlite3_extended_result_codes(db, 1);

            let result = ffi::sqlite3_busy_timeout(db, 5000);

            if result != ffi::SQLITE_OK {
                let err = ffi::sqlite3_errmsg(db);
                let err = CStr::from_ptr(err).to_string_lossy().into_owned();
                ffi::sqlite3_close(db);
                Err(Error::Open(err))?;
            }
        }

        Ok(Self { handle: Arc::new(SqliteHandle::setup(db)?) })
    }

    pub fn prepare(&mut self, sql: &str) -> Result<Statement> {
        Statement::prepare(self.handle.clone(),sql)
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

