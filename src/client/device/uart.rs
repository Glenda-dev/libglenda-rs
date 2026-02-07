use crate::cap::Endpoint;
use crate::interface::device::UartDevice;
use crate::ipc::{MsgFlags, MsgTag, UTCB};
use crate::protocol::device::{UART_PROTO, uart};

pub struct UartClient {
    endpoint: Endpoint,
}

impl UartClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }
}

impl UartDevice for UartClient {
    fn put_char(&mut self, c: u8) {
        let utcb = unsafe { UTCB::get() };
        let tag = MsgTag::new(UART_PROTO, uart::PUT_CHAR, MsgFlags::NONE);
    }

    fn get_char(&mut self) -> Option<u8> {
        let utcb = unsafe { UTCB::get() };
        let tag = MsgTag::new(UART_PROTO, uart::GET_CHAR, MsgFlags::NONE);

        match self.endpoint.call(tag) {
            Ok(_) => Some(utcb.mrs_regs[0] as u8),
            Err(_) => None,
        }
    }

    fn put_str(&mut self, s: &str) {
        let utcb = unsafe { UTCB::get() };
        let buf = &mut utcb.ipc_buffer;
        let bytes = s.as_bytes();

        for chunk in bytes.chunks(buf.len()) {
            buf[..chunk.len()].copy_from_slice(chunk);
            let tag = MsgTag::new(UART_PROTO, uart::PUT_STR, MsgFlags::NONE);
            utcb.size = chunk.len();

            let _ = self.endpoint.call(tag);
        }
    }
}
