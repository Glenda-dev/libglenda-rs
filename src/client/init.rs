use crate::cap::{CapPtr, Endpoint, Reply};
use crate::error::Error;
use crate::interface::{InitService, SystemClient};
use crate::ipc::{Badge, MsgFlags, MsgTag, UTCB};
use crate::protocol::INIT_PROTO;
use crate::protocol::init;
use crate::protocol::init::{ServiceState, ServiceStatus};
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
        utcb.write_str(service.as_str())?;

        self.endpoint.call(tag)
    }

    fn stop_service(&mut self, service: String) -> Result<(), Error> {
        let tag = MsgTag::new(INIT_PROTO, init::STOP, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.write_str(service.as_str())?;

        self.endpoint.call(tag)
    }

    fn restart_service(&mut self, service: String) -> Result<(), Error> {
        let tag = MsgTag::new(INIT_PROTO, init::RESTART, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.write_str(service.as_str())?;

        self.endpoint.call(tag)
    }

    fn reload_service(&mut self, service: String) -> Result<(), Error> {
        let tag = MsgTag::new(INIT_PROTO, init::RELOAD, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.write_str(service.as_str())?;

        self.endpoint.call(tag)
    }

    fn query_service(&self, service: String) -> Result<ServiceStatus, Error> {
        let tag = MsgTag::new(INIT_PROTO, init::QUERY, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.write_str(service.as_str())?;

        self.endpoint.call(tag)?;

        unsafe { utcb.read_postcard::<ServiceStatus>().map_err(|_| Error::Unknown) }
    }

    fn report_service(&self, _badge: Badge, stat: ServiceState) -> Result<(), Error> {
        let tag = MsgTag::new(INIT_PROTO, init::REPORT, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.mrs_regs[0] = stat as usize;
        self.endpoint.send(tag)
    }

    fn list_services(&self) -> Result<Vec<(String, ServiceStatus)>, Error> {
        let tag = MsgTag::new(INIT_PROTO, init::LIST, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        self.endpoint.call(tag)?;
        unsafe { utcb.read_postcard::<Vec<(String, ServiceStatus)>>().map_err(|_| Error::Unknown) }
    }
}
