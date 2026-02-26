use crate::cap::Endpoint;
use crate::error::Error;
use crate::ipc::{MsgFlags, MsgTag, UTCB, Badge};
use crate::protocol::{TIME_PROTO, time};
use crate::interface::TimeService;
use crate::set_mrs;

pub struct TimeClient(Endpoint);

impl TimeClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self(endpoint)
    }
}

impl TimeService for TimeClient {
    fn time_now(&mut self, _badge: Badge) -> Result<u64, Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(TIME_PROTO, time::TIME_NOW, MsgFlags::NONE);
        utcb.set_msg_tag(tag);

        self.0.call(&mut utcb)?;
        Ok(utcb.get_mr(0) as u64)
    }

    fn mono_now(&mut self, _badge: Badge) -> Result<u64, Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(TIME_PROTO, time::MONO_NOW, MsgFlags::NONE);
        utcb.set_msg_tag(tag);

        self.0.call(&mut utcb)?;
        Ok(utcb.get_mr(0) as u64)
    }

    fn sleep(&mut self, _badge: Badge, ms: usize) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(TIME_PROTO, time::SLEEP, MsgFlags::NONE);
        utcb.set_msg_tag(tag);
        utcb.set_mr(0, ms);

        self.0.call(&mut utcb)
    }

    fn adj_time(&mut self, _badge: Badge, absolute_ns: u64, drift_ppb: i64) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(TIME_PROTO, time::ADJ_TIME, MsgFlags::NONE);
        utcb.set_msg_tag(tag);
        set_mrs!(utcb, absolute_ns as usize, drift_ppb as usize);
        self.0.call(&mut utcb)
    }
}
