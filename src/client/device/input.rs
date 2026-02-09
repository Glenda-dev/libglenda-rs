use crate::cap::Endpoint;
use crate::interface::device::InputDevice;
use crate::ipc::{MsgFlags, MsgTag, UTCB};
use crate::protocol::device::input::InputEvent;
use crate::protocol::device::{INPUT_PROTO, input};

pub struct InputClient {
    endpoint: Endpoint,
}

impl InputClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }
}

impl InputDevice for InputClient {
    fn poll_event(&mut self) -> Option<InputEvent> {
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(INPUT_PROTO, input::READ_EVENT, MsgFlags::NONE);
        utcb.set_msg_tag(tag);
        if self.endpoint.call(&mut utcb).is_ok() {
            unsafe { utcb.read_obj::<InputEvent>().ok() }
        } else {
            None
        }
    }
}
