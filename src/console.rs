use crate::cap::CONSOLE_CAP;
use crate::cap::Console;
use core::fmt;
use spin::Mutex;

pub const ANSI_RED: &str = "\x1b[31m";
pub const ANSI_RESET: &str = "\x1b[0m";

pub static GLOBAL_CONSOLE: Mutex<Console> = Mutex::new(Console::null());

pub fn init() {
    *GLOBAL_CONSOLE.lock() = CONSOLE_CAP;
}

/// Force unlock the console mutex.
/// This is unsafe and should only be used in panic handlers.
pub unsafe fn force_unlock() {
    unsafe { GLOBAL_CONSOLE.force_unlock() };
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    let _ = GLOBAL_CONSOLE.lock().write_fmt(args);
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
