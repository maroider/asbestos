use std::{error::Error, fmt};

pub mod file;

#[derive(Debug)]
enum HookError {
    SymbolAddressNotFound {
        module: &'static str,
        symbol: &'static str,
    },
}

impl fmt::Display for HookError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SymbolAddressNotFound { module, symbol } => write!(
                f,
                r#"Could not find address of symbol "{}" in "{}""#,
                module, symbol
            ),
        }
    }
}

impl Error for HookError {}
