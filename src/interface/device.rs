use crate::error::Error;
use crate::manager::device::DeviceNode;
use crate::utils::platform::PlatformInfo;

/// DeviceService provides hardware discovery and management.
pub trait DeviceService {
    fn scan_platform(&mut self, info: &PlatformInfo);
    fn get_node(&self, id: usize) -> Option<&DeviceNode>;
    fn find_compatible(&self, compat: &str) -> Option<&DeviceNode>;
}

/// PciService provides PCI bus access.
pub trait PciService {
    fn read_config(&self, bus: u8, dev: u8, func: u8, offset: usize, size: usize) -> u32;
    fn write_config(&mut self, bus: u8, dev: u8, func: u8, offset: usize, value: u32, size: usize);
    fn scan(&mut self, dev_mgr: &mut dyn DeviceService);
}

/// DmaService provides DMA-safe memory allocation.
pub trait DmaService {
    fn alloc_dma(&mut self, size: usize) -> Result<usize, Error>;
    fn free_dma(&mut self, paddr: usize, size: usize);
}
