//! Implementation of `errno` functionality for Windows.
//!
//! Adapted from `src/libstd/sys/windows/os.rs` in the Rust distribution.

// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::char::{self, REPLACEMENT_CHARACTER};
use core::ptr;
use core::str;
use winapi::shared::minwindef::DWORD;
use winapi::shared::ntdef::WCHAR;
use winapi::um::winbase::{FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS};

use Errno;

fn from_utf16_lossy<'a>(input: &[u16], output: &'a mut [u8]) -> &'a str {
    let mut output_len = 0;
    for c in char::decode_utf16(input.iter().copied().take_while(|&x| x != 0)).map(|x| x.unwrap_or(REPLACEMENT_CHARACTER)) {
        let c_len = c.len_utf8();
        if c_len > output.len() - output_len { break; }
        c.encode_utf8(&mut output[output_len ..]);
        output_len += c_len;
    }
    unsafe { str::from_utf8_unchecked(&output[.. output_len]) }
}

pub fn with_description<F, T>(err: Errno, callback: F) -> T where
    F: FnOnce(Result<&str, Errno>) -> T
{
    // This value is calculated from the macro
    // MAKELANGID(LANG_SYSTEM_DEFAULT, SUBLANG_SYS_DEFAULT)
    let lang_id = 0x0800 as DWORD;

    let mut buf = [0 as WCHAR; 2048];

    unsafe {
        let res = ::winapi::um::winbase::FormatMessageW(FORMAT_MESSAGE_FROM_SYSTEM |
                                           FORMAT_MESSAGE_IGNORE_INSERTS,
                                           ptr::null_mut(),
                                           err.0 as DWORD,
                                           lang_id,
                                           buf.as_mut_ptr(),
                                           buf.len() as DWORD,
                                           ptr::null_mut());
        if res == 0 {
            // Sometimes FormatMessageW can fail e.g. system doesn't like lang_id
            let fm_err = errno();
            return callback(Err(fm_err));
        }

        let mut msg = [0u8; 2048];
        let msg = from_utf16_lossy(&buf[..res as usize], &mut msg[..]);
        // Trim trailing CRLF inserted by FormatMessageW
        #[allow(deprecated)] // TODO: remove when MSRV >= 1.30
        callback(Ok(msg.trim_right()))
    }
}

pub const STRERROR_NAME: &'static str = "FormatMessageW";

pub fn errno() -> Errno {
    unsafe {
        Errno(::winapi::um::errhandlingapi::GetLastError() as i32)
    }
}

pub fn set_errno(Errno(errno): Errno) {
    unsafe {
        ::winapi::um::errhandlingapi::SetLastError(errno as DWORD)
    }
}
