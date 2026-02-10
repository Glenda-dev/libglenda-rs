use super::{CapPtr, kernelmethod};
use crate::error::Error;
use crate::ipc::IPC_BUFFER_SIZE;
use crate::ipc::UTCB;
use alloc::string::String;
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
        let utcb = unsafe { UTCB::new() };
        // Backup UTCB state
        let original_msg_tag = utcb.get_msg_tag();
        let original_mrs = utcb.get_mrs();

        let bytes = s.as_bytes();
        let remaining = bytes.len();
        let mut offset = 0;

        while offset < remaining {
            let mut chunk_size = core::cmp::min(remaining - offset, IPC_BUFFER_SIZE);
            if offset + chunk_size < remaining {
                while !s.is_char_boundary(offset + chunk_size) {
                    chunk_size -= 1;
                }
            }
            utcb.clear();
            let written = utcb.write(&bytes[offset..offset + chunk_size]);
            if written != chunk_size {
                return Err(Error::Unknown);
            }
            self.0.invoke(kernelmethod::CONSOLE_PUT_STR, utcb)?;
            offset += chunk_size;
        }

        // Restore UTCB state
        utcb.set_msg_tag(original_msg_tag);
        utcb.set_mrs(original_mrs);
        Ok(())
    }

    pub fn console_get_char(&self) -> char {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let ret = self.0.invoke(kernelmethod::CONSOLE_GET_CHAR, &mut utcb);
        if ret.is_ok() { utcb.get_mr(0) as u8 as char } else { '\0' }
    }

    pub fn console_get_str(&self) -> Result<String, Error> {
        let mut utcb = unsafe { UTCB::new() };
        // 清空 UTCB 以便接收数据
        utcb.clear();
        self.0.invoke(kernelmethod::CONSOLE_GET_STR, &mut utcb)?;
        // MR0 contains length
        let len = utcb.get_mr(0);
        let mut buf = alloc::vec![0u8; len];
        utcb.read(&mut buf);
        String::from_utf8(buf).map_err(|_| Error::Unknown)
    }

    pub fn shell(&self) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        self.0.invoke(kernelmethod::SHELL, &mut utcb)
    }

    pub fn get_time(&self) -> Result<usize, Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        self.0.invoke(kernelmethod::GET_TIME, &mut utcb)?;
        Ok(utcb.get_mr(0))
    }
}

impl fmt::Write for Kernel {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.console_put_str(s).map_err(|_| fmt::Error)
    }
}
