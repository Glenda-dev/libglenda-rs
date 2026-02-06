use super::{CapPtr, kernelmethod};
use crate::error::Error;
use crate::ipc::UTCB;
use crate::ipc::utcb::MAX_MRS;
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

    pub fn console_put_str(&self, s: &str) -> Result<(), Error> {
        let utcb = unsafe { UTCB::get() };
        utcb.clear();
        let len = utcb.write(s.as_bytes());
        if len == s.len() {
            self.0.invoke(kernelmethod::CONSOLE_PUT_STR, [0; MAX_MRS])
        } else {
            // Buffer overflow
            Err(Error::Unknown)
        }
    }

    pub fn console_get_char(&self) -> char {
        let utcb = unsafe { UTCB::get() };
        let ret = self.0.invoke(kernelmethod::CONSOLE_GET_CHAR, [0; MAX_MRS]);
        if ret.is_ok() { utcb.mrs_regs[0] as u8 as char } else { '\0' }
    }

    pub fn console_get_str(&self) -> Result<alloc::string::String, Error> {
        let utcb = unsafe { UTCB::get() };
        // 清空 UTCB 以便接收数据
        utcb.clear();
        let ret = self.0.invoke(kernelmethod::CONSOLE_GET_STR, [0; MAX_MRS]);
        if ret.is_ok() {
            // MR0 contains length
            let len = utcb.mrs_regs[0];
            let mut buf = alloc::vec![0u8; len];
            utcb.read(&mut buf);
            alloc::string::String::from_utf8(buf).map_err(|_| Error::Unknown)
        } else {
            Err(ret.unwrap_err())
        }
    }

    pub fn shell(&self) -> Result<(), Error> {
        self.0.invoke(kernelmethod::SHELL, [0; MAX_MRS])
    }

    pub fn get_time(&self) -> Result<usize, Error> {
        self.0.invoke(kernelmethod::SHELL, [0; MAX_MRS])?;
        let utcb = unsafe { UTCB::get() };
        Ok(utcb.mrs_regs[0])
    }
}

impl fmt::Write for Kernel {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.console_put_str(s).map_err(|_| fmt::Error)
    }
}
