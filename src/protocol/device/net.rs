//! Ethernet Device Protocol


/// Get MAC Address
pub const GET_MAC: usize = 1;
/// Send Packet
/// Args: length
pub const SEND: usize = 2;
/// Receive Packet
pub const RECV: usize = 3;

// Shared Memory Ring Buffer Setup?
pub const SETUP_RX_RING: usize = 4;
pub const SETUP_TX_RING: usize = 5;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct MacAddress {
    pub octets: [u8; 6],
}
