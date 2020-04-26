use std::{
    borrow::Cow,
    error::Error,
    fmt,
    io::{self, Read, Write},
    path::PathBuf,
};

use bincode::{deserialize, deserialize_from, serialize, serialize_into};
use serde::{Deserialize, Serialize};

use crate::wrapper_enum;

pub struct Connection<R: Read, W: Write> {
    rx: R,
    tx: W,
    state: ConnectionState,
}

impl<R: Read, W: Write> Connection<R, W> {
    pub fn new(rx: R, tx: W) -> Self {
        Self {
            rx,
            tx,
            state: ConnectionState::Connected,
        }
    }

    pub fn connected(&self) -> bool {
        matches!(self.state, ConnectionState::Connected)
    }

    pub fn read_message(&mut self) -> Result<Message, ProtocolError> {
        if !self.connected() {
            return Err(ProtocolError::Disconnected);
        }

        let res: Result<Vec<u8>, _> = deserialize_from(&mut self.rx);
        let container = match res {
            Ok(value) => Ok(value),
            Err(err) => match *err {
                bincode::ErrorKind::Io(err) => match err.kind() {
                    io::ErrorKind::UnexpectedEof => {
                        self.state = ConnectionState::Disconnected;
                        Err(ProtocolError::ConnectionLost)
                    }
                    _ => Err(Box::new(bincode::ErrorKind::Io(err)).into()),
                },
                _ => Err(err.into()),
            },
        }?;
        let value = deserialize(&container)?;
        Ok(value)
    }

    pub fn write_message<T: Into<Message>>(&mut self, value: T) -> Result<(), ProtocolError> {
        if !self.connected() {
            return Err(ProtocolError::Disconnected);
        }

        let message: Message = value.into();
        let container = serialize(&message)?;
        serialize_into(&mut self.tx, &container)?;
        Ok(())
    }
}

enum ConnectionState {
    Connected,
    Disconnected,
}

wrapper_enum! {
    #[derive(Debug)]
    pub enum ProtocolError {
        Io(io::Error),
        Bincode(bincode::Error),
        /// The connection was terminated unexpectedly.
        ///
        /// This can happen for a number of reasons, one of them being that `dllMain` isn't called with `DLL_PROCESS_DETACH`.
        ConnectionLost,
        /// The connection is no longer active.
        Disconnected,
    }
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Io(err) => err.fmt(f),
            Self::Bincode(err) => err.fmt(f),
            Self::ConnectionLost => write!(f, "The connection was unexpectedly closed."),
            Self::Disconnected => write!(f, "The connection is closed."),
        }
    }
}

impl Error for ProtocolError {}

wrapper_enum! {
    #[derive(Debug, Deserialize, Serialize)]
    pub enum Message {
        StartupInfo(StartupInfo),
        LogMessage(LogMessage),
        /// The payload has finished its initalization routine.
        Initialized,
        /// The payload encountered an error in its initialization routine.
        InitializationFailed(String),
        ProcessSpawned(ProcessSpawned),
        /// The payload was unloaded from the target, either because it was manually unloaded, or because the process
        /// terminated.
        ProcessDetach,
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct StartupInfo {
    pub main_thread_suspended: bool,
    pub dont_hook_subprocesses: bool,
    pub show_console: bool,
    pub mappings: Mappings,
}

// TODO: Validate mappings. eg. `from` should always be a directory unless `kind` is `Redirect`, in which case `from`
//       and `to` should point to the same kind of file system resource.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Mappings {
    pub mappings: Vec<Mapping>,
}

impl Mappings {
    pub fn iter(&self) -> impl Iterator<Item = &Mapping> {
        self.mappings.iter()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Mapping {
    pub kind: MappingKind,
    pub from: PathBuf,
    pub to: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MappingKind {
    /// Redirect access to a file or folder to another file or folder.
    Redirect,
    /// Virtually add a file or folder to a folder.
    Mount,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogMessage {
    pub level: LogLevel,
    pub module_path: Cow<'static, str>,
    pub file: Cow<'static, str>,
    pub line: u32,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProcessSpawned {
    pub pid: u32,
}
