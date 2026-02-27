// Device Interface
pub const GET_MMIO: usize = 0x01;
pub const GET_IRQ: usize = 0x02;
pub const SCAN_PLATFORM: usize = 0x03;
pub const REPORT: usize = 0x04;
pub const UPDATE: usize = 0x05;
pub const QUERY: usize = 0x06;
pub const GET_DESC: usize = 0x07;

pub const REGISTER_LOGIC: usize = 0x10;
pub const ALLOC_LOGIC: usize = 0x11;
pub const GET_LOGIC_DESC: usize = 0x12;

pub const HOOK: usize = 0x20;
pub const UNHOOK: usize = 0x21;

use alloc::string::String;
use alloc::vec::Vec;
use num_enum::FromPrimitive;
use serde::{Deserialize, Serialize};

pub const NOTIFY_HOOK: usize = 1 << 35;

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
    pub dev_type: Option<LogicDeviceType>, // Change dev_type from u32 to Option<LogicDeviceType>
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, FromPrimitive,
)]
#[repr(u16)]
pub enum LogicDeviceType {
    #[default]
    Generic = 0,
    Block = 1,
    Volume = 2,
    Net = 3,
    Fb = 4,
    Uart = 5,
    Timer = 6,
    Input = 7,
    Gpio = 8,
    Platform = 9,
    Thermal = 10,
    Battery = 11,
    Terminal = 12,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicDeviceDesc {
    pub name: String,
    pub dev_type: LogicDeviceType,
    pub parent_name: String,
    pub badge: Option<usize>, // Badge meant for the hardware driver to distinguish logical units
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocLogicRequest {
    pub dev_type: LogicDeviceType,
    pub criteria: String,
}
