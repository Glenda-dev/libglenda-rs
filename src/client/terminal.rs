use crate::cap::{CapPtr, Endpoint, Page};
use crate::error::Error;
use crate::interface::terminal::{TerminalService, VirtualTerminalService};
use crate::io::uring::IoUringGeneric;
use crate::ipc::{Badge, MsgFlags, MsgTag, UTCB};
use crate::protocol;
use crate::protocol::terminal::{
    SeatDesc, TerminalDisplayMode, TerminalInputEvent, TerminalSessionMode, TerminalUringConfig,
    VTDesc, WindowSize,
};

/// TerminalClient represents a connection to a specific virtual terminal.
#[derive(Clone, Copy, Debug)]
pub struct TerminalClient {
    endpoint: Endpoint,
    config: Option<TerminalUringConfig>,
    frame: Option<Page>,
}

impl TerminalClient {
    /// Create a new terminal client wrapper around an endpoint.
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint, config: None, frame: None }
    }

    pub fn endpoint(&self) -> Endpoint {
        self.endpoint
    }

    pub fn set_session_mode(&mut self, mode: TerminalSessionMode) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(MsgTag::new(
            protocol::TERMINAL_PROTO,
            protocol::terminal::TERM_STREAM_SET_MODE,
            MsgFlags::NONE,
        ));
        utcb.set_mr(
            0,
            match mode {
                TerminalSessionMode::ByteStream => 0,
                TerminalSessionMode::Native => 1,
            },
        );
        self.endpoint.call(&mut utcb)
    }

    pub fn stream_read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(MsgTag::new(
            protocol::TERMINAL_PROTO,
            protocol::terminal::TERM_STREAM_READ,
            MsgFlags::HAS_BUFFER,
        ));
        utcb.set_mr(0, buf.len());
        self.endpoint.call(&mut utcb)?;
        let read_len = core::cmp::min(utcb.get_mr(0), core::cmp::min(buf.len(), utcb.buffer().len()));
        if read_len > 0 {
            buf[..read_len].copy_from_slice(&utcb.buffer()[..read_len]);
        }
        Ok(read_len)
    }

    pub fn stream_write(&mut self, data: &[u8]) -> Result<usize, Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(MsgTag::new(
            protocol::TERMINAL_PROTO,
            protocol::terminal::TERM_STREAM_WRITE,
            MsgFlags::HAS_BUFFER,
        ));
        let copied = utcb.write(data);
        self.endpoint.call(&mut utcb)?;
        Ok(if utcb.get_mr(0) > 0 { core::cmp::min(utcb.get_mr(0), copied) } else { copied })
    }

    pub fn stream_poll_readable(&mut self) -> Result<bool, Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(MsgTag::new(
            protocol::TERMINAL_PROTO,
            protocol::terminal::TERM_STREAM_POLL,
            MsgFlags::NONE,
        ));
        self.endpoint.call(&mut utcb)?;
        Ok(utcb.get_mr(0) != 0)
    }

    pub fn native_poll_event(&mut self) -> Result<bool, Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(MsgTag::new(
            protocol::TERMINAL_PROTO,
            protocol::terminal::TERM_NATIVE_POLL_EVENT,
            MsgFlags::NONE,
        ));
        self.endpoint.call(&mut utcb)?;
        Ok(utcb.get_mr(0) != 0)
    }

    pub fn native_get_event(&mut self) -> Result<TerminalInputEvent, Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(MsgTag::new(
            protocol::TERMINAL_PROTO,
            protocol::terminal::TERM_NATIVE_GET_EVENT,
            MsgFlags::NONE,
        ));
        self.endpoint.call(&mut utcb)?;
        unsafe { utcb.read_postcard() }
    }

    pub fn native_put_text(&mut self, text: &str) -> Result<usize, Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(MsgTag::new(
            protocol::TERMINAL_PROTO,
            protocol::terminal::TERM_NATIVE_PUT_TEXT,
            MsgFlags::HAS_BUFFER,
        ));
        let copied = utcb.write(text.as_bytes());
        self.endpoint.call(&mut utcb)?;
        Ok(if utcb.get_mr(0) > 0 { core::cmp::min(utcb.get_mr(0), copied) } else { copied })
    }

    /// Connect to the terminal service and initialize the high-performance channel.
    /// This method fetches the io_uring configuration and prepares the metadata.
    /// `recv_frame` is the capability slot to receive the shared memory frame.
    pub fn connect(&mut self, _badge: Badge, recv_frame: CapPtr) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(
            protocol::TERMINAL_PROTO,
            protocol::terminal::TERM_GET_URING,
            MsgFlags::NONE,
        );
        utcb.set_recv_window(recv_frame);
        utcb.set_msg_tag(tag);

        self.endpoint.call(&mut utcb)?;

        let config: TerminalUringConfig = unsafe { utcb.read_postcard()? };
        self.config = Some(config);
        self.frame = Some(Page::from(recv_frame));

        Ok(())
    }

    /// Get the io_uring configuration.
    pub fn config(&self) -> Option<TerminalUringConfig> {
        self.config
    }

    /// Get the shared memory frame.
    pub fn frame(&self) -> Option<Page> {
        self.frame
    }

    /// Get the io_uring instance if the terminal is connected.
    /// `base` must be the virtual address where the frame was mapped.
    pub unsafe fn get_uring(&self, base: *mut u8) -> Option<IoUringGeneric<'_>> {
        self.config.as_ref().map(|_config| {
            // Using a default entries from config if available or 32 as fallback
            unsafe { IoUringGeneric::new(base, 32) }
        })
    }
}

