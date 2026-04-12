use crate::cap::{Endpoint, Frame};
use crate::error::Error;
use crate::ipc::Badge;
use crate::protocol::terminal::{
    SeatDesc, TerminalDisplayMode, TerminalUringConfig, VTDesc, WindowSize,
};
use alloc::vec::Vec;

/// VirtualTerminalService is the global manager for all TTYs and Virtual Terminals.
/// It handles multiplexing, multiseat, and administrative tasks.
pub trait VirtualTerminalService {
    /// Create or allocate a new Virtual Terminal.
    /// Returns the VT ID and receives endpoint in recv slot.
    fn create_vt(
        &mut self,
        badge: Badge,
        name: &str,
        recv: crate::cap::CapPtr,
    ) -> Result<(usize, Endpoint), Error>;

    /// Release an existing Virtual Terminal.
    fn destroy_vt(&mut self, badge: Badge, vt_id: usize) -> Result<(), Error>;

    /// List all currently available Virtual Terminals.
    fn list_vts(&mut self, badge: Badge) -> Result<Vec<VTDesc>, Error>;

    /// List all currently managed Seats (input/output groupings).
    fn list_seats(&mut self, badge: Badge) -> Result<Vec<SeatDesc>, Error>;

    /// Switch a designated Seat to focus on a particular Virtual Terminal.
    /// This performs context switching and routes input to the newly active VT.
    fn switch_vt(&mut self, badge: Badge, seat_id: usize, vt_id: usize) -> Result<(), Error>;

    /// Bind a Seat to a specific Virtual Terminal, establishing an affinity.
    /// This is used for multiseat configuration where multiple seats can have their own sets of VTs.
    fn bind_seat(&mut self, badge: Badge, seat_id: usize, vt_id: usize) -> Result<(), Error>;

    /// Open an existing VT endpoint by VT id.
    fn open_vt(
        &mut self,
        badge: Badge,
        vt_id: usize,
        recv: crate::cap::CapPtr,
    ) -> Result<Endpoint, Error>;

    /// Query PTY lock state for a VT id. true means locked.
    fn get_pty_lock(&mut self, badge: Badge, vt_id: usize) -> Result<bool, Error>;

    /// Set PTY lock state for a VT id.
    fn set_pty_lock(&mut self, badge: Badge, vt_id: usize, locked: bool) -> Result<(), Error>;

    /// Add a hardware peripheral (input/output) to a specific seat.
    fn assign_device_to_seat(
        &mut self,
        badge: Badge,
        seat_id: usize,
        device_name: &str,
    ) -> Result<(), Error>;

    /// Remove a hardware peripheral from a specific seat.
    fn revoke_device_from_seat(
        &mut self,
        badge: Badge,
        seat_id: usize,
        device_name: &str,
    ) -> Result<(), Error>;
}

/// TerminalService provides a specific terminal session (VT).
/// It supports zero-copy io_uring communication for high-performance reading and writing.
pub trait TerminalService {
    /// Configure and obtain the io_uring zero-copy interface config.
    /// The manager returns a shared memory frame (Frame capability)
    /// containing the Submission and Completion rings, and the data buffer.
    fn get_uring_config(
        &mut self,
        badge: Badge,
        recv_frame: crate::cap::CapPtr,
    ) -> Result<(Frame, TerminalUringConfig), Error>;

    /// Change the operational mode of the terminal (Raw, Canonical, etc.).
    fn set_mode(&mut self, badge: Badge, mode: u32) -> Result<(), Error>;

    /// Switch between Text (cell-based) and Graphics (pixel/fb) display modes.
    /// In Graphics mode, the terminal may expose its framebuffer via io_uring or shared memory.
    fn set_display_mode(&mut self, badge: Badge, mode: TerminalDisplayMode) -> Result<(), Error>;

    /// Get current window size (rows, cols, pixels).
    fn get_winsize(&mut self, badge: Badge) -> Result<WindowSize, Error>;

    /// Set new window size.
    fn set_winsize(&mut self, badge: Badge, size: WindowSize) -> Result<(), Error>;
}
