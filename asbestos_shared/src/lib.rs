pub use named_pipe;

pub fn named_pipe_name(pid: u32) -> String {
    format!(
        r"\\.\pipe\{}-{}",
        concat!(env!("CARGO_PKG_NAME"), "-", env!("CARGO_PKG_VERSION")),
        pid
    )
}
