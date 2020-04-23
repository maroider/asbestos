//! Hooks for `CreateProcess[A|W]`.
//!
//! `CreateProcessAsUser`, `CreateProcessWithLogon` and `CreateProcessWithToken` are currently not
//! hooked since they're probably not used much in games.

use detour::static_detour;
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
    log_error, log_info, log_trace,
    protocol::{Message, ProcessSpawned},
};

use crate::get_conn;

use super::decl_hook_init;

static_detour! {
    static CreateProcessAHook: unsafe extern "system" fn(
        LPCSTR,
        LPSTR,
        LPSECURITY_ATTRIBUTES,
        LPSECURITY_ATTRIBUTES,
        BOOL,
        DWORD,
        LPVOID,
        LPCSTR,
        LPSTARTUPINFOA,
        LPPROCESS_INFORMATION
    ) -> BOOL;
}

type FnCreateProcessA = unsafe extern "system" fn(
    LPCSTR,
    LPSTR,
    LPSECURITY_ATTRIBUTES,
    LPSECURITY_ATTRIBUTES,
    BOOL,
    DWORD,
    LPVOID,
    LPCSTR,
    LPSTARTUPINFOA,
    LPPROCESS_INFORMATION,
) -> BOOL;

decl_hook_init!(
    CreateProcessAHook,
    FnCreateProcessA,
    CreateProcessA,
    createprocessa_hook,
    "kernel32.dll",
    createprocessa_detour
);

#[allow(non_snake_case)]
pub fn createprocessa_detour(
    lpApplicationName: LPCSTR,
    lpCommandLine: LPSTR,
    lpProcessAttributes: LPSECURITY_ATTRIBUTES,
    lpThreadAttributes: LPSECURITY_ATTRIBUTES,
    bInheritHandles: BOOL,
    dwCreationFlags: DWORD,
    lpEnvironment: LPVOID,
    lpCurrentDirectory: LPCSTR,
    lpStartupInfo: LPSTARTUPINFOA,
    lpProcessInformation: LPPROCESS_INFORMATION,
) -> BOOL {
    let mut conn = get_conn();
    let conn = conn.as_mut().unwrap();

    let res = unsafe {
        CreateProcessAHook.call(
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

static_detour! {
    static CreateProcessWHook: unsafe extern "system" fn(
        LPCWSTR,
        LPWSTR,
        LPSECURITY_ATTRIBUTES,
        LPSECURITY_ATTRIBUTES,
        BOOL,
        DWORD,
        LPVOID,
        LPCWSTR,
        LPSTARTUPINFOW,
        LPPROCESS_INFORMATION
    ) -> BOOL;
}

type FnCreateProcessW = unsafe extern "system" fn(
    LPCWSTR,
    LPWSTR,
    LPSECURITY_ATTRIBUTES,
    LPSECURITY_ATTRIBUTES,
    BOOL,
    DWORD,
    LPVOID,
    LPCWSTR,
    LPSTARTUPINFOW,
    LPPROCESS_INFORMATION,
) -> BOOL;

decl_hook_init!(
    CreateProcessWHook,
    FnCreateProcessW,
    CreateProcessW,
    createprocessw_hook,
    "kernel32.dll",
    createprocessw_detour
);

#[allow(non_snake_case)]
pub fn createprocessw_detour(
    lpApplicationName: LPCWSTR,
    lpCommandLine: LPWSTR,
    lpProcessAttributes: LPSECURITY_ATTRIBUTES,
    lpThreadAttributes: LPSECURITY_ATTRIBUTES,
    bInheritHandles: BOOL,
    dwCreationFlags: DWORD,
    lpEnvironment: LPVOID,
    lpCurrentDirectory: LPCWSTR,
    lpStartupInfo: LPSTARTUPINFOW,
    lpProcessInformation: LPPROCESS_INFORMATION,
) -> BOOL {
    let mut conn = get_conn();
    let conn = conn.as_mut().unwrap();

    let res = unsafe {
        CreateProcessWHook.call(
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
