use super::{CapPtr, kernelmethod};
use crate::ipc::utcb;
use core::fmt;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Kernel(CapPtr);

impl Kernel {
    pub const fn from(cap: CapPtr) -> Self {
        Self(cap)
    }

    pub fn cap(&self) -> CapPtr {
        self.0
    }

    pub const fn null() -> Self {
        Self(CapPtr::null())
    }

    pub fn console_put_str(&self, s: &str) -> usize {
        let utcb = unsafe { utcb::get() };
        if let Some((offset, len)) = utcb.append_str(s) {
            self.0.invoke(kernelmethod::CONSOLE_PUT_STR, [offset, len, 0, 0, 0, 0, 0])
        } else {
            // Buffer overflow
            1
        }
    }

    pub fn console_get_char(&self) -> char {
        let utcb = unsafe { utcb::get() };
        let ret = self.0.invoke(kernelmethod::CONSOLE_GET_CHAR, [0, 0, 0, 0, 0, 0, 0]);
        if ret == 0 { utcb.mrs_regs[0] as u8 as char } else { '\0' }
    }

    pub fn shell(&self) -> usize {
        self.0.invoke(kernelmethod::SHELL, [0, 0, 0, 0, 0, 0, 0])
    }
}

impl fmt::Write for Kernel {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match self.console_put_str(s) {
            0 => Ok(()),
            _ => Err(fmt::Error),
        }
    }
}
