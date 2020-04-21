use std::{
    env,
    io::BufReader,
    os::windows::process::CommandExt,
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

use structopt::StructOpt;

use asbestos::shared::{
    named_pipe::PipeOptions,
    named_pipe_name,
    protocol::{Connection, Message, ProtocolError, StartupInfo},
    PipeEnd,
};

static CTRL_C: AtomicBool = AtomicBool::new(false);

const CREATE_SUSPENDED: u32 = 0x00000004;
const CREATE_NEW_CONSOLE: u32 = 0x00000010;

fn main() {
    let opts = Opts::from_args();

    ctrlc::set_handler(move || CTRL_C.store(true, Ordering::SeqCst))
        .expect("Error setting Ctrl-C handler");

    match opts.cmd {
        Cmd::Inject(opts) => inject(opts),
        Cmd::Wrap(opts) => wrap(opts),
    }
}

fn inject(opts: Inject) {
    inject_impl(opts.pid, false);
}

fn wrap(opts: Wrap) {
    let process = Command::new(&opts.command)
        .args(opts.args)
        .creation_flags(
            CREATE_SUSPENDED
                | if opts.create_console {
                    CREATE_NEW_CONSOLE
                } else {
                    0
                },
        )
        .spawn()
        .unwrap();
    let process_id = process.id();
    inject_impl(process_id, true);
}

#[derive(StructOpt)]
struct Opts {
    #[structopt(subcommand)]
    cmd: Cmd,
}

#[derive(StructOpt)]
enum Cmd {
    Inject(Inject),
    Wrap(Wrap),
}

/// Inject the payload into the specified process.
#[derive(StructOpt)]
struct Inject {
    pid: u32,
}

/// Create a process with <command> and [args]  and the `CREATE_SUSPENDED` flag.
///
/// The process will be allowed to begin execution once the payload has been initalized.
#[derive(StructOpt)]
struct Wrap {
    command: String,
    args: Vec<String>,
    /// Create a console for the wrapped process
    #[structopt(long)]
    create_console: bool,
}

fn inject_impl(pid: u32, target_suspended: bool) {
    let connecting_server_rx = PipeOptions::new(named_pipe_name(pid, PipeEnd::Rx))
        .single()
        .expect("Could not open named pipe");

    let connecting_server_tx = PipeOptions::new(named_pipe_name(pid, PipeEnd::Tx))
        .single()
        .expect("Could not open named pipe");

    let current_exe = env::current_exe().expect("asbestos could not locate its own executable");
    let mut dll = current_exe.clone();
    dll.set_file_name("asbestos_payload");
    dll.set_extension("dll");

    let injection_thread = thread::spawn(move || {
        syringe::inject_dll(pid, &dll).unwrap();
    });

    eprintln!("Waiting for connection from {}", pid);
    let pipe_rx = match connecting_server_rx.wait_ms(3000) {
        Err(err) => {
            eprintln!("Platform IO error: {}", err);
            return;
        }
        Ok(ok) => match ok {
            Err(_) => {
                eprintln!("{} did not connect within 3 seconds", pid);
                return;
            }
            Ok(ok) => ok,
        },
    };
    let pipe_tx = match connecting_server_tx.wait_ms(3000) {
        Err(err) => {
            eprintln!("Platform IO error: {}", err);
            return;
        }
        Ok(ok) => match ok {
            Err(_) => {
                eprintln!("{} did not connect within 3 seconds", pid);
                return;
            }
            Ok(ok) => ok,
        },
    };
    let mut connection = Connection::new(BufReader::new(pipe_rx), pipe_tx);
    connection
        .write_message(Message::StartupInfo(StartupInfo {
            main_thread_suspended: target_suspended,
        }))
        .unwrap();
    injection_thread.join().unwrap();
    loop {
        match connection.read_message() {
            Ok(msg) => match msg {
                Message::StartupInfo(_) => {}
                Message::LogMessage(log_message) => {
                    eprintln!(
                        "{}: [{}:{}] {}",
                        pid,
                        log_message
                            .module_path
                            .trim_start_matches("asbestos_payload::"),
                        log_message.line,
                        log_message.message
                    );
                }
                Message::Initialized => eprintln!("{}: Payload initialized", pid),
                Message::InitializationFailed(err) => {
                    eprintln!("{}: Payload initalization failed: {}", pid, err)
                }
                Message::ProcessDetach => {
                    eprintln!("{}: Payload unloaded", pid);
                    return;
                }
            },
            Err(err) => {
                if matches!(err, ProtocolError::Disconnected) {
                    return;
                }
                eprintln!("{}: {:?} => {}", pid, err, err)
            }
        }
        if CTRL_C.load(Ordering::SeqCst) {
            eprintln!("Ctrl-C");
            return;
        }
    }
}
