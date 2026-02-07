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
        let utcb = unsafe { UTCB::get() };
        let tag = MsgTag::new(INPUT_PROTO, input::READ_EVENT, MsgFlags::NONE);

        if self.endpoint.call(tag).is_ok() {
            let len = utcb.size;
            if len >= core::mem::size_of::<InputEvent>() {
                // Safety: InputEvent is repr(C) and POD
                let mut event = InputEvent::default();
                unsafe {
                    let src = utcb.ipc_buffer.as_ptr();
                    let dst = &mut event as *mut InputEvent as *mut u8;
                    core::ptr::copy_nonoverlapping(src, dst, core::mem::size_of::<InputEvent>());
                }
                Some(event)
            } else {
                None
            }
        } else {
            None
        }
    }
}