impl TerminalService for TerminalClient {
    fn get_uring_config(
        &mut self,
        _badge: Badge,
        _recv_frame: CapPtr,
    ) -> Result<(Page, TerminalUringConfig), Error> {
        if let (Some(frame), Some(config)) = (self.frame, self.config) {
            return Ok((frame, config));
        }
        Err(Error::InvalidCapability)
    }

    fn set_mode(&mut self, _badge: Badge, mode: u32) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(
            protocol::TERMINAL_PROTO,
            protocol::terminal::TERM_SET_MODE,
            MsgFlags::NONE,
        );
        utcb.set_mr(0, mode as usize);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }

    fn set_display_mode(&mut self, _badge: Badge, mode: TerminalDisplayMode) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(
            protocol::TERMINAL_PROTO,
            protocol::terminal::TERM_SET_DISPLAY,
            MsgFlags::HAS_BUFFER,
        );
        unsafe {
            utcb.write_postcard(&mode)?;
        }
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }

    fn get_winsize(&mut self, _badge: Badge) -> Result<WindowSize, Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(
            protocol::TERMINAL_PROTO,
            protocol::terminal::TERM_GET_WINSIZE,
            MsgFlags::NONE,
        );
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        unsafe { utcb.read_postcard() }
    }

    fn set_winsize(&mut self, _badge: Badge, size: WindowSize) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(
            protocol::TERMINAL_PROTO,
            protocol::terminal::TERM_SET_WINSIZE,
            MsgFlags::HAS_BUFFER,
        );
        unsafe {
            utcb.write_postcard(&size)?;
        }
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }
}

impl core::fmt::Write for TerminalClient {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let mut utcb = unsafe { UTCB::new() };
        // Basic fallback: use generic Write/Put str if uring is not used or for simplicity
        // In a real high-performance case, we would use the uring buffer.
        let tag = MsgTag::new(
            protocol::TERMINAL_PROTO,
            protocol::terminal::TERM_PUT_STR,
            MsgFlags::HAS_BUFFER,
        );
        if utcb.write(s.as_bytes()) == 0 {
            return Err(core::fmt::Error);
        }
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb).map_err(|_| core::fmt::Error)
    }
}

pub struct VirtualTerminalClient {
    endpoint: Endpoint,
}

impl VirtualTerminalClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }
}

