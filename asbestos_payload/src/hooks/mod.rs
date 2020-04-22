use std::{error::Error, fmt};

pub mod file;
pub mod process;

#[doc(hidden)]
#[macro_export]
macro_rules! decl_hook_init {
    ($hook:ident, $hooked_fn_type:ty, $hooked_fn:ident, $init_fn:ident, $module:literal, $detour_fn:ident) => {
        pub unsafe fn $init_fn<R: std::io::Read, W: std::io::Write>(
            conn: &mut crate::Connection<R, W>,
        ) -> Result<(), Box<dyn std::error::Error>> {
            log_trace!(
                conn,
                concat!("Locating ", stringify!($hooked_fn), "'s address")
            )?;
            let address = crate::util::get_module_symbol_address($module, stringify!($hooked_fn))
                .ok_or(super::HookError::SymbolAddressNotFound {
                module: $module,
                symbol: stringify!($hooked_fn),
            })?;
            let target: $hooked_fn_type = std::mem::transmute(address);

            log_trace!(
                conn,
                concat!("Initalizing ", stringify!($hooked_fn), "'s hook")
            )?;
            $hook.initialize(target, $detour_fn)?.enable()?;
            log_info!(
                conn,
                concat!(stringify!($hooked_fn), "'s hook has been initialized")
            )?;
            Ok(())
        }
    };
}

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
