// Security Capability Flags
pub const SEC_OPEN: u8 = 0;
pub const SEC_WEP: u8 = 1;
pub const SEC_WPA2: u8 = 2;
pub const SEC_WPA3: u8 = 3;

// Connection Status
pub const STATUS_DISCONNECTED: u8 = 0;
pub const STATUS_CONNECTING: u8 = 1;
pub const STATUS_CONNECTED: u8 = 2;
pub const STATUS_FAILED: u8 = 3;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct WifiApInfo {
    pub ssid: [u8; 32],
    pub ssid_len: u8,
    pub bssid: [u8; 6], // MAC Address of AP
    pub security: u8,
    pub channel: u8,
    pub rssi: i8, // Signal strength in dBm
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WifiConnectReq {
    pub ssid: [u8; 32],
    pub ssid_len: u8,
    pub password: [u8; 64],
    pub password_len: u8,
    pub security: u8,
}
