//! Hooks for functions that deal with the file system.

// The following is a list of functions that may need to be hooked:
//
// [ ] CreateDirectoryA                            priority: high
// [ ] CreateDirectoryW                            priority: high
// [ ] CreateFile2 (W?) (A?)                       priority:
// [ ] CreateFileA                                 priority: high
// [ ] CreateFileW                                 priority: high
// [ ] ? DefineDosDeviceW (A?)                     priority:
// [ ] DeleteFileA                                 priority: high
// [ ] DeleteFileW                                 priority: high
// [ ] ? DeleteVolumeMountPointW (A?)              priority:
// [ ] ? FindClose                                 priority: high
// [ ] ? FindCloseChangeNotification               priority:
// [ ] FindFirstChangeNotificationA                priority:
// [ ] FindFirstChangeNotificationW                priority:
// [ ] FindFirstFileA                              priority: high
// [ ] FindFirstFileW                              priority: high
// [ ] FindFirstFileExA                            priority: high
// [ ] FindFirstFileExW                            priority: high
// [ ] FindFirstFileNameW (A?)                     priority: high
// [ ] ? FindFirstStreamW (A?)                     priority:
// [ ] ? FindFirstVolumeW (A?)                     priority:
// [ ] FindNextChangeNotification (W?) (A?)        priority:
// [ ] FindNextFileA                               priority: high
// [ ] FindNextFileW                               priority: high
// [ ] FindNextFileNameW (A?)                      priority: high
// [ ] ? FindNextStreamW (A?)                      priority:
// [ ] ? FindNextVolumeW (A?)                      priority:
// [ ] ? FindVolumeClose                           priority:
// [ ] GetCompressedFileSizeA                      priority:
// [ ] GetCompressedFileSizeW                      priority:
// [ ] ? GetDiskFreeSpaceA                         priority:
// [ ] ? GetDiskFreeSpaceW                         priority:
// [ ] ? GetDiskFreeSpaceExA                       priority:
// [ ] ? GetDiskFreeSpaceExW                       priority:
// [ ] ? GetDriveTypeA                             priority:
// [ ] ? GetDriveTypeW                             priority:
// [ ] GetFileAttributesA                          priority:
// [ ] GetFileAttributesExA                        priority:
// [ ] GetFileAttributesW                          priority:
// [ ] GetFileAttributesExW                        priority:
// [ ] ? GetFileInformationByHandle                priority:
// [ ] GetFileSize                                 priority:
// [ ] ? GetFinalPathNameByHandleA                 priority:
// [ ] ? GetFinalPathNameByHandleW                 priority:
// [ ] GetFullPathNameA                            priority: medium
// [ ] GetFullPathNameW                            priority: medium
// [ ] GetLongPathNameA                            priority: medium
// [ ] GetLongPathNameW                            priority: medium
// [ ] GetShortPathNameW (A?)                      priority: medium
// [ ] ? GetTempFileNameA                          priority:
// [ ] ? GetTempFileNameW                          priority:
// [ ] ? GetTempPathA                              priority:
// [ ] ? GetTempPathW                              priority:
// [ ] ? GetVolumeInformationA                     priority:
// [ ] ? GetVolumeInformationW                     priority:
// [ ] ? GetVolumeInformationByHandleW (A?)        priority:
// [ ] ? GetVolumeNameForVolumeMountPointW (A?)    priority:
// [ ] ? GetVolumePathnamesForVolumeNameW (A?)     priority:
// [ ] ? GetVolumePathNameW (A?)                   priority:
// [ ] ? QueryDosDeviceW (A?)                      priority:
// [ ] ? ReadFile                                  priority:
// [ ] ? ReadFileEx                                priority:
// [ ] ? ReadFileScatter                           priority:
// [ ] RemoveDirectoryA                            priority: medium
// [ ] RemoveDirectoryW                            priority: medium
// [ ] SetFileAttributesA                          priority:
// [ ] SetFileAttributesW                          priority:
// [ ] ? SetFileInformationByHandle                priority:
// [ ] ? WriteFile                                 priority:
// [ ] ? WriteFileEx                               priority:
// [ ] ? WriteFileGather                           priority:

use std::{
    ffi::{OsStr, OsString},
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::{Component, Path},
    ptr, slice,
};

use detour::static_detour;
use widestring::U16CStr;
use winapi::{
    shared::minwindef::{DWORD, HFILE, UINT},
    um::{
        minwinbase::LPSECURITY_ATTRIBUTES,
        winbase::LPOFSTRUCT,
        winnt::{HANDLE, LPCSTR, LPCWSTR},
    },
};

use asbestos_shared::{log_info, log_trace, log_warn};

use crate::{get_conn, util::cstrlen, vfs};

use super::decl_hook_init;

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

    if !lpFileName.is_null() {
        let file_name = unsafe { U16CStr::from_ptr_str(lpFileName) };
        let file_name = OsString::from_wide(file_name.as_slice());
    let file_name = {
            let file_name: &Path = file_name.as_ref();
            // TODO: Implement a purely text-based path canonicalization function
            if file_name.components().any(|comp| comp == Component::CurDir) {
                log_warn!(
                    conn,
                    r#"lpFileName: "{}" is relative to the current directory (".")"#,
                    file_name.display()
                )
                .ok();
            } else if file_name
                .components()
                .any(|comp| comp == Component::ParentDir)
            {
                log_warn!(
                    conn,
                    r#"lpFileName: "{}" contains a parent directory component ("..")"#,
                    file_name.display()
                )
                .ok();
        }
            file_name
    };
        let mapped_file_name = vfs::resolve_path(file_name).unwrap();
        if file_name != mapped_file_name {
            let mapped_file_name: &Path = mapped_file_name.as_ref();

    log_info!(
        conn,
                r#"CreateFileW(lpFileName: "{}") mapped to "{}""#,
                file_name.display(),
                mapped_file_name.display()
            )
            .ok();

            let mapped_file_name: &OsStr = mapped_file_name.as_ref();
            let mut mapped_file_name: Vec<u16> = mapped_file_name.encode_wide().collect();
            if mapped_file_name.last() == Some(&0) {
                mapped_file_name.push(0);
            }

            unsafe {
                CreateFileWHook.call(
                    mapped_file_name.as_ptr(),
                    dwDesiredAccess,
                    dwShareMode,
                    lpSecurityAttributes,
                    dwCreationDisposition,
                    dwFlagsAndAttributes,
                    hTemplateFile,
                )
            }
        } else {
            log_info!(
                conn,
                r#"CreateFileW(lpFileName: "{}")"#,
                file_name.to_string_lossy()
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
    } else {
        log_info!(conn, "CreateFileW(lpFileName: [NULL POINTER])").ok();

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
}
