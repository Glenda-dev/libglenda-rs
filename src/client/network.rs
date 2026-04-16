use crate::cap::{Endpoint, Page};
use crate::error::Error;
use crate::interface::{NetworkService, SocketService};
use crate::io::uring::{IOURING_OP_READ, IOURING_OP_WRITE, IoUringClient, IoUringSqe};
use crate::ipc::{MsgFlags, MsgTag, UTCB};
use crate::protocol::{NETWORK_PROTO, network};
use crate::set_mrs;

pub struct NetworkClient {
    endpoint: Endpoint,
    uring: Option<IoUringClient>,
}

impl NetworkClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint, uring: None }
    }

    pub fn setup_uring(&mut self, uring: IoUringClient) {
        self.uring = Some(uring);
    }

    pub fn read_uring(&self, addr: usize, len: u32, user_data: usize) -> Result<(), Error> {
        let Some(uring) = &self.uring else {
            return Err(Error::InvalidArgs);
        };
        let sqe =
            IoUringSqe { opcode: IOURING_OP_READ, addr, len, user_data, ..Default::default() };
        uring.submit(sqe)
    }

    pub fn write_uring(&self, addr: usize, len: u32, user_data: usize) -> Result<(), Error> {
        let Some(uring) = &self.uring else {
            return Err(Error::InvalidArgs);
        };
        let sqe =
            IoUringSqe { opcode: IOURING_OP_WRITE, addr, len, user_data, ..Default::default() };
        uring.submit(sqe)
    }

    pub fn pop_completion(&self) -> Option<crate::io::uring::IoUringCqe> {
        self.uring.as_ref().and_then(|u| u.pop_completion())
    }
}

impl NetworkService for NetworkClient {
    fn socket(&mut self, domain: i32, socket_type: i32, protocol: i32) -> Result<usize, Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::SOCKET, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, domain, socket_type, protocol);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;

        Ok(utcb.get_mr(0))
    }
}

impl SocketService for NetworkClient {
    fn bind(&mut self, address: &[u8]) -> Result<(), Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::BIND, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.write(address);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }

    fn listen(&mut self, backlog: i32) -> Result<(), Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::LISTEN, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, backlog);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }

    fn accept(&mut self) -> Result<usize, Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::ACCEPT, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        Ok(utcb.get_mr(0))
    }

    fn connect(&mut self, address: &[u8]) -> Result<(), Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::CONNECT, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.write(address);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }

    fn send(&mut self, data: &[u8], flags: i32) -> Result<usize, Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::SEND, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let len = utcb.write(data);
        utcb.set_msg_tag(tag);
        set_mrs!(utcb, len, flags);

        self.endpoint.call(&mut utcb)?;
        Ok(utcb.get_mr(0))
    }

    fn recv(&mut self, buffer: &mut [u8], flags: i32) -> Result<usize, Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::RECV, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, buffer.len(), flags);
        utcb.set_msg_tag(tag);

        self.endpoint.call(&mut utcb)?;

        let len = utcb.get_mr(0);
        buffer[..len].copy_from_slice(&utcb.ipc_buffer()[..len]);
        Ok(len)
    }

    fn close(&mut self) -> Result<(), Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::CLOSE, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(tag);

        self.endpoint.call(&mut utcb)
    }

    fn get_sockname(&self, address: &mut [u8]) -> Result<usize, Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::GET_SOCKNAME, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        let len = utcb.get_mr(0);
        address[..len].copy_from_slice(&utcb.ipc_buffer()[..len]);
        Ok(len)
    }

    fn get_peername(&self, address: &mut [u8]) -> Result<usize, Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::GET_PEERNAME, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        let len = utcb.get_mr(0);
        address[..len].copy_from_slice(&utcb.ipc_buffer()[..len]);
        Ok(len)
    }

    fn setsockopt(&mut self, level: i32, optname: i32, optval: &[u8]) -> Result<(), Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::SET_SOCKOPT, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.write(optval);
        set_mrs!(utcb, level as usize, optname as usize, optval.len());
        utcb.set_msg_tag(tag);

        self.endpoint.call(&mut utcb)
    }

    fn getsockopt(&self, level: i32, optname: i32, optval: &mut [u8]) -> Result<usize, Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::GET_SOCKOPT, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        set_mrs!(utcb, level as usize, optname as usize, optval.len());
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        let len = utcb.get_mr(0);
        optval[..len].copy_from_slice(&utcb.ipc_buffer()[..len]);
        Ok(len)
    }

    fn setup_iouring(
        &mut self,
        client_vaddr: usize,
        size: usize,
        frame: Option<Page>,
    ) -> Result<(), Error> {
        let mut flags = MsgFlags::NONE;
        if frame.is_some() {
            flags |= MsgFlags::HAS_CAP;
        }
        let tag = MsgTag::new(NETWORK_PROTO, network::SETUP_IOURING, flags);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, client_vaddr, size);
        if let Some(f) = frame {
            utcb.set_cap_transfer(f.cap());
        }
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }

    fn process_iouring(&mut self) -> Result<(), Error> {
        let tag = MsgTag::new(NETWORK_PROTO, network::PROCESS_IOURING, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }
}
