use libsqlite3_sys::{self as ffi};
use std::ffi::{c_char, c_int};

use crate::BoxError;

pub trait FfiExt {
    /// Returns `Ok((string ptr, len as c_int, SQLITE_STATIC | SQLITE_TRANSIENT))` normally.
    ///
    /// Returns error if the string is too large for sqlite.
    ///
    /// The `sqlite3_destructor_type` item is always `SQLITE_TRANSIENT` unless
    /// the string was empty (in which case it's `SQLITE_STATIC`, and the ptr is static).
    fn as_sqlite_cstr(&self) -> Result<(*const c_char, c_int, ffi::sqlite3_destructor_type), BoxError>;
}

impl FfiExt for str {
    fn as_sqlite_cstr(&self) -> Result<(*const c_char, c_int, ffi::sqlite3_destructor_type), BoxError> {
        self.as_bytes().as_sqlite_cstr()
    }
}

impl FfiExt for [u8] {
    fn as_sqlite_cstr(&self) -> Result<(*const c_char, c_int, ffi::sqlite3_destructor_type), BoxError> {
        let len = len_as_c_int(self.len())?;
        let (ptr, dtor_info) = if len != 0 {
            (self.as_ptr().cast::<c_char>(), ffi::SQLITE_TRANSIENT())
        } else {
            // Return a pointer guaranteed to live forever
            ("".as_ptr().cast::<c_char>(), ffi::SQLITE_STATIC())
        };
        Ok((ptr, len, dtor_info))
    }
}

// Helper to cast to c_int safely, returning the correct error type if the cast
// failed.
fn len_as_c_int(len: usize) -> Result<c_int, BoxError> {
    if len >= (c_int::MAX as usize) {
        todo!()
        // Err(err!(ffi::SQLITE_TOOBIG))
    } else {
        Ok(len as c_int)
    }
}

