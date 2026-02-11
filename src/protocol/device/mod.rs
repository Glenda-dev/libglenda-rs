pub mod block;
pub mod fb;
pub mod gpio;
pub mod i2c;
pub mod input;
pub mod net;
pub mod pci;
pub mod rng;
pub mod spi;
pub mod timer;
pub mod uart;
pub mod usb;
pub mod wifi;

// Core System
pub const PCI_PROTO: usize = 0x301;
pub const IOMMU_PROTO: usize = 0x302;
pub const UART_PROTO: usize = 0x303;
pub const TIMER_PROTO: usize = 0x30F;

// Storage & Network
pub const BLOCK_PROTO: usize = 0x304;
pub const NET_PROTO: usize = 0x305;
pub const IB_PROTO: usize = 0x306;
pub const WIFI_PROTO: usize = 0x307;

// Human Interface
pub const INPUT_PROTO: usize = 0x308;
pub const FB_PROTO: usize = 0x309;

// Peripheral Bus
pub const USB_PROTO: usize = 0x30A;
pub const SPI_PROTO: usize = 0x30B;
pub const I2C_PROTO: usize = 0x30C;
pub const GPIO_PROTO: usize = 0x30D;
pub const RNG_PROTO: usize = 0x30E;

// Driver Interface
pub const GET_DESC: usize = 1;
pub const GET_MMIO: usize = 2;
pub const MAP_MMIO: usize = 2;
pub const GET_IRQ: usize = 3;
pub const MAP_IRQ: usize = 3;
pub const SCAN_PLATFORM: usize = 4;
pub const FIND_COMPATIBLE: usize = 5;
pub const ALLOC_DMA: usize = 6;
pub const FREE_DMA: usize = 7;

// Bus Types
pub const BUS_PCI: usize = 1;
pub const BUS_PLATFORM: usize = 2; // DTB

use crate::utils::platform::DeviceKind;
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DeviceInfo {
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub revision: u8,
    pub bus: u8,
    pub dev: u8,
    pub func: u8,
    pub irq_line: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: usize,
    pub dev_type: DeviceKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceNode {
    pub id: usize,
    pub compatible: String,
    pub base_addr: usize,
    pub size: usize,
    pub irq: u32,
    pub kind: DeviceKind,
    pub parent_id: Option<usize>,
    pub children: Vec<usize>,
}
