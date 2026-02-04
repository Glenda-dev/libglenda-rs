//! GPIO Protocol (0x308)

pub const SET_MODE: usize = 1; // arg0: pin, arg1: mode
pub const WRITE: usize = 2;    // arg0: pin, arg1: value
pub const READ: usize = 3;     // arg0: pin, ret: value

pub const MODE_INPUT: u8 = 0;
pub const MODE_OUTPUT: u8 = 1;
pub const MODE_ALT: u8 = 2;
