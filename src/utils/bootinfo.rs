use crate::arch::mem::PGSIZE;
use core::fmt::Display;

/// Magic number to verify BootInfo validity: 'GLENDA_B'
pub const BOOTINFO_MAGIC: u32 = 0x99999999;

/// Fixed size of the BootInfo page (usually 4KB)
pub const BOOTINFO_SIZE: usize = PGSIZE;

/// Maximum number of untyped memory regions we can describe
pub const MAX_UNTYPED_REGIONS: usize = 8;
pub const MAX_MMIO_REGIONS: usize = 64;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BootInfo {
    /// Magic number for verification
    pub magic: u32,

    //// Initrd memory region
    pub initrd_start: usize,
    pub initrd_size: usize,

    /// Number of valid entries in `untyped_list`
    pub untyped_count: usize,

    /// List of untyped memory regions available to the system
    /// The i-th entry here corresponds to the capability at `untyped.start + i`
    pub untyped_list: [UntypedRegion; MAX_UNTYPED_REGIONS],

    /// Number of valid entries in `untyped_list`
    pub mmio_count: usize,

    /// List of untyped memory regions available to the system
    /// The i-th entry here corresponds to the capability at `untyped.start + i`
    pub mmio_list: [MemoryRange; MAX_MMIO_REGIONS],

    /// Number of IRQs available
    pub irq_count: usize,
}

impl Display for BootInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "BootInfo:")?;
        writeln!(f, "  Magic: 0x{:08x}", self.magic)?;
        writeln!(f, "  Initrd: start=0x{:x}, size=0x{:x}", self.initrd_start, self.initrd_size)?;
        writeln!(f, "  Untyped Regions (count={}):", self.untyped_count)?;
        for i in 0..self.untyped_count {
            writeln!(f, "    {}", self.untyped_list[i])?;
        }
        writeln!(f, "  MMIO Regions (count={}):", self.mmio_count)?;
        for i in 0..self.mmio_count {
            writeln!(f, "    {}", self.mmio_list[i])?;
        }
        writeln!(f, "  IRQ Count: {}", self.irq_count)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryRange {
    /// Physical address of the memory region
    pub paddr: usize,

    /// Size of the region (2^size_bits bytes)
    pub size: usize,
}

impl Display for MemoryRange {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "MemoryRange {{ paddr: 0x{:x}, size: 0x{:x} }}", self.paddr, self.size)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UntypedRegion {
    pub start: usize,
    pub pages: usize,
    pub watermark: usize,
}

impl Display for UntypedRegion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "UntypedRegion {{ start: 0x{:x}, pages: {}, watermark: {} }}",
            self.start, self.pages, self.watermark
        )
    }
}
