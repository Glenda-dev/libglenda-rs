use super::{CapPtr, kernelmethod};
use crate::error::Error;
use crate::ipc::{MsgFlags, MsgTag, UTCB};
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

        // Backup current UTCB state to prevent corruption of nested/interrupted IPC
        let old_tag = utcb.get_msg_tag();
        let old_mrs = utcb.get_mrs();
        let old_head = utcb.get_head();
        let old_size = utcb.get_size();

        // Backup first 256 bytes of IPC buffer
        const LOG_CHUNK_SIZE: usize = 256;
        let mut buffer_backup = [0u8; LOG_CHUNK_SIZE];
        unsafe {
            core::ptr::copy_nonoverlapping(
                utcb.get_buffer_ptr(),
                buffer_backup.as_mut_ptr(),
                LOG_CHUNK_SIZE,
            );
        }

        let bytes = s.as_bytes();
        let remaining = bytes.len();
        let mut offset = 0;

        while offset < remaining {
            let mut chunk_size = core::cmp::min(remaining - offset, LOG_CHUNK_SIZE);
            if offset + chunk_size < remaining {
                while !s.is_char_boundary(offset + chunk_size) {
                    chunk_size -= 1;
                }
            }

            // Set up UTCB for console_put_str
            utcb.set_head(0);
            utcb.set_size(chunk_size);
            unsafe {
                core::ptr::copy_nonoverlapping(
                    bytes.as_ptr().add(offset),
                    utcb.get_buffer_ptr(),
                    chunk_size,
                );
            }
            // MR0 contains length for the kernel
            utcb.set_mr(0, chunk_size);
            utcb.set_msg_tag(MsgTag::new(0, 0, MsgFlags::NONE));

            if let Err(e) = self.0.invoke(kernelmethod::CONSOLE_PUT_STR, utcb) {
                // Restore state before exit
                utcb.set_msg_tag(old_tag);
                utcb.set_mrs(old_mrs);
                utcb.set_head(old_head);
                utcb.set_size(old_size);
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        buffer_backup.as_ptr(),
                        utcb.get_buffer_ptr(),
                        LOG_CHUNK_SIZE,
                    );
                }
                return Err(e);
            }
            offset += chunk_size;
        }

        // Restore state fully
        utcb.set_msg_tag(old_tag);
        utcb.set_mrs(old_mrs);
        utcb.set_head(old_head);
        utcb.set_size(old_size);
        unsafe {
            core::ptr::copy_nonoverlapping(
                buffer_backup.as_ptr(),
                utcb.get_buffer_ptr(),
                LOG_CHUNK_SIZE,
            );
        }

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
