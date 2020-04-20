#[macro_export]
macro_rules! log_message {
    ($level:expr, $target:ident, $($to_fmt:tt)*) => {
            $target.write_message(
            $crate::protocol::LogMessage {
                level: $level,
                module_path: module_path!().into(),
                file: file!().into(),
                line: line!(),
                message: format!($($to_fmt)*),
            },
        )
    };
}

#[macro_export]
macro_rules! log_error {
    ($($forward:tt)*) => {
        $crate::log_message!($crate::protocol::LogLevel::Error, $($forward)*)
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($forward:tt)*) => {
        $crate::log_message!($crate::protocol::LogLevel::Warn, $($forward)*)
    };
}

#[macro_export]
macro_rules! log_info {
    ($($forward:tt)*) => {
        $crate::log_message!($crate::protocol::LogLevel::Info, $($forward)*)
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($forward:tt)*) => {
        $crate::log_message!($crate::protocol::LogLevel::Debug, $($forward)*)
    };
}

#[macro_export]
macro_rules! log_trace {
    ($($forward:tt)*) => {
        $crate::log_message!($crate::protocol::LogLevel::Trace, $($forward)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! wrapper_enum {
    ($( #[$enum_attr:meta] )* $v:vis enum $name:ident { $( $( #[$member_attr:meta] )* $member:ident $( ( $embed:ty ) )? ),* $(,)? }) => {
        $( #[$enum_attr] )*
        $v enum $name {
            $( $( #[$member_attr] )* $member $( ( $embed ) )? ),*
        }

        $(
            $(
                impl From<$embed> for $name {
                    fn from(from: $embed) -> Self {
                        Self::$member(from)
                    }
                }
            )?
        )*
    };
}
