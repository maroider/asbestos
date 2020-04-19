use std::{ffi::CString, iter};

use winapi::um::{
    libloaderapi::{GetModuleHandleW, GetProcAddress},
    winnt::{LPCSTR, LPCWSTR},
};

/// Returns a module symbol's absolute address.
pub fn get_module_symbol_address(module: &str, symbol: &str) -> Option<usize> {
    let module = module
        .encode_utf16()
        .chain(iter::once(0))
        .collect::<Vec<u16>>();
    let symbol = CString::new(symbol).unwrap();
    unsafe {
        let handle = GetModuleHandleW(module.as_ptr());
        match GetProcAddress(handle, symbol.as_ptr()) as usize {
            0 => None,
            n => Some(n),
        }
    }
}

pub unsafe fn cwstrlen(pcwstr: LPCWSTR) -> usize {
    let mut len = 0;
    loop {
        let wchar = *(pcwstr.add(len));
        if wchar == 0 {
            break;
        } else {
            len += 1;
        }
    }
    len
}

pub unsafe fn cstrlen(pcstr: LPCSTR) -> usize {
    let mut len = 0;
    loop {
        let c = *(pcstr.add(len));
        if c == 0 {
            break;
        } else {
            len += 1;
        }
    }
    len
}
