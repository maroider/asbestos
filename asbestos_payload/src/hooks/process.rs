//! Hooks for `CreateProcess[A|W]`.
//!
//! `CreateProcessAsUser`, `CreateProcessWithLogon` and `CreateProcessWithToken` are currently not
//! hooked since they're probably not used much in games.

use winapi::{
    shared::minwindef::{BOOL, DWORD, LPVOID},
    um::{
        errhandlingapi::GetLastError,
        minwinbase::LPSECURITY_ATTRIBUTES,
        processthreadsapi::{LPPROCESS_INFORMATION, LPSTARTUPINFOA, LPSTARTUPINFOW},
        winbase::CREATE_SUSPENDED,
        winnt::{LPCSTR, LPCWSTR, LPSTR, LPWSTR},
    },
};

use asbestos_shared::{
    log_error,
    protocol::{Message, ProcessSpawned},
};

use crate::get_conn;

use super::decl_detour;

decl_detour!(
    "kernel32",
    createprocessa,
    BOOL CreateProcessA(
        LPCSTR                lpApplicationName,
        LPSTR                 lpCommandLine,
        LPSECURITY_ATTRIBUTES lpProcessAttributes,
        LPSECURITY_ATTRIBUTES lpThreadAttributes,
        BOOL                  bInheritHandles,
        DWORD                 dwCreationFlags,
        LPVOID                lpEnvironment,
        LPCSTR                lpCurrentDirectory,
        LPSTARTUPINFOA        lpStartupInfo,
        LPPROCESS_INFORMATION lpProcessInformation
    ) {
        let mut conn = get_conn();
        let conn = conn.as_mut().unwrap();

        let res = unsafe {
            Hook.call(
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
            )
        };

        if res == 0 {
            log_error!(conn, "Error in CreateProcessA: 0x{:x}", unsafe {
                GetLastError()
            })
            .ok();
            return res;
        }

        conn.write_message(Message::ProcessSpawned(ProcessSpawned {
            // TODO: Figure out what to do if `lpProcessInformation` is null.
            pid: unsafe { *lpProcessInformation }.dwProcessId,
        }))
        .ok();

        res
    }
);

decl_detour!(
    "kernel32",
    createprocessw,
    BOOL CreateProcessW(
        LPCWSTR               lpApplicationName,
        LPWSTR                lpCommandLine,
        LPSECURITY_ATTRIBUTES lpProcessAttributes,
        LPSECURITY_ATTRIBUTES lpThreadAttributes,
        BOOL                  bInheritHandles,
        DWORD                 dwCreationFlags,
        LPVOID                lpEnvironment,
        LPCWSTR               lpCurrentDirectory,
        LPSTARTUPINFOW        lpStartupInfo,
        LPPROCESS_INFORMATION lpProcessInformation
    ) {
        let mut conn = get_conn();
        let conn = conn.as_mut().unwrap();

        let res = unsafe {
            Hook.call(
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
            )
        };

        if res == 0 {
            log_error!(conn, "Error in CreateProcessW: 0x{:x}", unsafe {
                GetLastError()
            })
            .ok();
            return res;
        }

        conn.write_message(Message::ProcessSpawned(ProcessSpawned {
            // TODO: Figure out what to do if `lpProcessInformation` is null.
            pid: unsafe { *lpProcessInformation }.dwProcessId,
        }))
        .ok();

        res
    }
);
