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
    fn spawn(&mut self, _pid: Badge, name: &str) -> Result<usize, Error> {
        let tag = MsgTag::new(PROCESS_PROTO, process::SPAWN, MsgFlags::HAS_BUFFER);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        unsafe { utcb.write_str(name)? };
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        Ok(utcb.get_mr(0))
    }

    fn create(&mut self, _pid: Badge, name: &str) -> Result<usize, Error> {
        let tag = MsgTag::new(PROCESS_PROTO, process::CREATE, MsgFlags::HAS_BUFFER);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        unsafe { utcb.write_str(name)? };
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

    fn get_cnode(&mut self, _pid: Badge, target: usize, recv: CapPtr) -> Result<CNode, Error> {
        let tag = MsgTag::new(PROCESS_PROTO, process::GET_CNODE, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(tag);
        utcb.set_mr(0, target);
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
        self.endpoint.call(&mut utcb)?;
        Ok(utcb.get_mr(0))
    }
}
