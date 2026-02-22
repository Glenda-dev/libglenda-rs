use crate::cap::Endpoint;
use crate::error::Error;
use crate::interface::InitService;
use crate::ipc::{Badge, MsgFlags, MsgTag, UTCB};
use crate::protocol::INIT_PROTO;
use crate::protocol::init;
use crate::protocol::init::{ServiceState, ServiceStatus};
use crate::set_mrs;
use alloc::string::String;
use alloc::vec::Vec;

pub struct InitClient {
    endpoint: Endpoint,
}

impl InitClient {
    pub const fn new(ep: Endpoint) -> Self {
        Self { endpoint: ep }
    }
}

impl InitService for InitClient {
    fn start_service(&mut self, service: &str) -> Result<(), Error> {
        let tag = MsgTag::new(INIT_PROTO, init::START, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        unsafe { utcb.write_str(service)? };
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }

    fn stop_service(&mut self, service: &str) -> Result<(), Error> {
        let tag = MsgTag::new(INIT_PROTO, init::STOP, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        unsafe { utcb.write_str(service)? };
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }

    fn restart_service(&mut self, service: &str) -> Result<(), Error> {
        let tag = MsgTag::new(INIT_PROTO, init::RESTART, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        unsafe { utcb.write_str(service)? };
        utcb.set_msg_tag(tag);

        self.endpoint.call(&mut utcb)
    }

    fn reload_service(&mut self, service: &str) -> Result<(), Error> {
        let tag = MsgTag::new(INIT_PROTO, init::RELOAD, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        unsafe { utcb.write_str(service)? };
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }

    fn query_service(&self, service: &str) -> Result<ServiceStatus, Error> {
        let tag = MsgTag::new(INIT_PROTO, init::QUERY, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        unsafe { utcb.write_str(service)? };
        utcb.set_msg_tag(tag);

        self.endpoint.call(&mut utcb)?;

        unsafe { utcb.read_postcard::<ServiceStatus>().map_err(|_| Error::Unknown) }
    }

    fn report_service(&mut self, _badge: Badge, stat: ServiceState) -> Result<(), Error> {
        let tag = MsgTag::new(INIT_PROTO, init::REPORT, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, stat);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }

    fn list_services(&self) -> Result<Vec<(String, ServiceStatus)>, Error> {
        let tag = MsgTag::new(INIT_PROTO, init::LIST, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        unsafe { utcb.read_postcard::<Vec<(String, ServiceStatus)>>().map_err(|_| Error::Unknown) }
    }
}
