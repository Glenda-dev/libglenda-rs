use crate::arch::runtime::panic_break;
use crate::cap::Kernel;
use core::fmt;
use core::fmt::Write;

pub struct KConsole(Kernel);

impl KConsole {
    pub const fn new(cap: Kernel) -> Self {
        Self(cap)
    }

    pub const fn null() -> Self {
        Self(Kernel::null())
    }

    pub fn initialize(&mut self, cap: Kernel) {
        self.0 = cap;
    }

    pub fn print(&mut self, args: fmt::Arguments) {
        match self.0.write_fmt(args) {
            Ok(_) => {}
            Err(_) => unsafe { panic_break() },
        }
    }
}
