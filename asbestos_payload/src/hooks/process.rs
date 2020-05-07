//! Hooks for `CreateProcess[A|W]`.
//!
//! `CreateProcessAsUser`, `CreateProcessWithLogon` and `CreateProcessWithToken` are currently not
//! hooked since they're probably not used much in games.

use std::{ffi::OsStr, iter, mem, os::windows::ffi::OsStrExt};

use widestring::U16CStr;
use winapi::{
    shared::{
        minwindef::{BOOL, DWORD, LPVOID},
        ntdef::{HANDLE, PHANDLE},
    },
    um::{
        errhandlingapi::GetLastError,
        minwinbase::LPSECURITY_ATTRIBUTES,
        processthreadsapi::{LPPROCESS_INFORMATION, LPSTARTUPINFOW},
        winbase::CREATE_SUSPENDED,
        winnt::{LPCWSTR, LPWSTR},
    },
};

use asbestos_shared::{
    log_error,
    protocol::{Message, ProcessSpawned},
};

use crate::{get_conn, vfs};

use super::decl_detour;

// This is called by both `CreateProcessA` and `CreateProcessW`
decl_detour!(
    "KernelBase.dll",
    createprocessinternalw,
    BOOL CreateProcessInternalW(
        HANDLE                hUserToken,
        LPCWSTR               lpApplicationName,
        LPWSTR                lpCommandLine,
        LPSECURITY_ATTRIBUTES lpProcessAttributes,
        LPSECURITY_ATTRIBUTES lpThreadAttributes,
        BOOL                  bInheritHandles,
        DWORD                 dwCreationFlags,
        LPVOID                lpEnvironment,
        LPCWSTR               lpCurrentDirectory,
        LPSTARTUPINFOW        lpStartupInfo,
        LPPROCESS_INFORMATION lpProcessInformation,
        PHANDLE               hNewToken
    )  {
        let mut result = None;

        if !lpApplicationName.is_null() {
            let mut conn_lock = get_conn();
            let conn = conn_lock.as_mut().unwrap();

            let os_file_name = unsafe { U16CStr::from_ptr_str(lpApplicationName) }.to_os_string();
            let utf8_file_name = os_file_name.to_string_lossy();

            log_info!(conn, "CreateProcessInternalW(lpApplicationName: {})", utf8_file_name).ok();

            match vfs::resolve_path(Some(conn), os_file_name.as_ref()) {
                Err(err) => {
                    log_error!(conn, "Error while redirecting from {}: {}", utf8_file_name, err).ok();
                }
                Ok(redirected_object_name) => {
                    if redirected_object_name != os_file_name {
                        log_info!(conn, r#"Redirected "{}" to "{}""#, utf8_file_name, redirected_object_name.display()).ok();
                        let redirected_object_name: &OsStr = redirected_object_name.as_ref().as_ref();
                        let redirected_object_name: Vec<_> = redirected_object_name.encode_wide().chain(iter::once(0)).collect();

                        mem::drop(conn_lock);

                        let res = unsafe {
                            Hook.call(
                                hUserToken,
                                redirected_object_name.as_ptr(),
                                lpCommandLine,
                                lpProcessAttributes,
                                lpThreadAttributes,
                                bInheritHandles,
                                dwCreationFlags | CREATE_SUSPENDED,
                                lpEnvironment,
                                lpCurrentDirectory,
                                lpStartupInfo,
                                lpProcessInformation,
                                hNewToken,
                            )
                        };

                        result = Some(res);
                    }
                }
            }
        }

        if result.is_none() {
            let res = unsafe {
                Hook.call(
                    hUserToken,
                    lpApplicationName,
                    lpCommandLine,
                    lpProcessAttributes,
                    lpThreadAttributes,
                    bInheritHandles,
                    dwCreationFlags | CREATE_SUSPENDED,
                    lpEnvironment,
                    lpCurrentDirectory,
                    lpStartupInfo,
                    lpProcessInformation,
                    hNewToken,
                )
            };

            result = Some(res);
        }

        let mut conn = get_conn();
        let conn = conn.as_mut().unwrap();

        let res = result.unwrap();

        if res == 0 {
            log_error!(conn, "Error in CreateProcessInternalW: 0x{:x}", unsafe {
                GetLastError()
            })
            .ok();
            return res;
        }

        conn.write_message(Message::ProcessSpawned(ProcessSpawned {
            // TODO: Figure out what to do if `lpProcessInformation` is null.
            pid: unsafe { *lpProcessInformation }.dwProcessId,
            tid: unsafe { *lpProcessInformation }.dwThreadId,
        }))
        .ok();

        res
    }
);
