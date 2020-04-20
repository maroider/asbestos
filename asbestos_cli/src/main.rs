use std::{
    env,
    io::BufReader,
    sync::atomic::{AtomicBool, Ordering},
};

use structopt::StructOpt;

use asbestos::shared::{
    named_pipe::PipeOptions,
    named_pipe_name,
    protocol::{Connection, Message, ProtocolError},
    PipeEnd,
};

static CTRL_C: AtomicBool = AtomicBool::new(false);

fn main() {
    let opts = Opts::from_args();

    ctrlc::set_handler(move || CTRL_C.store(true, Ordering::SeqCst))
        .expect("Error setting Ctrl-C handler");

    let connecting_server_rx = PipeOptions::new(named_pipe_name(opts.pid, PipeEnd::Rx))
        .single()
        .expect("Could not open named pipe");

    let connecting_server_tx = PipeOptions::new(named_pipe_name(opts.pid, PipeEnd::Tx))
        .single()
        .expect("Could not open named pipe");

    let current_exe = env::current_exe().expect("asbestos could not locate its own executable");
    let mut dll = current_exe.clone();
    dll.set_file_name("asbestos_payload");
    dll.set_extension("dll");

    syringe::inject_dll(opts.pid, &dll).unwrap();

    eprintln!("Waiting for connection from {}", opts.pid);
    let pipe_rx = match connecting_server_rx.wait_ms(3000) {
        Err(err) => {
            eprintln!("Platform IO error: {}", err);
            return;
        }
        Ok(ok) => match ok {
            Err(_) => {
                eprintln!("{} did not connect within 3 seconds", opts.pid);
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
                eprintln!("{} did not connect within 3 seconds", opts.pid);
                return;
            }
            Ok(ok) => ok,
        },
    };
    let mut connection = Connection::new(BufReader::new(pipe_rx), pipe_tx);
    loop {
        match connection.read_message() {
            Ok(msg) => match msg {
                Message::LogMessage(log_message) => {
                    eprintln!(
                        "{}: [{}:{}] {}",
                        opts.pid,
                        log_message
                            .module_path
                            .trim_start_matches("asbestos_payload::"),
                        log_message.line,
                        log_message.message
                    );
                }
                Message::ProcessDetach => {
                    eprintln!("{}: Payload unloaded", opts.pid);
                    return;
                }
            },
            Err(err) => {
                if matches!(err, ProtocolError::Disconnected) {
                    return;
                }
                eprintln!("{}: {:?} => {}", opts.pid, err, err)
            }
        }
        if CTRL_C.load(Ordering::SeqCst) {
            eprintln!("Ctrl-C");
            return;
        }
    }
}

#[derive(StructOpt)]
struct Opts {
    pid: u32,
}
