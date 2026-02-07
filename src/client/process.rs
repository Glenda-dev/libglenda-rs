use crate::cap::{CapPtr, Endpoint};
use crate::error::Error;
use crate::interface::{ProcessService, SystemClient};
use crate::ipc::{Badge, MsgFlags, MsgTag, UTCB};
use crate::protocol::{PROCESS_PROTO, process};
use alloc::string::String;

pub struct ProcessClient {
    endpoint: Endpoint,
}

impl ProcessClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }
}

impl SystemClient for ProcessClient {
    fn connect(&mut self, ep: Endpoint, _reply: CapPtr) -> Result<(), Error> {
        self.endpoint = ep;
        Ok(())
    }

    fn disconnect(&mut self) {}

    fn send(&mut self, info: MsgTag) -> Result<(), Error> {
        self.endpoint.send(info)
    }
}

impl ProcessService for ProcessClient {
    fn get_pid(&mut self, pid: Badge) -> Result<usize, Error> {
        let tag = MsgTag::new(PROCESS_PROTO, process::GET_PID, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.mrs_regs = [0; 8];
        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL)?;
        Ok(utcb.mrs_regs[0])
    }

    fn spawn(&mut self, pid: Badge, name: String) -> Result<usize, Error> {
        let tag = MsgTag::new(PROCESS_PROTO, process::SPAWN, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.write(name.as_bytes());
        utcb.mrs_regs = [0; 8];

        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL)?;

        Ok(utcb.mrs_regs[0])
    }

    fn fork(&mut self, pid: Badge) -> Result<usize, Error> {
        let tag = MsgTag::new(PROCESS_PROTO, process::FORK, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.mrs_regs = [pid.bits(), 0, 0, 0, 0, 0, 0, 0];

        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL)?;

        Ok(utcb.mrs_regs[0])
    }

    fn exit(&mut self, pid: Badge, code: usize) -> Result<(), Error> {
        let tag = MsgTag::new(PROCESS_PROTO, process::EXIT, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.mrs_regs = [pid.bits(), code, 0, 0, 0, 0, 0, 0];

        self.endpoint.cap().invoke(crate::cap::ipcmethod::SEND)
    }

    fn load_image(&mut self, pid: Badge, elf_data: &[u8]) -> Result<(usize, usize), Error> {
        let tag = MsgTag::new(PROCESS_PROTO, process::EXEC, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.mrs_regs = [pid.bits(), elf_data.len(), 0, 0, 0, 0, 0, 0];
        // Note: passing large buffers might need another mechanism if it exceeds BUFFER_MAX_SIZE
        // For now we assume the caller handled it if it fits, or this is just a protocol definition.
        // Usually, load_image might use a Frame capability instead of raw data in IPC buffer.

        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL)?;

        Ok((utcb.mrs_regs[0], utcb.mrs_regs[1]))
    }
}
