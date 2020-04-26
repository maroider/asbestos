use std::{
    collections::HashMap,
    env,
    fs::File,
    io::BufReader,
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};

use structopt::StructOpt;

use asbestos::shared::{
    named_pipe::{ConnectingServer, PipeOptions, PipeServer},
    named_pipe_name,
    protocol::{Connection, Mappings, Message, ProtocolError, StartupInfo},
    PipeEnd,
};

static CTRL_C: AtomicBool = AtomicBool::new(false);

const CREATE_SUSPENDED: u32 = 0x00000004;
const DETACHED_PROCESS: u32 = 0x00000008;

fn main() {
    let opts = dbg!(Opts::from_args());

    ctrlc::set_handler(move || CTRL_C.store(true, Ordering::SeqCst))
        .expect("Error setting Ctrl-C handler");

    match opts.cmd {
        Cmd::Inject(opts) => inject(opts),
        Cmd::Wrap(opts) => wrap(opts),
    }
}

fn inject(opts: Inject) {
    let mappings = match load_mappings(&opts.common.mappings) {
        Ok(ok) => ok,
        Err(_) => loop {},
    };
    inject_impl(
        opts.pid,
        StartupInfo {
            main_thread_suspended: false,
            dont_hook_subprocesses: opts.common.no_sub_hook,
            show_console: true,
            mappings,
        },
    );
}

fn wrap(opts: Wrap) {
    let mappings = match load_mappings(&opts.common.mappings) {
        Ok(ok) => ok,
        Err(_) => loop {},
    };
    let process = Command::new(&opts.command)
        .args(opts.args)
        .creation_flags(CREATE_SUSPENDED | DETACHED_PROCESS)
        .spawn()
        .unwrap();
    let process_id = process.id();
    inject_impl(
        process_id,
        StartupInfo {
            main_thread_suspended: true,
            dont_hook_subprocesses: opts.common.no_sub_hook,
            show_console: opts.show_console,
            mappings,
        },
    );
}

fn load_mappings(path: &Path) -> Result<Mappings, ()> {
    let file = match File::open(path) {
        Ok(ok) => ok,
        Err(err) => {
            eprintln!("Could not open {}: {}", path.display(), err);
            return Err(());
        }
    };
    let reader = BufReader::new(file);
    dbg!(serde_json::from_reader(reader).map_err(|err| {
        eprintln!("Could not deserialize mappings file: {}", err);
    }))
}

#[derive(Debug, StructOpt)]
struct Opts {
    #[structopt(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, StructOpt)]
enum Cmd {
    Inject(Inject),
    Wrap(Wrap),
}

/// Inject the payload into the specified process.
#[derive(Debug, StructOpt)]
struct Inject {
    pid: u32,
    #[structopt(flatten)]
    common: CommonOpts,
}

/// Create a process with <command> and [args]  and the `CREATE_SUSPENDED` flag.
///
/// The process will be allowed to begin execution once the payload has been initalized.
#[derive(Debug, StructOpt)]
struct Wrap {
    command: String,
    args: Vec<String>,
    /// Immediately show the console of the wrapped process
    #[structopt(long)]
    show_console: bool,
    #[structopt(flatten)]
    common: CommonOpts,
    /// Ignore all arguments beyond this flag
    #[allow(dead_code)]
    #[structopt(long)]
    asbestos_cli_ignore: Vec<String>,
}

#[derive(Debug, StructOpt)]
struct CommonOpts {
    /// Don't hook subprocesses created by the hooked process
    #[structopt(long)]
    no_sub_hook: bool,
    #[structopt(long = "with-mappings")]
    mappings: PathBuf,
}

fn inject_impl(pid: u32, startup_info: StartupInfo) {
    let mut connections = HashMap::new();
    if let Ok(connection) = inject_and_connect(pid, &startup_info) {
        connections.insert(pid, connection);
    }

    loop {
        let mut morgue = Vec::new();
        let mut nursery = Vec::new();
        for (pid, connection) in connections.iter_mut() {
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
                    Message::ProcessSpawned(ps) => {
                        eprintln!("{}: Spawned a new process: {}", pid, ps.pid);
                        if let Ok(new_connection) = inject_and_connect(ps.pid, &startup_info) {
                            nursery.push((ps.pid, new_connection));
                        }
                    }
                    Message::ProcessDetach => {
                        eprintln!("{}: Payload unloaded", pid);
                        morgue.push(*pid);
                    }
                },
                Err(err) => {
                    if matches!(err, ProtocolError::Disconnected) {
                        morgue.push(*pid);
                    }
                    eprintln!("{}: {:?} => {}", pid, err, err)
                }
            }
        }
        for dead_process in morgue {
            connections.remove(&dead_process);
        }
        for (pid, connection) in nursery {
            connections.insert(pid, connection);
        }

