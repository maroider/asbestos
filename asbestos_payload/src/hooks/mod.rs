use std::{error::Error, fmt};

pub mod file;
pub mod process;

macro_rules! _decl_detour {
    ($module:literal, $lowercase_name:ident, $ret:tt $name:ident ($($arg_type:tt $arg_name:ident),* $(,)?) $detour_body:tt ) => {
        pub mod $lowercase_name {
            #[allow(unused_imports)]
            use super::*;

            use detour::static_detour;

            use asbestos_shared::{log_info, log_trace};

                    static_detour! {
                        static Hook: unsafe extern "system" fn($($arg_type),*) -> $ret;
                }

                    pub unsafe fn hook<R: std::io::Read, W: std::io::Write>(
                    conn: &mut crate::Connection<R, W>,
                ) -> Result<(), Box<dyn std::error::Error>> {
                    log_trace!(
                    conn,
                    concat!("Locating ", stringify!($name), "'s address")
                )?;
                let address = crate::util::get_module_symbol_address($module, stringify!($name))
                    .ok_or(super::super::HookError::SymbolAddressNotFound {
                    module: $module,
                    symbol: stringify!($name),
                })?;
                let target: unsafe extern "system" fn($($arg_type),*) -> $ret = std::mem::transmute(address);

                log_trace!(
                    conn,
                    concat!("Initalizing ", stringify!($name), "'s hook")
                )?;
                    Hook.initialize(target, detour)?.enable()?;
                log_info!(
                    conn,
                    concat!(stringify!($name), "'s hook has been initialized")
                )?;
                Ok(())
            }

            #[allow(non_snake_case)]
                pub fn detour($($arg_name: $arg_type),*) -> $ret {
                $detour_body
            }
        }
    };
}

pub(crate) use _decl_detour as decl_detour;

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
