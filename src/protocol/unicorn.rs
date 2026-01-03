// Protocol ID
pub const UNICORN_PROTO: usize = 0x0200;

// Management Interface
pub const SCAN_BUS: usize = 1;
pub const LOAD_DRIVER: usize = 2;
pub const LIST_DEVICES: usize = 3;
pub const INIT_MANIFEST: usize = 4; // arg0: frame_cap, arg1: size
pub const GET_DEVICE_BY_NAME: usize = 5; // arg0: name_len

// Driver Interface
pub const GET_INFO: usize = 10;
pub const MAP_MMIO: usize = 11; // arg0: device_id, arg1: mmio_index, arg2: dest_slot
pub const GET_IRQ: usize = 12; // arg0: device_id, arg1: irq_index, arg2: dest_slot
pub const ALLOC_DMA: usize = 13;

// Bus Types
pub const BUS_PCI: usize = 1;
pub const BUS_PLATFORM: usize = 2; // DTB

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
