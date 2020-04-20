// #![recursion_limit = "1024"]

use std::{
    error::Error,
    io::BufReader,
    process,
    sync::{Mutex, MutexGuard, TryLockError},
};

use lazy_static::lazy_static;
use winapi::{
    shared::minwindef::{BOOL, DWORD, FALSE, HINSTANCE, LPVOID, TRUE},
    um::winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
};

use asbestos_shared::{
    log_error, log_info,
    named_pipe::PipeClient,
    named_pipe_name,
    protocol::{Connection, Message},
    PipeEnd,
};

mod hooks;
mod util;

#[macro_export]
macro_rules! c_str {
    ($s:literal) => {
        concat!($s, "\0").as_bytes().as_ptr() as *const i8
    };
}

type PipeConnection = Connection<BufReader<PipeClient>, PipeClient>;

lazy_static! {
    static ref PIPE: Mutex<Option<PipeConnection>> = Mutex::new(None);
}

fn get_pipe() -> MutexGuard<'static, Option<PipeConnection>> {
    loop {
        let res = PIPE.try_lock();
        match res {
            Ok(pipe) => return pipe,
            Err(err) => {
                if let TryLockError::Poisoned(_) = err {
                    unreachable!("This should never happen. Ever.")
                }
            }
        }
    }
}

fn init_payload() -> Result<(), Box<dyn Error>> {
    let mut conn = Connection::new(
        BufReader::new(PipeClient::connect_ms(
            named_pipe_name(process::id(), PipeEnd::Tx),
            500,
        )?),
        PipeClient::connect_ms(named_pipe_name(process::id(), PipeEnd::Rx), 500)?,
    );
    unsafe {
        hooks::file::openfile_hook(&mut conn)?;
        hooks::file::createfilea_hook(&mut conn)?;
        hooks::file::createfilew_hook(&mut conn)?;
    }
    *PIPE.lock().unwrap() = Some(conn);
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
        match init_payload() {
            Ok(_) => {
                let mut conn = PIPE.lock().unwrap();
                let conn = conn.as_mut().unwrap();
                log_info!(conn, "Payload initialized").ok();
                TRUE
            }
            Err(err) => {
                let mut pipe = PIPE.lock().unwrap().take().unwrap();
                log_error!(pipe, "Payload initialization failed: {}", err).ok();
                FALSE
            }
        }
    } else if call_reason == DLL_PROCESS_DETACH {
        let f: fn() -> Result<(), Box<dyn Error>> = || {
            if let Some(mut connection) = PIPE.lock()?.take() {
                connection.write_message(Message::ProcessDetach).ok();
            }
            Ok(())
        };
        f().is_ok() as BOOL
    } else {
        TRUE
    }
}
