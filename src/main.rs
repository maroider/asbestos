use std::{
    env,
    io::{BufRead, BufReader},
    sync::atomic::{AtomicBool, Ordering},
};

use named_pipe::PipeOptions;
use structopt::StructOpt;

include!("shared.rs");

static CTRL_C: AtomicBool = AtomicBool::new(false);

fn main() {
    let opts = Opts::from_args();

    ctrlc::set_handler(move || CTRL_C.store(true, Ordering::SeqCst))
        .expect("Error setting Ctrl-C handler");

    let connecting_server = PipeOptions::new(named_pipe_name(opts.pid))
        .single()
        .expect("Could not open named pipe");

    let current_exe = env::current_exe().expect("asbestos could not locate its own executable");
    let mut dll = current_exe.clone();
    dll.set_extension("dll");

    syringe::inject_dll(opts.pid, &dll).unwrap();

    eprintln!("Waiting for connection from {}", opts.pid);
    let pipe = match connecting_server.wait_ms(5000) {
        Err(err) => {
            eprintln!("Platform IO error: {}", err);
            return;
        }
        Ok(ok) => match ok {
            Err(_) => {
                eprintln!("{} did not connect within 5 seconds", opts.pid);
                return;
            }
            Ok(ok) => ok,
        },
    };
    let mut pipe = BufReader::new(pipe);
    let mut buf = String::new();
    loop {
        buf.clear();
        if let Err(err) = pipe.read_line(&mut buf) {
            eprintln!("Could not read from pipe to {}: {}", opts.pid, err);
        }
        if buf.len() > 0 {
            eprintln!("{}: {}", opts.pid, buf.trim_end());
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
