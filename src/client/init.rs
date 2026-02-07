use crate::cap::{CapPtr, Endpoint, Reply};
use crate::error::Error;
use crate::interface::{InitService, SystemClient};
use crate::ipc::{MsgFlags, MsgTag, UTCB};
use crate::protocol::{
    INIT_PROTO,
    init::{self, ServiceState, ServiceStatus},
};
use alloc::string::String;
use alloc::vec::Vec;

pub struct InitClient {
    endpoint: Endpoint,
    reply: Reply,
}

impl InitClient {
    pub const fn new() -> Self {
        Self { endpoint: Endpoint::from(CapPtr::null()), reply: Reply::from(CapPtr::null()) }
    }
}

impl SystemClient for InitClient {
    fn connect(&mut self, ep: Endpoint, reply: CapPtr) -> Result<(), Error> {
        self.endpoint = ep;
        self.reply = Reply::from(reply);
        Ok(())
    }

    fn disconnect(&mut self) {}

    fn send(&mut self, info: MsgTag) -> Result<(), Error> {
        self.endpoint.send(info)
    }
}

impl InitService for InitClient {
    fn start_service(&mut self, service: String) -> Result<(), Error> {
        let tag = MsgTag::new(INIT_PROTO, init::START, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.write(service.as_bytes());
        utcb.mrs_regs = [0; 8];

        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL)
    }

    fn stop_service(&mut self, service: String) -> Result<(), Error> {
        let tag = MsgTag::new(INIT_PROTO, init::STOP, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.write(service.as_bytes());
        utcb.mrs_regs = [0; 8];

        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL)
    }

    fn restart_service(&mut self, service: String) -> Result<(), Error> {
        let tag = MsgTag::new(INIT_PROTO, init::RESTART, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.write(service.as_bytes());
        utcb.mrs_regs = [0; 8];

        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL)
    }

    fn reload_service(&mut self, service: String) -> Result<(), Error> {
        let tag = MsgTag::new(INIT_PROTO, init::RELOAD, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.write(service.as_bytes());
        utcb.mrs_regs = [0; 8];

        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL)
    }

    fn query_service(&self, service: String) -> Result<ServiceStatus, Error> {
        let tag = MsgTag::new(INIT_PROTO, init::QUERY, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.write(service.as_bytes());
        utcb.mrs_regs = [0; 8];

        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL)?;

        unsafe { utcb.read_postcard::<ServiceStatus>().map_err(|_| Error::Unknown) }
    }

    fn report_service(&self, pid: usize, stat: ServiceState) -> Result<(), Error> {
        let tag = MsgTag::new(INIT_PROTO, init::REPORT, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.mrs_regs = [pid, stat as usize, 0, 0, 0, 0, 0, 0];

        self.endpoint.cap().invoke(crate::cap::ipcmethod::SEND)
    }

    fn list_services(&self) -> Result<Vec<(String, ServiceStatus)>, Error> {
        let tag = MsgTag::new(INIT_PROTO, init::LIST, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.mrs_regs = [0; 8];

        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL)?;

        unsafe { utcb.read_postcard::<Vec<(String, ServiceStatus)>>().map_err(|_| Error::Unknown) }
    }
}
