use crate::cap::{Endpoint, CapPtr};
use crate::error::Error;
use crate::ipc::{MsgArgs, MsgFlags, MsgTag, UTCB};
use crate::interface::{InitService, SystemClient};
use crate::protocol::{INIT_PROTO, init::{self, ServiceStatus, ServiceState}};
use alloc::string::String;
use alloc::vec::Vec;

pub struct InitClient {
    endpoint: Endpoint,
}

impl InitClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }
}

impl SystemClient for InitClient {
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

impl InitService for InitClient {
    fn start_service(&mut self, service: String) -> Result<(), Error> {
        let tag = MsgTag::new(INIT_PROTO, init::START, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.write(service.as_bytes());
        
        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL, [0; 8])
    }

    fn stop_service(&mut self, service: String) -> Result<(), Error> {
        let tag = MsgTag::new(INIT_PROTO, init::STOP, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.write(service.as_bytes());
        
        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL, [0; 8])
    }

    fn restart_service(&mut self, service: String) -> Result<(), Error> {
        let tag = MsgTag::new(INIT_PROTO, init::RESTART, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.write(service.as_bytes());
        
        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL, [0; 8])
    }

    fn reload_service(&mut self, service: String) -> Result<(), Error> {
        let tag = MsgTag::new(INIT_PROTO, init::RELOAD, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.write(service.as_bytes());
        
        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL, [0; 8])
    }

    fn query_service(&self, service: String) -> Result<ServiceStatus, Error> {
        let tag = MsgTag::new(INIT_PROTO, init::QUERY, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.write(service.as_bytes());
        
        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL, [0; 8])?;
        
        unsafe { utcb.read_postcard::<ServiceStatus>().map_err(|_| Error::Unknown) }
    }

    fn report_service(&self, pid: usize, stat: ServiceState) -> Result<(), Error> {
        let tag = MsgTag::new(INIT_PROTO, init::REPORT, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.mrs_regs = [pid, stat as usize, 0, 0, 0, 0, 0, 0];
        
        self.endpoint.cap().invoke(crate::cap::ipcmethod::SEND, utcb.mrs_regs)
    }

    fn list_services(&self) -> Result<Vec<(String, ServiceStatus)>, Error> {
        let tag = MsgTag::new(INIT_PROTO, init::LIST, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        
        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL, [0; 8])?;
        
        unsafe { utcb.read_postcard::<Vec<(String, ServiceStatus)>>().map_err(|_| Error::Unknown) }
    }
}
