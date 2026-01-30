#[cfg(feature = "kernel-console")]
mod kconsole;

#[cfg(feature = "kernel-console")]
pub use kconsole::*;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print_unsynced {
    ($($arg:tt)*) => ($crate::console::_print_unsynced(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println_unsynced {
    () => ($crate::print_unsynced!("\n"));
    ($($arg:tt)*) => ($crate::print_unsynced!("{}\n", format_args!($($arg)*)));
}
