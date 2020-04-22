use std::{ffi::OsString, os::windows::ffi::OsStringExt, ptr, slice};

use detour::static_detour;
use winapi::{
    shared::minwindef::{DWORD, HFILE, UINT},
    um::{
        minwinbase::LPSECURITY_ATTRIBUTES,
        winbase::LPOFSTRUCT,
        winnt::{HANDLE, LPCSTR, LPCWSTR},
    },
};

use asbestos_shared::{log_info, log_trace};

use crate::{
    decl_hook_init, get_conn,
    util::{cstrlen, cwstrlen},
};

static_detour! {
    static OpenFileHook: unsafe extern "system" fn(
        LPCSTR,
        LPOFSTRUCT,
        UINT
    ) -> HFILE;
}

type FnOpenFile = unsafe extern "system" fn(LPCSTR, LPOFSTRUCT, UINT) -> HFILE;

decl_hook_init!(
    OpenFileHook,
    FnOpenFile,
    OpenFile,
    openfile_hook,
    "kernel32.dll",
    openfile_detour
);

#[allow(non_snake_case)]
pub fn openfile_detour(lpFileName: LPCSTR, lpReOpenBuff: LPOFSTRUCT, uStyle: UINT) -> HFILE {
    let mut conn = get_conn();
    let conn = conn.as_mut().unwrap();

    let file_name = {
        if lpFileName != ptr::null_mut() {
            let file_name_len = unsafe { cstrlen(lpFileName) };
            let name_slice =
                unsafe { slice::from_raw_parts(lpFileName as *const u8, file_name_len) };
            let name_string = String::from_utf8_lossy(name_slice).into_owned();
            Some(name_string)
        } else {
            None
        }
    };

    log_info!(
        conn,
        "OpenFile(lpFileName: {})",
        file_name.unwrap_or_else(|| "[NULL POINTER]".into())
    )
    .ok();
    unsafe { OpenFileHook.call(lpFileName, lpReOpenBuff, uStyle) }
}

//

static_detour! {
    static CreateFileAHook: unsafe extern "system" fn(
        LPCSTR,
        DWORD,
        DWORD,
        LPSECURITY_ATTRIBUTES,
        DWORD,
        DWORD,
        HANDLE
    ) -> HANDLE;
}

type FnCreateFileA = unsafe extern "system" fn(
    LPCSTR,
    DWORD,
    DWORD,
    LPSECURITY_ATTRIBUTES,
    DWORD,
    DWORD,
    HANDLE,
) -> HANDLE;

decl_hook_init!(
    CreateFileAHook,
    FnCreateFileA,
    CreateFileA,
    createfilea_hook,
    "kernel32.dll",
    createfilea_detour
);

#[allow(non_snake_case)]
pub fn createfilea_detour(
    lpFileName: LPCSTR,
    dwDesiredAccess: DWORD,
    dwShareMode: DWORD,
    lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
    dwCreationDisposition: DWORD,
    dwFlagsAndAttributes: DWORD,
    hTemplateFile: HANDLE,
) -> HANDLE {
    let mut conn = get_conn();
    let conn = conn.as_mut().unwrap();

    let file_name = {
        if lpFileName != ptr::null_mut() {
            let file_name_len = unsafe { cstrlen(lpFileName) };
            let name_slice =
                unsafe { slice::from_raw_parts(lpFileName as *const u8, file_name_len) };
            let name_string = String::from_utf8_lossy(name_slice).into_owned();
            Some(name_string)
        } else {
            None
        }
    };

    log_info!(
        conn,
        "CreateFileA(lpFileName: {})",
        file_name.unwrap_or_else(|| "[NULL POINTER]".into())
    )
    .ok();
    unsafe {
        CreateFileAHook.call(
            lpFileName,
            dwDesiredAccess,
            dwShareMode,
            lpSecurityAttributes,
            dwCreationDisposition,
            dwFlagsAndAttributes,
            hTemplateFile,
        )
    }
}

//

static_detour! {
    static CreateFileWHook: unsafe extern "system" fn(
        LPCWSTR,
        DWORD,
        DWORD,
        LPSECURITY_ATTRIBUTES,
        DWORD,
        DWORD,
        HANDLE
    ) -> HANDLE;
}

type FnCreateFileW = unsafe extern "system" fn(
    LPCWSTR,
    DWORD,
    DWORD,
    LPSECURITY_ATTRIBUTES,
    DWORD,
    DWORD,
    HANDLE,
) -> HANDLE;

decl_hook_init!(
    CreateFileWHook,
    FnCreateFileW,
    CreateFileW,
    createfilew_hook,
    "kernel32.dll",
    createfilew_detour
);

#[allow(non_snake_case)]
pub fn createfilew_detour(
    lpFileName: LPCWSTR,
    dwDesiredAccess: DWORD,
    dwShareMode: DWORD,
    lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
    dwCreationDisposition: DWORD,
    dwFlagsAndAttributes: DWORD,
    hTemplateFile: HANDLE,
) -> HANDLE {
    let mut conn = get_conn();
    let conn = conn.as_mut().unwrap();

    let file_name = {
        if lpFileName != ptr::null_mut() {
            let file_name_len = unsafe { cwstrlen(lpFileName) };
            let name_slice = unsafe { slice::from_raw_parts(lpFileName, file_name_len) };
            let name_os_string = OsString::from_wide(name_slice);
            Some(name_os_string.to_string_lossy().into_owned())
        } else {
            None
        }
    };

    log_info!(
        conn,
        "CreateFileW(lpFileName: {})",
        file_name.unwrap_or_else(|| "[NULL POINTER]".into())
    )
    .ok();

    unsafe {
        CreateFileWHook.call(
            lpFileName,
            dwDesiredAccess,
            dwShareMode,
            lpSecurityAttributes,
            dwCreationDisposition,
            dwFlagsAndAttributes,
            hTemplateFile,
        )
    }
}
