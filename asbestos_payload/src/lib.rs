use std::{
    env,
    error::Error,
    io::BufReader,
    panic, process,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex, MutexGuard, TryLockError,
    },
    thread,
    time::Duration,
};

use lazy_static::lazy_static;
use winapi::{
    shared::minwindef::{BOOL, DWORD, FALSE, HINSTANCE, LPVOID, TRUE},
    um::{
        consoleapi::AllocConsole,
        handleapi::CloseHandle,
        processthreadsapi::{GetCurrentThreadId, OpenThread, ResumeThread},
        wincon::GetConsoleWindow,
        winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, THREAD_SUSPEND_RESUME},
        winuser::{ShowWindow, SW_HIDE, SW_SHOW},
    },
};

use asbestos_shared::{
    named_pipe::PipeClient,
    named_pipe_name,
    protocol::{Connection, Mappings, Message},
    PipeEnd,
};

mod hooks;
mod missing_from_winapi;
mod util;
pub mod vfs;

#[macro_export]
macro_rules! c_str {
    ($s:literal) => {
        concat!($s, "\0").as_bytes().as_ptr() as *const i8
    };
}

type PipeConnection = Connection<BufReader<PipeClient>, PipeClient>;

lazy_static! {
    static ref CONN: Mutex<Option<PipeConnection>> = Mutex::new(None);
    static ref MAPPINGS: Mutex<Mappings> = Mutex::new(Mappings::default());
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn DllMain(
    _module: HINSTANCE,
    call_reason: DWORD,
    _reserved: LPVOID,
) -> BOOL {
    if call_reason == DLL_PROCESS_ATTACH {
        install_panic_hook();

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

    unsafe { AllocConsole() };
    let handle = unsafe { GetConsoleWindow() };
    if !handle.is_null() {
        if startup_info.show_console {
            unsafe { ShowWindow(handle, SW_SHOW) };
        } else {
            unsafe { ShowWindow(handle, SW_HIDE) };
        }
    }

    unsafe {
        hooks::ntdll::ntcreatefile::hook(&mut conn)?;
    }

    if !startup_info.dont_hook_subprocesses {
        unsafe {
            hooks::process::createprocessa::hook(&mut conn)?;
            hooks::process::createprocessw::hook(&mut conn)?;
        }
    }

    *MAPPINGS.lock().unwrap() = startup_info.mappings;
    *CONN.lock().unwrap() = Some(conn);

    if startup_info.main_thread_suspended {
        resume_main_thread();
    }

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

fn install_panic_hook() {
    let default_panic_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        static FIRST_PANIC: AtomicBool = AtomicBool::new(true);
        if FIRST_PANIC.swap(false, Ordering::SeqCst) {
            env::set_var("RUST_BACKTRACE", "full");
            if let Ok(_) = ensure_console_window() {
                default_panic_hook(panic_info);
                loop {
                    thread::sleep(Duration::from_millis(100));
                }
            }
        } else {
            default_panic_hook(panic_info);
            loop {
                thread::sleep(Duration::from_millis(100));
            }
        }
    }));
}

fn ensure_console_window() -> Result<(), ()> {
    let handle = {
        let handle = unsafe { GetConsoleWindow() };
        if handle.is_null() {
            if unsafe { AllocConsole() } == 0 {
                return Err(());
            } else {
                let handle = unsafe { GetConsoleWindow() };
                if handle.is_null() {
                    return Err(());
                }
                handle
            }
        } else {
            handle
        }
    };

    unsafe { ShowWindow(handle, SW_SHOW) };

    Ok(())
}
