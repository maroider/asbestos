use std::{
    error::Error,
    io::{Read, Write},
    mem, ptr,
};

use detour::static_detour;
use winapi::{
    shared::minwindef::{DWORD, HFILE, UINT},
    um::{
        minwinbase::LPSECURITY_ATTRIBUTES,
        winbase::LPOFSTRUCT,
        winnt::{HANDLE, LPCSTR, LPCWSTR},
        winuser::MessageBoxA,
    },
};

use crate::{c_str, util::get_module_symbol_address};

static_detour! {
    static OpenFileHook: unsafe extern "system" fn(
        LPCSTR,
        LPOFSTRUCT,
        UINT
    ) -> HFILE;
}

type FnOpenFile = unsafe extern "system" fn(LPCSTR, LPOFSTRUCT, UINT) -> HFILE;

pub unsafe fn openfile_hook(mut pipe: impl Read + Write) -> Result<(), Box<dyn Error>> {
    writeln!(pipe, "Locating OpenFile's address")?;
    let address = get_module_symbol_address("kernel32.dll", "OpenFile").unwrap();
    let target: FnOpenFile = mem::transmute(address);

    writeln!(pipe, "Initializing OpenFile's hook")?;
    OpenFileHook.initialize(target, openfile_detour)?.enable()?;
    writeln!(pipe, "OpenFile's hook has been initialized")?;

    Ok(())
}

#[allow(non_snake_case)]
pub fn openfile_detour(lpFileName: LPCSTR, lpReOpenBuff: LPOFSTRUCT, uStyle: UINT) -> HFILE {
    unsafe {
        MessageBoxA(
            ptr::null_mut(),
            c_str!("OpenFile"),
            c_str!(r#"File was "opened""#),
            0,
        )
    };
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

pub unsafe fn createfilea_hook(mut pipe: impl Read + Write) -> Result<(), Box<dyn Error>> {
    writeln!(pipe, "Locating CreateFileA's address")?;
    let address = get_module_symbol_address("kernel32.dll", "CreateFileA").unwrap();
    let target: FnCreateFileA = mem::transmute(address);

    writeln!(pipe, "Initializing CreateFileA's hook")?;
    CreateFileAHook
        .initialize(target, createfilea_detour)?
        .enable()?;
    writeln!(pipe, "CreateFileA's hook has been initialized")?;

    Ok(())
}

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
    unsafe {
        MessageBoxA(
            ptr::null_mut(),
            c_str!("CreateFileA"),
            c_str!(r#"File was "opened""#),
            0,
        )
    };
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

pub unsafe fn createfilew_hook(mut pipe: impl Read + Write) -> Result<(), Box<dyn Error>> {
    writeln!(pipe, "Locating CreateFileW's address")?;
    let address = get_module_symbol_address("kernel32.dll", "CreateFileW").unwrap();
    let target: FnCreateFileW = mem::transmute(address);

    writeln!(pipe, "Initializing CreateFileW's hook")?;
    CreateFileWHook
        .initialize(target, createfilew_detour)?
        .enable()?;
    writeln!(pipe, "CreateFileW's hook has been initialized")?;

    Ok(())
}

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
    unsafe {
        MessageBoxA(
            ptr::null_mut(),
            c_str!("CreateFileW"),
            c_str!(r#"File was "opened""#),
            0,
        )
    };
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
