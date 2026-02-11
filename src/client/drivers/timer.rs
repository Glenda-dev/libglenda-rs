use crate::cap::Endpoint;
use crate::error::Error;
use crate::interface::TimerDriver;
use crate::ipc::{MsgFlags, MsgTag, UTCB};
use crate::protocol::drivers::{TIMER_PROTO, timer};
use crate::set_mrs;

pub struct TimerClient(Endpoint);

impl TimerClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self(endpoint)
    }
}

impl TimerDriver for TimerClient {
    fn get_time(&self) -> u64 {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(TIMER_PROTO, timer::GET_TIME, MsgFlags::NONE);
        utcb.set_msg_tag(tag);

        if self.0.call(&mut utcb).is_ok() {
            let low = utcb.get_mr(0) as u64;
            let high = utcb.get_mr(1) as u64;
            (high << 32) | low
        } else {
            0
        }
    }

    fn set_alarm(&mut self, timestamp: u64) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(TIMER_PROTO, timer::SET_ALARM, MsgFlags::NONE);
        utcb.set_msg_tag(tag);
        set_mrs!(utcb, (timestamp & 0xFFFFFFFF) as usize, (timestamp >> 32) as usize);
        self.0.call(&mut utcb)
    }

    fn stop_alarm(&mut self) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(TIMER_PROTO, timer::STOP_ALARM, MsgFlags::NONE);
        utcb.set_msg_tag(tag);
        self.0.call(&mut utcb)
    }
}
