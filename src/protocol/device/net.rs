//! Ethernet Device Protocol

/// Get MAC Address
pub const GET_MAC: usize = 0x1;
/// Send Packet
/// Args: length
pub const SEND: usize = 0x2;
/// Receive Packet
pub const RECV: usize = 0x3;

// Shared Memory Ring Buffer Setup?
pub const SETUP_RX_RING: usize = 0x10;
pub const SETUP_TX_RING: usize = 0x11;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct MacAddress {
    pub octets: [u8; 6],
}
