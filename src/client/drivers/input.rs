use crate::cap::Endpoint;
use crate::interface::drivers::InputDriver;
use crate::ipc::{MsgFlags, MsgTag, UTCB};
use crate::protocol::device::input::InputEvent;
use crate::protocol::drivers::{INPUT_PROTO, input};

pub struct InputClient {
    endpoint: Endpoint,
}

impl InputClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }
}

impl InputDriver for InputClient {
    fn poll_event(&mut self) -> Option<InputEvent> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(INPUT_PROTO, input::READ_EVENT, MsgFlags::NONE);
        utcb.set_msg_tag(tag);
        if self.endpoint.call(&mut utcb).is_ok() {
            unsafe { utcb.read_obj::<InputEvent>().ok() }
        } else {
            None
        }
    }
}
