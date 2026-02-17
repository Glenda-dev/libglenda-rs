use crate::cap::{CNode, CapPtr, Endpoint};
use crate::error::Error;
use crate::interface::{ProcessService, ThreadService};
use crate::ipc::{Badge, MsgFlags, MsgTag, UTCB};
use crate::protocol::PROCESS_PROTO;
use crate::protocol::process;
use crate::set_mrs;

pub struct ProcessClient {
    endpoint: Endpoint,
}

impl ProcessClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }
}

impl ProcessService for ProcessClient {
    fn get_pid(&mut self, _pid: Badge) -> Result<usize, Error> {
        let tag = MsgTag::new(PROCESS_PROTO, process::GET_PID, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        Ok(utcb.get_mr(0))
    }

    fn get_ppid(&mut self, _pid: Badge) -> Result<usize, Error> {
        let tag = MsgTag::new(PROCESS_PROTO, process::GET_PPID, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        Ok(utcb.get_mr(0))
    }

    fn spawn(&mut self, _pid: Badge, name: &str) -> Result<usize, Error> {
        let tag = MsgTag::new(PROCESS_PROTO, process::SPAWN, MsgFlags::HAS_BUFFER);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        unsafe { utcb.write_str(name)? };
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        Ok(utcb.get_mr(0))
    }

    fn fork(&mut self, _pid: Badge) -> Result<usize, Error> {
        let tag = MsgTag::new(PROCESS_PROTO, process::FORK, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        Ok(utcb.get_mr(0))
    }

    fn exit(&mut self, _pid: Badge, code: usize) -> Result<(), Error> {
        let tag = MsgTag::new(PROCESS_PROTO, process::EXIT, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, code);
        utcb.set_msg_tag(tag);
        self.endpoint.send(&mut utcb)
    }

    fn kill(&mut self, _pid: Badge, target: usize) -> Result<(), Error> {
        let tag = MsgTag::new(PROCESS_PROTO, process::KILL, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_mr(0, target);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        Ok(())
    }

    fn exec(&mut self, _pid: Badge, path: &str) -> Result<(usize, usize), Error> {
        let tag = MsgTag::new(PROCESS_PROTO, process::EXEC, MsgFlags::HAS_BUFFER);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        unsafe { utcb.write_str(path)? };
        utcb.set_msg_tag(tag);
        // Note: passing large buffers might need another mechanism if it exceeds IPC_BUFFER_SIZE
        // For now we assume the caller handled it if it fits, or this is just a protocol definition.
        // Usually, exec might use a Frame capability instead of raw data in IPC buffer.
        self.endpoint.call(&mut utcb)?;
        Ok((utcb.get_mr(0), utcb.get_mr(1)))
    }

    fn get_cnode(&mut self, _pid: Badge, target: Badge, recv: CapPtr) -> Result<CNode, Error> {
        let tag = MsgTag::new(PROCESS_PROTO, process::GET_CNODE, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(tag);
        utcb.set_mr(0, target.bits());
        utcb.set_recv_window(recv);
        self.endpoint.call(&mut utcb)?;
        Ok(CNode::from(recv))
    }
}

impl ThreadService for ProcessClient {
    fn thread_create(
        &mut self,
        _pid: Badge,
        entry: usize,
        arg: usize,
        stack_top: usize,
        tls: usize,
    ) -> Result<usize, Error> {
        let tag = MsgTag::new(PROCESS_PROTO, process::THREAD_CREATE, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, entry, arg, stack_top, tls);
        utcb.set_msg_tag(tag);
        // Note: Currently thread creation still uses the Process endpoint, which is fine
        // as the server likely handles both on the same capability or separate ones.
        // Assuming ProcessClient uses an endpoint that maps to the ProcessManager logic
        // which implements both services.
        self.endpoint.call(&mut utcb)?;
        Ok(utcb.get_mr(0))
    }
}
