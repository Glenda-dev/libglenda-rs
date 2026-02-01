use crate::cap::{CNode, CapPtr, CapType, Frame};
use crate::error::Error;
use crate::manager::device::DeviceNode;
use crate::mem::Perms;
use crate::utils::platform::PlatformInfo;

/// ResourceManager is responsible for allocating kernel objects from untyped memory.
pub trait IResourceManager {
    fn alloc(
        &mut self,
        obj_type: CapType,
        flags: usize,
        dest_cnode: CNode,
        dest_slot: CapPtr,
    ) -> Result<(), Error>;
}

/// SlotManager is responsible for managing capability slots.
pub trait ISlotManager {
    fn alloc(&mut self, objects: &mut dyn IResourceManager) -> Result<CapPtr, Error>;
}

/// VSpaceManager is responsible for managing virtual memory mappings.
pub trait IVSpaceManager {
    fn map_frame(
        &mut self,
        frame: Frame,
        vaddr: usize,
        perms: Perms,
        pages: usize,
        objects: &mut dyn IResourceManager,
        slots: &mut dyn ISlotManager,
        dest_cnode: CNode,
    ) -> Result<(), Error>;

    fn unmap(&mut self, vaddr: usize, pages: usize) -> Result<(), Error>;
}

/// ProcessManager provides high-level process control.
pub trait IProcessManager {
    fn spawn(&mut self, name: &str) -> Result<usize, Error>;
    fn fork(&mut self, pid: usize) -> Result<usize, Error>;
    fn exit(&mut self, pid: usize, code: usize) -> Result<(), Error>;
    fn load_image(&mut self, pid: usize, elf_data: &[u8]) -> Result<(usize, usize), Error>;
}

/// MemoryService provides system-level memory operations for processes.
pub trait IMemoryService {
    fn brk(&mut self, pid: usize, incr: isize) -> Result<usize, Error>;
    fn mmap(&mut self, pid: usize, args: &[usize]) -> Result<usize, Error>;
    fn munmap(&mut self, pid: usize, args: &[usize]) -> Result<(), Error>;
}

/// DeviceManager provides hardware discovery and management.
pub trait IDeviceManager {
    fn scan_platform(&mut self, info: &PlatformInfo);
    fn get_node(&self, id: usize) -> Option<&DeviceNode>;
    fn find_compatible(&self, compat: &str) -> Option<&DeviceNode>;
}

/// PciService provides PCI bus access.
pub trait IPciService {
    fn read_config(&self, bus: u8, dev: u8, func: u8, offset: usize, size: usize) -> u32;
    fn write_config(&mut self, bus: u8, dev: u8, func: u8, offset: usize, value: u32, size: usize);
    fn scan(&mut self, dev_mgr: &mut dyn IDeviceManager);
}

/// DmaService provides DMA-safe memory allocation.
pub trait IDmaService {
    fn alloc_dma(&mut self, size: usize) -> Result<usize, Error>;
    fn free_dma(&mut self, paddr: usize, size: usize);
}
