use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// Terminal Display Modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminalDisplayMode {
    /// Classic text mode (cell-based)
    Text,
    /// Graphics mode (pixel-based/framebuffer)
    Graphics,
    /// Bridge (raw bridge)
    Bridge,
}

/// Window Size information (rows/cols for text, pixels for graphics)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WindowSize {
    pub rows: u16,
    pub cols: u16,
    pub xpixel: u16,
    pub ypixel: u16,
}

/// Information about a Virtual Terminal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VTDesc {
    pub id: usize,
    pub name: String,
    pub mode: TerminalDisplayMode,
    pub seat_ids: Vec<usize>,
}

/// Information about a Seat (Input/Display binding)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeatDesc {
    pub id: usize,
    pub name: String,
    pub active_vt: Option<usize>,
    pub input_devices: Vec<String>,
    pub output_devices: Vec<String>,
}

/// Dynamic input events for graphics mode (pixel-based VT)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TerminalInputEvent {
    KeyDown { keycode: u32 },
    KeyUp { keycode: u32 },
    MouseMove { x: i32, y: i32 },
    MouseDown { button: u32 },
    MouseUp { button: u32 },
    Scroll { dx: i32, dy: i32 },
}

/// io_uring configuration for zero-copy terminal communication
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TerminalUringConfig {
    /// Shared memory size for rings and buffer
    pub size: usize,
    /// Number of SQ entries
    pub sq_entries: u32,
    /// Number of CQ entries
    pub cq_entries: u32,
    /// Offset of the SQ ring in the shared memory
    pub sq_offset: usize,
    /// Offset of the CQ ring in the shared memory
    pub cq_offset: usize,
    /// Offset of the data buffer in the shared memory
    pub buf_offset: usize,
}

// Additional methods for terminal I/O
pub const TERM_PUT_STR: usize = 0x01;
pub const TERM_GET_STR: usize = 0x02;
pub const TERM_GET_CHAR: usize = 0x03;
pub const TERM_PUT_CHAR: usize = 0x04;
pub const TERM_POLL_READ: usize = 0x05;

// VTS (Virtual Terminal Service) Methods (Manager)
pub const VTS_ALLOC_VT: usize = 0x21;
pub const VTS_FREE_VT: usize = 0x22;
pub const VTS_LIST_VTS: usize = 0x23;
pub const VTS_LIST_SEATS: usize = 0x24;
pub const VTS_SWITCH_VT: usize = 0x25;
pub const VTS_BIND_SEAT: usize = 0x26;
pub const VTS_SET_EXCLUSIVE: usize = 0x27;
pub const VTS_OPEN_VT: usize = 0x28;
pub const VTS_GET_PTY_LOCK: usize = 0x29;
pub const VTS_SET_PTY_LOCK: usize = 0x2A;

pub const SEAT_BIND_DEVICE: usize = 0x30;
pub const SEAT_UNBIND_DEVICE: usize = 0x31;

// Terminal Service Methods (Individual VT session)
pub const TERM_GET_URING: usize = 0x11;
pub const TERM_SET_MODE: usize = 0x12;
pub const TERM_GET_WINSIZE: usize = 0x13;
pub const TERM_SET_WINSIZE: usize = 0x14;
pub const TERM_SET_DISPLAY: usize = 0x15;
pub const TERM_GET_TERMIOS: usize = 0x16;
pub const TERM_SET_TERMIOS: usize = 0x17;
pub const TERM_GET_PGRP: usize = 0x18;
pub const TERM_SET_PGRP: usize = 0x19;

// UART IOCTL split
pub const TERM_SET_BAUD: usize = 0x31;
pub const TERM_SET_LCR: usize = 0x32;
pub const TERM_SET_FCR: usize = 0x33;
pub const TERM_GET_BAUD: usize = 0x34;

/// Character stream constants
pub const CTRL_C: u8 = 0x03;
pub const CTRL_D: u8 = 0x04;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterStream {
    pub buffer: alloc::vec::Vec<u8>,
}
