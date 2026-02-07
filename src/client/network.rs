use crate::cap::{Endpoint, CapPtr};
use crate::error::Error;
use crate::ipc::{MsgArgs, MsgFlags, MsgTag, UTCB};
use crate::interface::{NetworkService, SocketService, SystemClient};
use crate::protocol::{NETWORK_PROTO, network};

pub struct NetworkClient {
    endpoint: Endpoint,
}

impl NetworkClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }
}

impl SystemClient for NetworkClient {
    fn connect(&mut self, ep: Endpoint, _reply: CapPtr) -> Result<(), Error> {
        self.endpoint = ep;
        Ok(())
    }

    fn disconnect(&mut self) {}

    fn send(
        &mut self,
        label: usize,
        proto: usize,
        flags: MsgFlags,
        msg: MsgArgs,
    ) -> Result<(), Error> {
        let tag = MsgTag::new(proto, label, flags);
        self.endpoint.send(tag, msg)
    }
}

impl NetworkService for NetworkClient {
    fn socket(&mut self, domain: i32, socket_type: i32, protocol: i32) -> Result<usize, Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::SOCKET, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.mrs_regs = [domain as usize, socket_type as usize, protocol as usize, 0, 0, 0, 0, 0];
        
        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL, utcb.mrs_regs)?;
        
        Ok(utcb.mrs_regs[0])
    }
}

impl SocketService for NetworkClient {
    fn bind(&mut self, address: &[u8]) -> Result<(), Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::BIND, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.write(address);
        
        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL, [0; 8])
    }

    fn listen(&mut self, backlog: i32) -> Result<(), Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::LISTEN, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.mrs_regs = [backlog as usize, 0, 0, 0, 0, 0, 0, 0];
        
        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL, utcb.mrs_regs)
    }

    fn accept(&mut self) -> Result<usize, Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::ACCEPT, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        
        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL, [0; 8])?;
        
        Ok(utcb.mrs_regs[0])
    }

    fn connect(&mut self, address: &[u8]) -> Result<(), Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::CONNECT, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.write(address);
        
        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL, [0; 8])
    }

    fn send(&mut self, data: &[u8], flags: i32) -> Result<usize, Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::SEND, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        let len = utcb.write(data);
        utcb.mrs_regs = [len, flags as usize, 0, 0, 0, 0, 0, 0];
        
        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL, utcb.mrs_regs)?;
        
        Ok(utcb.mrs_regs[0])
    }

    fn recv(&mut self, buffer: &mut [u8], flags: i32) -> Result<usize, Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::RECV, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.mrs_regs = [buffer.len(), flags as usize, 0, 0, 0, 0, 0, 0];
        
        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL, utcb.mrs_regs)?;
        
        let len = utcb.mrs_regs[0];
        buffer[..len].copy_from_slice(&utcb.ipc_buffer[..len]);
        Ok(len)
    }

    fn close(&mut self) -> Result<(), Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::CLOSE, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        
        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL, [0; 8])
    }

    fn get_sockname(&self, address: &mut [u8]) -> Result<usize, Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::GET_SOCKNAME, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        
        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL, [0; 8])?;
        
        let len = utcb.mrs_regs[0];
        address[..len].copy_from_slice(&utcb.ipc_buffer[..len]);
        Ok(len)
    }

    fn get_peername(&self, address: &mut [u8]) -> Result<usize, Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::GET_PEERNAME, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        
        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL, [0; 8])?;
        
        let len = utcb.mrs_regs[0];
        address[..len].copy_from_slice(&utcb.ipc_buffer[..len]);
        Ok(len)
    }

    fn setsockopt(&mut self, level: i32, optname: i32, optval: &[u8]) -> Result<(), Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::SET_SOCKOPT, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.write(optval);
        utcb.mrs_regs = [level as usize, optname as usize, 0, 0, 0, 0, 0, 0];
        
        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL, utcb.mrs_regs)
    }

    fn getsockopt(&self, level: i32, optname: i32, optval: &mut [u8]) -> Result<usize, Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::GET_SOCKOPT, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.mrs_regs = [level as usize, optname as usize, optval.len(), 0, 0, 0, 0, 0];
        
        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL, utcb.mrs_regs)?;
        
        let len = utcb.mrs_regs[0];
        optval[..len].copy_from_slice(&utcb.ipc_buffer[..len]);
        Ok(len)
    }
}
