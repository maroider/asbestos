pub fn named_pipe_name(pid: u32) -> String {
    format!(r"\\.\pipe\{}-{}", env!("CARGO_PKG_NAME"), pid)
}
