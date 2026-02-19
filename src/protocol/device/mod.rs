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

pub mod block;
pub mod fb;
pub mod gpio;
pub mod i2c;
pub mod input;
pub mod net;
pub mod pci;
pub mod sdio;
pub mod spi;
pub mod uart;
pub mod usb;
pub mod wifi;

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

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
    pub compatible: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionMetadata {
    pub parent: u64, // CPtr to the parent RawBlock
    pub start_lba: u64,
    pub num_blocks: u64,
    pub block_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogicDeviceType {
    RawBlock(u64), // Capacity in bytes
    Block(PartitionMetadata),
    Net,
    Fb,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicDeviceDesc {
    pub name: String,
    pub parent_name: String,
    pub dev_type: LogicDeviceType,
}
