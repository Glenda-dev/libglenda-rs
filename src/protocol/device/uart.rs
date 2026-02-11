#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct UartConfig {
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
    pub parity: u8,
}
