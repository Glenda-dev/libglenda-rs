use crate::cap::{CNode, CapPtr, Endpoint};
use crate::error::Error;
use crate::interface::{ProcessService, SystemClient};
use crate::ipc::{Badge, MsgFlags, MsgTag, UTCB};
use crate::protocol::PROCESS_PROTO;
use crate::protocol::process;
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
    fn get_pid(&mut self, _pid: Badge) -> Result<usize, Error> {
        let tag = MsgTag::new(PROCESS_PROTO, process::GET_PID, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };

        self.endpoint.call(tag)?;
        Ok(utcb.mrs_regs[0])
    }

    fn get_ppid(&mut self, _pid: Badge) -> Result<usize, Error> {
        let tag = MsgTag::new(PROCESS_PROTO, process::GET_PPID, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };

        self.endpoint.call(tag)?;
        Ok(utcb.mrs_regs[0])
    }

    fn spawn(&mut self, _pid: Badge, name: String) -> Result<usize, Error> {
        let tag = MsgTag::new(PROCESS_PROTO, process::SPAWN, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.write(name.as_bytes());
        self.endpoint.call(tag)?;
        Ok(utcb.mrs_regs[0])
    }

    fn fork(&mut self, _pid: Badge) -> Result<usize, Error> {
        let tag = MsgTag::new(PROCESS_PROTO, process::FORK, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.mrs_regs[0] = 0;
        self.endpoint.call(tag)?;
        Ok(utcb.mrs_regs[0])
    }

    fn exit(&mut self, _pid: Badge, code: usize) -> Result<(), Error> {
        let tag = MsgTag::new(PROCESS_PROTO, process::EXIT, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.mrs_regs[0] = code;
        self.endpoint.send(tag)
    }

    fn exec(&mut self, _pid: Badge, elf_data: &[u8]) -> Result<(usize, usize), Error> {
        let tag = MsgTag::new(PROCESS_PROTO, process::EXEC, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.mrs_regs[0] = elf_data.len();
        // Note: passing large buffers might need another mechanism if it exceeds BUFFER_MAX_SIZE
        // For now we assume the caller handled it if it fits, or this is just a protocol definition.
        // Usually, exec might use a Frame capability instead of raw data in IPC buffer.
        self.endpoint.call(tag)?;

        Ok((utcb.mrs_regs[0], utcb.mrs_regs[1]))
    }

    fn get_cnode(&mut self, _pid: Badge, target: Badge) -> Result<CNode, Error> {
        let tag = MsgTag::new(PROCESS_PROTO, process::GET_CNODE, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.mrs_regs[0] = target.bits();
        self.endpoint.call(tag)?;
        Ok(CNode::from(utcb.recv_window))
    }
}
