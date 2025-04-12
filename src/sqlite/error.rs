use libsqlite3_sys::{self as ffi};
use std::ffi::CStr;

use crate::error::DatabaseError;

/// check if result `SQLITE_OK`, otherwise treat as an [`DatabaseError`]
///
/// make sure the possible non error code is only `SQLITE_OK`
pub fn try_result<E: From<DatabaseError>>(db: *mut ffi::sqlite3, result: i32) -> Result<(), E> {
    match result {
        ffi::SQLITE_OK => Ok(()),
        _ => Err(self::db_error(db, result)),
    }
}

/// convert result code into [`DatabaseError`]
pub fn db_error<E: From<DatabaseError>>(db: *mut ffi::sqlite3, result: i32) -> E {
    if ffi::SQLITE_MISUSE == result {
        panic!("(bug) sqlite returns SQLITE_MISUSE")
    }

    let data = unsafe { ffi::sqlite3_errmsg(db) };
    if data.is_null() {
        return DatabaseError::new(ffi::code_to_str(result).into(), result).into();
    }

    let msg = match unsafe { CStr::from_ptr(data) }.to_str() {
        Ok(ok) => ok.into(),
        Err(_) => ffi::code_to_str(result).into(),
    };

    DatabaseError::new(msg, result).into()
}

