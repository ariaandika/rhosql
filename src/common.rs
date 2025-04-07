use libsqlite3_sys::{self as ffi};
use std::ffi::{c_char, c_int};

use crate::{Error, Result};

pub trait FfiExt {
    /// Returns `Ok((string ptr, len as c_int, SQLITE_STATIC | SQLITE_TRANSIENT))` normally.
    ///
    /// Returns error if the string is too large for sqlite. (c_int::MAX = 2147483647)
    ///
    /// The `sqlite3_destructor_type` item is always `SQLITE_TRANSIENT` unless
    /// the string was empty (in which case it's `SQLITE_STATIC`, and the ptr is static).
    fn as_sqlite_cstr(&self) -> Result<(*const c_char, c_int, ffi::sqlite3_destructor_type)>;
}

impl FfiExt for str {
    fn as_sqlite_cstr(&self) -> Result<(*const c_char, c_int, ffi::sqlite3_destructor_type)> {
        self.as_bytes().as_sqlite_cstr()
    }
}

impl FfiExt for [u8] {
    fn as_sqlite_cstr(&self) -> Result<(*const c_char, c_int, ffi::sqlite3_destructor_type)> {
        let len = c_int::try_from(self.len()).map_err(|_|Error::StringTooLarge)?;
        let (ptr, dtor_info) = if len != 0 {
            (self.as_ptr().cast::<c_char>(), ffi::SQLITE_TRANSIENT())
        } else {
            // Return a pointer guaranteed to live forever
            ("".as_ptr().cast::<c_char>(), ffi::SQLITE_STATIC())
        };
        Ok((ptr, len, dtor_info))
    }
}