        if CTRL_C.load(Ordering::SeqCst) {
            eprintln!("Ctrl-C");
            break;
        }

        if connections.is_empty() {
            thread::sleep(Duration::from_millis(100));
        }
    }
}

fn inject_and_connect(
    pid: u32,
    startup_info: &StartupInfo,
) -> Result<Connection<impl std::io::Read, impl std::io::Write>, ()> {
    let current_exe = env::current_exe().expect("asbestos could not locate its own executable");
    let mut dll = current_exe.clone();
    dll.set_file_name("asbestos_payload");
    dll.set_extension("dll");

    let (connecting_server_rx, connecting_server_tx) = create_connecting_pipe_server_pair(pid);

    let injection_thread = thread::spawn(move || {
        syringe::inject_dll(pid, &dll).unwrap();
    });

    let (pipe_rx, pipe_tx) = match wait_for_pipe_connection_with_timeout_ms(
        pid,
        connecting_server_rx,
        connecting_server_tx,
        3000,
    ) {
        Ok(ok) => ok,
        Err(_) => return Err(()),
    };
    let mut connection = Connection::new(BufReader::new(pipe_rx), pipe_tx);
    connection
        .write_message(Message::StartupInfo(startup_info.clone()))
        .unwrap();
    injection_thread.join().unwrap();

    Ok(connection)
}

fn create_connecting_pipe_server_pair(pid: u32) -> (ConnectingServer, ConnectingServer) {
    let connecting_server_rx = PipeOptions::new(named_pipe_name(pid, PipeEnd::Rx))
        .single()
        .expect("Could not open named pipe");

    let connecting_server_tx = PipeOptions::new(named_pipe_name(pid, PipeEnd::Tx))
        .single()
        .expect("Could not open named pipe");

    (connecting_server_rx, connecting_server_tx)
}

fn wait_for_pipe_connection_with_timeout_ms(
    pid: u32,
    connecting_server_rx: ConnectingServer,
    connecting_server_tx: ConnectingServer,
    timeout: u32,
) -> Result<(PipeServer, PipeServer), ()> {
    eprintln!("Waiting for connection from {}", pid);
    let pipe_rx = match connecting_server_rx.wait_ms(timeout) {
        Err(err) => {
            eprintln!("Platform IO error: {}", err);
            return Err(());
        }
        Ok(ok) => match ok {
            Err(_) => {
                eprintln!("{} did not connect within {} ms", pid, timeout);
                return Err(());
            }
            Ok(ok) => ok,
        },
    };
    let pipe_tx = match connecting_server_tx.wait_ms(timeout) {
        Err(err) => {
            eprintln!("Platform IO error: {}", err);
            return Err(());
        }
        Ok(ok) => match ok {
            Err(_) => {
                eprintln!("{} did not connect within {} ms", pid, timeout);
                return Err(());
            }
            Ok(ok) => ok,
        },
    };

    Ok((pipe_rx, pipe_tx))
}
