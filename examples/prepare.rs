use std::ptr;

use libsqlite3_sys::{self as ffi};

fn main() {
    let mut db = ptr::null_mut();

    let result = unsafe {
        ffi::sqlite3_open_v2(
            c":memory:".as_ptr(),
            &mut db,
            ffi::SQLITE_OPEN_READWRITE,
            ptr::null(),
        )
    };

    if result != ffi::SQLITE_OK {
        panic!("{}", ffi::code_to_str(result))
    }

    let mut stmt = ptr::null_mut();

    let result =
        unsafe { ffi::sqlite3_prepare_v2(db, c"select 1".as_ptr(), 9, &mut stmt, ptr::null_mut()) };

    if result != ffi::SQLITE_OK {
        panic!("{}", ffi::code_to_str(result))
    }

    // let result = unsafe { ffi::sqlite3_close(db) };
    //
    // if result != ffi::SQLITE_OK {
    //     panic!("{}", ffi::code_to_str(result))
    // }

    let result = unsafe { ffi::sqlite3_step(stmt) };

    if result != ffi::SQLITE_OK {
        panic!("{}", ffi::code_to_str(result))
    }

}

