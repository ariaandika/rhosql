use libsqlite3_sys::{self as ffi};
use std::{
    borrow::Cow,
    ffi::{CStr, CString, NulError, c_char, c_int},
};

use crate::Result;
use crate::sqlite::error::StringError;

pub(crate) mod stack;

pub(crate) mod sealed {
    pub trait Sealed {}
}

/// Conversion between sqlite and rust string
///
/// Sqlite and rust string are different:
///
/// - sqlite string max length is `i32::MAX`, while rust is `usize::MAX`
/// - sqlite string may nul terminated, while rust is not
///
/// [`SqliteStr`] is implemented to:
///
/// - `str`, `String`, with runtime check and allocation is required for `to_nul_string`
/// - `CStr` and `CString` without runtime check nor allocation for `to_nul_string`
///
/// so its could have performance improvement querying with cstr, `c"SELECT * FROM users"`
pub trait SqliteStr: sealed::Sealed + std::fmt::Debug {
    /// destructor string to pointer, nul terminator *excluded* length, and a sqlite destructor
    fn as_sqlite_str(&self) -> Result<(*const c_char, c_int, ffi::sqlite3_destructor_type), StringError>;

    /// destructor string to pointer, maybe nul included length, and a sqlite destructor
    fn as_nulstr(&self) -> (*const c_char, c_int, libsqlite3_sys::sqlite3_destructor_type);

    /// make sure string is nul terminated
    ///
    /// using `CStr` will avoid allocation since its already nul terminated
    ///
    /// so its could have performance improvement querying with cstr, `c"SELECT * FROM users"`
    ///
    /// return error if string contains nul byte in the middle
    fn to_nul_string(&self) -> Result<Cow<'_,CStr>, NulError>;
}

// somehow blanket implementation doesnt work
macro_rules! ref_impl {
    ($ty:ty) => {
        impl sealed::Sealed for $ty { }
        impl sealed::Sealed for &$ty { }
        impl SqliteStr for &$ty {
            fn as_sqlite_str(&self) -> Result<(*const c_char, c_int, libsqlite3_sys::sqlite3_destructor_type), StringError> {
                <$ty>::as_sqlite_str(*self)
            }

            fn as_nulstr(&self) -> (*const c_char, c_int, libsqlite3_sys::sqlite3_destructor_type) {
                <$ty>::as_nulstr(*self)
            }

            fn to_nul_string(&self) -> Result<Cow<'_,CStr>, NulError> {
                <$ty>::to_nul_string(*self)
            }
        }
    };
}

ref_impl!(CStr);

impl SqliteStr for CStr {
    fn as_sqlite_str(&self) -> Result<(*const c_char, i32, ffi::sqlite3_destructor_type), StringError> {
        let Ok(len) = c_int::try_from(self.count_bytes()) else {
            return Err(StringError::TooLarge);
        };

        let (ptr, dtor_info) = match len {
            0 => ("".as_ptr().cast(), ffi::SQLITE_STATIC()),
            _ => (self.as_ptr().cast(), ffi::SQLITE_TRANSIENT()),
        };

        Ok((ptr, len, dtor_info))
    }

    fn as_nulstr(&self) -> (*const c_char, c_int, ffi::sqlite3_destructor_type) {
        match c_int::try_from(self.count_bytes().saturating_add(1)) {
            Ok(1) => (c"".as_ptr().cast(), 1, ffi::SQLITE_STATIC()),
            Ok(len) => (self.as_ptr().cast(), len, ffi::SQLITE_TRANSIENT()),
            Err(_) => (self.as_ptr().cast(), c_int::MAX, ffi::SQLITE_TRANSIENT()),
        }
    }

    fn to_nul_string(&self) -> Result<Cow<'_,CStr>, NulError> {
        Ok(Cow::Borrowed(self))
    }
}

ref_impl!(str);

impl SqliteStr for str {
    fn as_sqlite_str(&self) -> Result<(*const c_char, c_int, ffi::sqlite3_destructor_type), StringError> {
        let Ok(len) = c_int::try_from(self.len()) else {
            return Err(StringError::TooLarge);
        };

        let (ptr, dtor_info) = match len {
            0 => ("".as_ptr().cast(), ffi::SQLITE_STATIC()),
            _ => (self.as_ptr().cast(), ffi::SQLITE_TRANSIENT()),
        };

        Ok((ptr, len, dtor_info))
    }

    fn as_nulstr(&self) -> (*const c_char, c_int, ffi::sqlite3_destructor_type) {
        match c_int::try_from(self.len()) {
            Ok(0) => ("".as_ptr().cast(), 0, ffi::SQLITE_STATIC()),
            Ok(len) => (self.as_ptr().cast(), len, ffi::SQLITE_TRANSIENT()),
            Err(_) => (self.as_ptr().cast(), c_int::MAX, ffi::SQLITE_TRANSIENT()),
        }
    }

    fn to_nul_string(&self) -> Result<Cow<'_,CStr>, NulError> {
        CString::new(self).map(Cow::Owned)
    }
}

ref_impl!(CString);

impl SqliteStr for CString {
    fn as_sqlite_str(&self) -> Result<(*const c_char, c_int, libsqlite3_sys::sqlite3_destructor_type), StringError> {
        self.as_c_str().as_sqlite_str()
    }

    fn as_nulstr(&self) -> (*const c_char, c_int, libsqlite3_sys::sqlite3_destructor_type) {
        self.as_c_str().as_nulstr()
    }

    fn to_nul_string(&self) -> Result<Cow<'_,CStr>, NulError> {
        self.as_c_str().to_nul_string()
    }
}

ref_impl!(String);

impl SqliteStr for String {
    fn as_sqlite_str(&self) -> Result<(*const c_char, c_int, libsqlite3_sys::sqlite3_destructor_type), StringError> {
        self.as_str().as_sqlite_str()
    }

    fn as_nulstr(&self) -> (*const c_char, c_int, libsqlite3_sys::sqlite3_destructor_type) {
        self.as_str().as_nulstr()
    }

    fn to_nul_string(&self) -> Result<Cow<'_,CStr>, NulError> {
        self.as_str().to_nul_string()
    }
}

