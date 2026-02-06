use crate::arch::mem::PGSIZE;
use core::fmt::Display;
use serde::{Deserialize, Serialize};
/// Fixed size of the BootInfo page (usually 4KB)
pub const BOOTINFO_SIZE: usize = PGSIZE;

/// Maximum number of untyped memory regions we can describe
pub const MAX_UNTYPED_REGIONS: usize = 4;
pub const MAX_MMIO_REGIONS: usize = 64;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BootInfo {
    //// Initrd memory region
    pub initrd_start: usize,
    pub initrd_size: usize,

    pub version: u32,
    pub build: [u8; 64],
    pub git_hash: [u8; 8],

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
        writeln!(
            f,
            "  Version: {}.{}.{}",
            (self.version >> 24) & 0xFF,
            (self.version >> 16) & 0xFF,
            self.version & 0xFFFF
        )?;
        let build_str =
            core::str::from_utf8(&self.build).unwrap_or("unknown").trim_matches(char::from(0));
        writeln!(f, "  Build: {}", build_str)?;
        let hash_str = core::str::from_utf8(&self.git_hash).unwrap_or("unknown");
        writeln!(f, "  Git Hash: {}", hash_str)?;
        writeln!(f, "  Initrd: start={:#x}, size={:#x}", self.initrd_start, self.initrd_size)?;
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MemoryRange {
    /// Physical address of the memory region
    pub paddr: usize,

    /// Size of the region (2^size_bits bytes)
    pub size: usize,
}

impl Display for MemoryRange {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "MemoryRange {{ paddr: {:#x}, size: {:#x} }}", self.paddr, self.size)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UntypedRegion {
    pub start: usize,
    pub pages: usize,
    pub watermark: usize,
}

impl Display for UntypedRegion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "UntypedRegion {{ start: {:#x}, pages: {}, watermark: {} }}",
            self.start, self.pages, self.watermark
        )
    }
}
