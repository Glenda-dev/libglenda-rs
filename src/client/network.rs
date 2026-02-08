use crate::cap::{CapPtr, Endpoint};
use crate::error::Error;
use crate::interface::{NetworkService, SocketService, SystemClient};
use crate::ipc::{MsgFlags, MsgTag, UTCB};
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

    fn send(&mut self, info: MsgTag) -> Result<(), Error> {
        self.endpoint.send(info)
    }
}

impl NetworkService for NetworkClient {
    fn socket(&mut self, domain: i32, socket_type: i32, protocol: i32) -> Result<usize, Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::SOCKET, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.mrs_regs[0] = domain as usize;
        utcb.mrs_regs[1] = socket_type as usize;
        utcb.mrs_regs[2] = protocol as usize;

        self.endpoint.call(tag)?;

        Ok(utcb.mrs_regs[0])
    }
}

impl SocketService for NetworkClient {
    fn bind(&mut self, address: &[u8]) -> Result<(), Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::BIND, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.write(address);

        self.endpoint.call(tag)
    }

    fn listen(&mut self, backlog: i32) -> Result<(), Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::LISTEN, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.mrs_regs[0] = backlog as usize;

        self.endpoint.call(tag)
    }

    fn accept(&mut self) -> Result<usize, Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::ACCEPT, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };

        self.endpoint.call(tag)?;

        Ok(utcb.mrs_regs[0])
    }

    fn connect(&mut self, address: &[u8]) -> Result<(), Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::CONNECT, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.write(address);

        self.endpoint.call(tag)
    }

    fn send(&mut self, data: &[u8], flags: i32) -> Result<usize, Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::SEND, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        let len = utcb.write(data);
        utcb.mrs_regs[0] = len;
        utcb.mrs_regs[1] = flags as usize;

        self.endpoint.call(tag)?;

        Ok(utcb.mrs_regs[0])
    }

    fn recv(&mut self, buffer: &mut [u8], flags: i32) -> Result<usize, Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::RECV, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.mrs_regs[0] = buffer.len();
        utcb.mrs_regs[1] = flags as usize;

        self.endpoint.call(tag)?;

        let len = utcb.mrs_regs[0];
        buffer[..len].copy_from_slice(&utcb.ipc_buffer[..len]);
        Ok(len)
    }

    fn close(&mut self) -> Result<(), Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::CLOSE, MsgFlags::NONE);

        self.endpoint.call(tag)
    }

    fn get_sockname(&self, address: &mut [u8]) -> Result<usize, Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::GET_SOCKNAME, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };

        self.endpoint.call(tag)?;

        let len = utcb.mrs_regs[0];
        address[..len].copy_from_slice(&utcb.ipc_buffer[..len]);
        Ok(len)
    }

    fn get_peername(&self, address: &mut [u8]) -> Result<usize, Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::GET_PEERNAME, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };

        self.endpoint.call(tag)?;

        let len = utcb.mrs_regs[0];
        address[..len].copy_from_slice(&utcb.ipc_buffer[..len]);
        Ok(len)
    }

    fn setsockopt(&mut self, level: i32, optname: i32, optval: &[u8]) -> Result<(), Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::SET_SOCKOPT, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.write(optval);
        utcb.mrs_regs[0] = level as usize;
        utcb.mrs_regs[1] = optname as usize;

        self.endpoint.call(tag)
    }

    fn getsockopt(&self, level: i32, optname: i32, optval: &mut [u8]) -> Result<usize, Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::GET_SOCKOPT, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.mrs_regs[0] = level as usize;
        utcb.mrs_regs[1] = optname as usize;
        utcb.mrs_regs[2] = optval.len();

        self.endpoint.call(tag)?;

        let len = utcb.mrs_regs[0];
        optval[..len].copy_from_slice(&utcb.ipc_buffer[..len]);
        Ok(len)
    }
}
