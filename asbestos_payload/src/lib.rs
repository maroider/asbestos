use std::{
    error::Error,
    io::BufReader,
    process,
    sync::{Mutex, MutexGuard, TryLockError},
};

use lazy_static::lazy_static;
use winapi::{
    shared::minwindef::{BOOL, DWORD, FALSE, HINSTANCE, LPVOID, TRUE},
    um::{
        handleapi::CloseHandle,
        processthreadsapi::{GetCurrentThreadId, OpenThread, ResumeThread},
        winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, THREAD_SUSPEND_RESUME},
    },
};

use asbestos_shared::{
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
    static ref CONN: Mutex<Option<PipeConnection>> = Mutex::new(None);
}

fn get_conn() -> MutexGuard<'static, Option<PipeConnection>> {
    loop {
        let res = CONN.try_lock();
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
    let startup_info = match dbg!(conn.read_message()?) {
        Message::StartupInfo(si) => si,
        _ => Default::default(),
    };
    unsafe {
        hooks::file::openfile_hook(&mut conn)?;
        hooks::file::createfilea_hook(&mut conn)?;
        hooks::file::createfilew_hook(&mut conn)?;
    }
    if !startup_info.dont_hook_subprocesses {
        unsafe {
            hooks::process::createprocessa_hook(&mut conn)?;
            hooks::process::createprocessw_hook(&mut conn)?;
        }
    }

    if startup_info.main_thread_suspended {
        resume_main_thread();
    }

    *CONN.lock().unwrap() = Some(conn);
    Ok(())
}

fn resume_main_thread() {
    let pid = process::id();
    let current_thread = unsafe { GetCurrentThreadId() };
    for entry in tlhelp32::Snapshot::new_thread().unwrap() {
        if entry.owner_process_id == pid && entry.thread_id != current_thread {
            let handle = unsafe { OpenThread(THREAD_SUSPEND_RESUME, FALSE, entry.thread_id) };
            if handle.is_null() {
                return;
            }
            unsafe { ResumeThread(handle) };
            unsafe { CloseHandle(handle) };
        }
    }
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
                let mut conn = CONN.lock().unwrap();
                let conn = conn.as_mut().unwrap();
                conn.write_message(Message::Initialized).ok();
                TRUE
            }
            Err(err) => {
                let mut conn = CONN.lock().unwrap().take().unwrap();
                conn.write_message(Message::InitializationFailed(err.to_string()))
                    .ok();
                FALSE
            }
        }
    } else if call_reason == DLL_PROCESS_DETACH {
        let f: fn() -> Result<(), Box<dyn Error>> = || {
            if let Some(mut connection) = CONN.lock()?.take() {
                connection.write_message(Message::ProcessDetach).ok();
            }
            Ok(())
        };
        f().is_ok() as BOOL
    } else {
        TRUE
    }
}
