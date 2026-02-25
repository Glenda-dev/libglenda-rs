use crate::arch::runtime::panic_break;
use crate::cap::Console;
use core::fmt;
use core::fmt::Write;

pub struct KConsole(Console);

impl KConsole {
    pub const fn new(cap: Console) -> Self {
        Self(cap)
    }

    pub const fn null() -> Self {
        Self(Console::null())
    }

    pub fn initialize(&mut self, cap: Console) {
        self.0 = cap;
    }

    pub fn print(&mut self, args: fmt::Arguments) {
        match self.0.write_fmt(args) {
            Ok(_) => {}
            Err(_) => unsafe { panic_break() },
        }
    }
}
