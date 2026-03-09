pub const READ_EVENT: usize = 0x1;
pub const SETUP_URING: usize = 0x2;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct InputEvent {
    pub kind: u8,
    pub code: u16,
    pub value: i32,
    pub timestamp: usize,
}

pub const INPUT_OP_READ: u8 = 0x1;
