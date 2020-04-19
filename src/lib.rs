// #![recursion_limit = "1024"]

use std::{error::Error, io::Write, process};

use named_pipe::PipeClient;
use winapi::{
    shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID, TRUE},
    um::winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
};

mod hooks;
mod util;

#[macro_export]
macro_rules! c_str {
    ($s:literal) => {
        concat!($s, "\0").as_bytes().as_ptr() as *const i8
    };
}

fn injected_main() -> Result<(), Box<dyn Error>> {
    let mut pipe = PipeClient::connect_ms(util::named_pipe_name(process::id()), 500)?;
    unsafe {
        hooks::file::openfile_hook(&mut pipe)?;
        hooks::file::createfilea_hook(&mut pipe)?;
        hooks::file::createfilew_hook(&mut pipe)?;
    }
    Ok(())
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn DllMain(
    _module: HINSTANCE,
    call_reason: DWORD,
    _reserved: LPVOID,
) -> BOOL {
    if call_reason == DLL_PROCESS_ATTACH {
        // A console may be useful for printing to 'stdout'
        // winapi::um::consoleapi::AllocConsole();

        // Preferably a thread should be created here instead, since as few
        // operations as possible should be performed within `DllMain`.
        injected_main().is_ok() as BOOL
    } else if call_reason == DLL_PROCESS_DETACH {
        let f: fn() -> Result<(), Box<dyn Error>> = || {
            let mut pipe = PipeClient::connect_ms(util::named_pipe_name(process::id()), 500)?;
            writeln!(&mut pipe, "PROCESS_DETACH")?;
            Ok(())
        };
        f().is_ok() as BOOL
    } else {
        TRUE
    }
}
