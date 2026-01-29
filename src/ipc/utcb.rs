use super::MsgTag;
use crate::cap::{CapPtr, Endpoint};
use crate::mem::UTCB_VA;

pub const BUFFER_MAX_SIZE: usize = 3 * 1024; // 3KB
pub const MAX_MRS: usize = 7;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UTCB {
    pub msg_tag: MsgTag,
    pub mrs_regs: [usize; MAX_MRS],
    pub cap_transfer: CapPtr,
    pub recv_window: Endpoint,
    pub tls: usize,
    pub head: usize,
    pub tail: usize,
    pub ipc_buffer: [u8; BUFFER_MAX_SIZE],
}

impl UTCB {
    pub fn current() -> &'static mut Self {
        unsafe { &mut *(UTCB_VA as *mut UTCB) }
    }

    pub fn available_data(&self) -> usize {
        if self.tail >= self.head {
            self.tail - self.head
        } else {
            BUFFER_MAX_SIZE - self.head + self.tail
        }
    }

    pub fn available_space(&self) -> usize {
        BUFFER_MAX_SIZE - self.available_data() - 1
    }

    pub fn write(&mut self, data: &[u8]) -> usize {
        let len = core::cmp::min(data.len(), self.available_space());
        for i in 0..len {
            self.ipc_buffer[self.tail] = data[i];
            self.tail = (self.tail + 1) % BUFFER_MAX_SIZE;
        }
        len
    }

    pub fn read(&mut self, data: &mut [u8]) -> usize {
        let len = core::cmp::min(data.len(), self.available_data());
        for i in 0..len {
            data[i] = self.ipc_buffer[self.head];
            self.head = (self.head + 1) % BUFFER_MAX_SIZE;
        }
        len
    }

    pub fn read_str(&self, offset: usize, len: usize) -> Option<alloc::string::String> {
        if len > BUFFER_MAX_SIZE || offset >= BUFFER_MAX_SIZE {
            return None;
        }
        let mut buf = alloc::vec![0u8; len];
        if offset + len <= BUFFER_MAX_SIZE {
            buf.copy_from_slice(&self.ipc_buffer[offset..offset + len]);
        } else {
            let part1_len = BUFFER_MAX_SIZE - offset;
            let part2_len = len - part1_len;
            buf[..part1_len].copy_from_slice(&self.ipc_buffer[offset..]);
            buf[part1_len..].copy_from_slice(&self.ipc_buffer[..part2_len]);
        }
        alloc::string::String::from_utf8(buf).ok()
    }

    pub fn append_str(&mut self, s: &str) -> Option<(usize, usize)> {
        let start = self.tail;
        let len = self.write(s.as_bytes());
        if len == s.len() { Some((start, len)) } else { None }
    }

    pub fn append_bytes(&mut self, bytes: &[u8]) -> Option<(usize, usize)> {
        let start = self.tail;
        let len = self.write(bytes);
        if len == bytes.len() { Some((start, len)) } else { None }
    }

    pub fn clear(&mut self) {
        self.msg_tag = MsgTag::empty();
        self.mrs_regs = [0; MAX_MRS];
        self.cap_transfer = CapPtr::null();
        self.recv_window = Endpoint::from(CapPtr::null());
        self.tls = 0;
        self.head = 0;
        self.tail = 0;
        for byte in self.ipc_buffer.iter_mut() {
            *byte = 0;
        }
    }
}

pub unsafe fn get() -> &'static mut UTCB {
    unsafe { &mut *(UTCB_VA as *mut UTCB) }
}
