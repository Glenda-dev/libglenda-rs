use crate::arch::runtime::panic_break;
use crate::cap::CONSOLE_CAP;
use crate::cap::Console;
use core::fmt;
use core::fmt::Write;
use spin::Mutex;

pub const ANSI_RED: &str = "\x1b[31m";
pub const ANSI_RESET: &str = "\x1b[0m";

pub static GLOBAL_CONSOLE: Mutex<Console> = Mutex::new(Console::null());

pub fn init() {
    *GLOBAL_CONSOLE.lock() = CONSOLE_CAP;
}

/// Force unlock the console mutex.
/// This is unsafe and should only be used in panic handlers.
unsafe fn force_unlock() {
    unsafe { GLOBAL_CONSOLE.force_unlock() };
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    match GLOBAL_CONSOLE.lock().write_fmt(args) {
        Ok(_) => {}
        Err(_) => unsafe { panic_break() },
    }
}

#[doc(hidden)]
pub fn _print_unsynced(args: fmt::Arguments) {
    unsafe {
        force_unlock();
    }
    match GLOBAL_CONSOLE.lock().write_fmt(args) {
        Ok(_) => {}
        Err(_) => unsafe { panic_break() },
    }
}