impl VirtualTerminalService for VirtualTerminalClient {
    fn create_vt(
        &mut self,
        _badge: Badge,
        name: &str,
        recv: CapPtr,
    ) -> Result<(usize, Endpoint), Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(
            protocol::TERMINAL_PROTO,
            protocol::terminal::VTS_ALLOC_VT,
            MsgFlags::HAS_BUFFER,
        );
        utcb.set_recv_window(recv);
        unsafe {
            utcb.write_str(name)?;
        }
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;

        let vt_id = utcb.get_mr(0);
        Ok((vt_id, Endpoint::from(recv)))
    }

    fn destroy_vt(&mut self, _badge: Badge, vt_id: usize) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag =
            MsgTag::new(protocol::TERMINAL_PROTO, protocol::terminal::VTS_FREE_VT, MsgFlags::NONE);
        utcb.set_mr(0, vt_id);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }

    fn list_vts(&mut self, _badge: Badge) -> Result<alloc::vec::Vec<VTDesc>, Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag =
            MsgTag::new(protocol::TERMINAL_PROTO, protocol::terminal::VTS_LIST_VTS, MsgFlags::NONE);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        unsafe { utcb.read_postcard() }
    }

    fn list_seats(&mut self, _badge: Badge) -> Result<alloc::vec::Vec<SeatDesc>, Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(
            protocol::TERMINAL_PROTO,
            protocol::terminal::VTS_LIST_SEATS,
            MsgFlags::NONE,
        );
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        unsafe { utcb.read_postcard() }
    }

    fn switch_vt(&mut self, _badge: Badge, seat_id: usize, vt_id: usize) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(
            protocol::TERMINAL_PROTO,
            protocol::terminal::VTS_SWITCH_VT,
            MsgFlags::NONE,
        );
        utcb.set_mr(0, seat_id);
        utcb.set_mr(1, vt_id);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }

    fn bind_seat(&mut self, _badge: Badge, seat_id: usize, vt_id: usize) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(
            protocol::TERMINAL_PROTO,
            protocol::terminal::VTS_BIND_SEAT,
            MsgFlags::NONE,
        );
        utcb.set_mr(0, seat_id);
        utcb.set_mr(1, vt_id);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }

    fn open_vt(&mut self, _badge: Badge, vt_id: usize, recv: CapPtr) -> Result<Endpoint, Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag =
            MsgTag::new(protocol::TERMINAL_PROTO, protocol::terminal::VTS_OPEN_VT, MsgFlags::NONE);
        utcb.set_recv_window(recv);
        utcb.set_mr(0, vt_id);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        Ok(Endpoint::from(recv))
    }

    fn get_pty_lock(&mut self, _badge: Badge, vt_id: usize) -> Result<bool, Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(
            protocol::TERMINAL_PROTO,
            protocol::terminal::VTS_GET_PTY_LOCK,
            MsgFlags::NONE,
        );
        utcb.set_mr(0, vt_id);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        Ok(utcb.get_mr(0) != 0)
    }

    fn set_pty_lock(&mut self, _badge: Badge, vt_id: usize, locked: bool) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(
            protocol::TERMINAL_PROTO,
            protocol::terminal::VTS_SET_PTY_LOCK,
            MsgFlags::NONE,
        );
        utcb.set_mr(0, vt_id);
        utcb.set_mr(1, usize::from(locked));
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }

    fn assign_device_to_seat(
        &mut self,
        _badge: Badge,
        seat_id: usize,
        device_name: &str,
    ) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(
            protocol::TERMINAL_PROTO,
            protocol::terminal::SEAT_BIND_DEVICE,
            MsgFlags::HAS_BUFFER,
        );
        utcb.set_mr(0, seat_id);
        unsafe {
            utcb.write_str(device_name)?;
        }
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }

    fn revoke_device_from_seat(
        &mut self,
        _badge: Badge,
        seat_id: usize,
        device_name: &str,
    ) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(
            protocol::TERMINAL_PROTO,
            protocol::terminal::SEAT_UNBIND_DEVICE,
            MsgFlags::HAS_BUFFER,
        );
        utcb.set_mr(0, seat_id);
        unsafe {
            utcb.write_str(device_name)?;
        }
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }
}
