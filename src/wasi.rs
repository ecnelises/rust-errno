//! Implementation of `errno` functionality for WASI.
//!
//! Adapted from `unix.rs`.

// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::str;
use libc::{self, c_char, c_int, strlen};

use Errno;

fn from_utf8_lossy(input: &[u8]) -> &str {
    match str::from_utf8(input) {
        Ok(valid) => valid,
        Err(error) => unsafe { str::from_utf8_unchecked(&input[.. error.valid_up_to()]) },
    }
}

pub fn with_description<F, T>(err: Errno, callback: F) -> T where
    F: FnOnce(Result<&str, Errno>) -> T
{
    let mut buf = [0u8; 1024];
    let c_str = unsafe {
        if strerror_r(err.0, buf.as_mut_ptr() as *mut _, buf.len() as libc::size_t) < 0 {
            let fm_err = errno();
            if fm_err != Errno(libc::ERANGE) {
                return callback(Err(fm_err));
            }
        }
        let c_str_len = strlen(buf.as_ptr() as *const _);
        &buf[.. c_str_len]
    };
    callback(Ok(from_utf8_lossy(c_str)))
}

pub const STRERROR_NAME: &'static str = "strerror_r";

pub fn errno() -> Errno {
    // libc_errno is thread-local, so simply read its value.
    unsafe {
        Errno(libc_errno)
    }
}

pub fn set_errno(Errno(new_errno): Errno) {
    // libc_errno is thread-local, so simply assign to it.
    unsafe {
        libc_errno = new_errno;
    }
}

extern {
    #[thread_local]
    #[link_name = "errno"]
    static mut libc_errno: c_int;

    fn strerror_r(errnum: c_int, buf: *mut c_char,
                  buflen: libc::size_t) -> c_int;
}
