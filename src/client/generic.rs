use crate::cap::CapPtr;
use crate::cap::Endpoint;
use crate::error::Error;
use crate::interface::GeneralService;
use crate::ipc::{MsgFlags, MsgTag, UTCB};
use crate::protocol::{GENERIC_PROTO, generic};
use crate::set_mrs;

pub struct GeneralClient {
    endpoint: Endpoint,
}

impl GeneralClient {
    pub fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }
}

impl GeneralService for GeneralClient {
    fn ping(&mut self, value: usize) -> Result<usize, Error> {
        let tag = MsgTag::new(GENERIC_PROTO, generic::PING, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, value);
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        Ok(utcb.get_mr(0))
    }

    fn share_memory(&mut self, cap: CapPtr) -> Result<(), Error> {
        let tag = MsgTag::new(GENERIC_PROTO, generic::SHARE_MEMORY, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_cap_transfer(cap);
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        Ok(())
    }

    fn send_message(&mut self, message: &str) -> Result<(), Error> {
        let tag = MsgTag::new(GENERIC_PROTO, generic::SEND_MESSAGE, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        unsafe { utcb.write_str(message)? };
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        Ok(())
    }

    fn notify(&mut self) -> Result<(), Error> {
        let tag = MsgTag::new(GENERIC_PROTO, generic::NOTIFY, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(tag);
        self.endpoint.notify(utcb)?;
        Ok(())
    }
}
