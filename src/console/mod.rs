#[cfg(feature = "kernel-console")]
mod kconsole;

#[cfg(feature = "kernel-console")]
pub use kconsole::*;

pub const ANSI_RESET: &str = "\x1b[0m";
pub const ANSI_RED: &str = "\x1b[31m";
pub const ANSI_GREEN: &str = "\x1b[32m";
pub const ANSI_YELLOW: &str = "\x1b[33m";
pub const ANSI_BLUE: &str = "\x1b[34m";
pub const ANSI_MAGENTA: &str = "\x1b[35m";
pub const ANSI_CYAN: &str = "\x1b[36m";
pub const ANSI_WHITE: &str = "\x1b[37m";

use crate::sync::once::Once;

pub static MODULE_NAME: Once<&'static str> = Once::new();

pub fn init_logging(name: &'static str) {
    MODULE_NAME.call_once(|| name);
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::println!("{}: {}", *$crate::console::MODULE_NAME.get().unwrap(), format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::println!("{}{}: {}{}", $crate::console::ANSI_YELLOW, *$crate::console::MODULE_NAME.get().unwrap(), format_args!($($arg)*), $crate::console::ANSI_RESET);
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::println!("{}{}: {}{}", $crate::console::ANSI_RED, *$crate::console::MODULE_NAME.get().unwrap(), format_args!($($arg)*), $crate::console::ANSI_RESET);
    };
}
