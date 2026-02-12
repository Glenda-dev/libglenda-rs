// Device Interface
pub const GET_DESC: usize = 1;
pub const GET_MMIO: usize = 2;
pub const GET_IRQ: usize = 3;
pub const SCAN_PLATFORM: usize = 4;

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
use num_enum::FromPrimitive;
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

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, FromPrimitive)]
pub enum DeviceKind {
    #[num_enum(default)]
    Unknown = 0,
    Uart = 1,
    Intc = 2, // 中断控制器 (PLIC/GIC)
    Timer = 3,
    Virtio = 4,
    PciHost = 5,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BusType {
    System = 0, // 系统主总线 (System Bus)
    Pci = 1,
    Usb = 2,
    Platform = 3, // 简单的内存映射设备
}
