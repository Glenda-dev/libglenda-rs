// Device Interface
pub const GET_MMIO: usize = 1;
pub const GET_IRQ: usize = 2;
pub const SCAN_PLATFORM: usize = 3;
pub const REPORT: usize = 4;
pub const UPDATE: usize = 5;
pub const REGISTER_LOGIC: usize = 6;
pub const ALLOC_LOGIC: usize = 7;
pub const QUERY: usize = 8;
pub const GET_DESC: usize = 9;
pub const HOOK: usize = 10;
pub const UNHOOK: usize = 11;
pub const GET_LOGIC_DESC: usize = 12;

/// Async notification for hook events
pub const NOTIFY_HOOK: usize = 0x20;

pub mod block;
pub mod fb;
pub mod gpio;
pub mod i2c;
pub mod input;
pub mod net;
pub mod pci;
pub mod sdio;
pub mod spi;
pub mod thermal;
pub mod uart;
pub mod usb;
pub mod wifi;

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookTarget {
    Endpoint(u64),
    Type(LogicDeviceType),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceNotification {
    Registered(u64, LogicDeviceDesc), // (badge, desc)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceDesc {
    pub name: String,
    pub compatible: Vec<String>,
    pub mmio: Vec<MMIORegion>,
    pub irq: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MMIORegion {
    pub base_addr: usize,
    pub size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceDescNode {
    pub parent: usize,
    pub desc: DeviceDesc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceQuery {
    pub name: Option<String>,
    pub compatible: Vec<String>,
    pub dev_type: Option<u32>, // 0 for any, others match specific LogicDeviceType discriminant or similar
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogicDeviceType {
    RawBlock(u64), // Capacity in bytes
    Block(u64),    // Capacity in blocks (sectors)
    Net,
    Fb,
    Uart,
    Input,
    Gpio,
    Platform,
    Thermal,
    Battery,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicDeviceDesc {
    pub name: String,
    pub dev_type: LogicDeviceType,
    pub parent_name: String,
    pub badge: Option<u64>, // Badge meant for the hardware driver to distinguish logical units
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocLogicRequest {
    pub dev_type: u32,
    pub criteria: String,
}
