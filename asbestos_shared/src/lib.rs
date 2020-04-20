pub use named_pipe;

pub mod protocol;

mod protocol_macros;

pub fn named_pipe_name(pid: u32, end: PipeEnd) -> String {
    format!(
        r"\\.\pipe\{}-{}-{}",
        concat!(env!("CARGO_PKG_NAME"), "-", env!("CARGO_PKG_VERSION")),
        pid,
        match end {
            PipeEnd::Tx => "tx",
            PipeEnd::Rx => "rx",
        }
    )
}

#[derive(Debug)]
pub enum PipeEnd {
    Tx,
    Rx,
}
