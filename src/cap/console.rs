use super::{CapPtr, consolemethod};
use crate::ipc::utcb;
use core::fmt;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Console(CapPtr);

impl Console {
    pub const fn from(cap: CapPtr) -> Self {
        Self(cap)
    }

    pub fn cap(&self) -> CapPtr {
        self.0
    }

    pub const fn null() -> Self {
        Self(CapPtr::null())
    }

    pub fn put_char(&self, c: char) -> usize {
        self.0.invoke(consolemethod::PUT_CHAR, [c as usize, 0, 0, 0, 0, 0, 0])
    }

    pub fn put_str(&self, s: &str) -> usize {
        let utcb = unsafe { utcb::get() };
        if let Some((offset, len)) = utcb.append_str(s) {
            self.0.invoke(consolemethod::PUT_STR, [offset, len, 0, 0, 0, 0, 0])
        } else {
            // Buffer overflow
            1
        }
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match self.put_str(s) {
            0 => Ok(()),
            _ => Err(fmt::Error),
        }
    }
}
