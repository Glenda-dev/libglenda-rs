use crate::arch::mem::PGSIZE;
use core::fmt::Display;

/// Fixed size of the BootInfo page (usually 4KB)
pub const BOOTINFO_SIZE: usize = PGSIZE;

/// Maximum number of untyped memory regions we can describe
pub const MAX_UNTYPED_REGIONS: usize = 4;
pub const MAX_MMIO_REGIONS: usize = 64;

#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformType {
    NULL = 0,
    ACPI = 1,
    DTB = 2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BootInfo {
    //// Initrd memory region
    pub initrd_paddr: usize,
    pub initrd_size: usize,

    pub platform_type: PlatformType,
    pub addr: usize,
    pub size: usize,

    pub version: u32,
    pub build: [u8; 64],
    pub git_hash: [u8; 8],

    /// Number of valid entries in `untyped_list`
    pub untyped_count: usize,

    /// List of untyped memory regions available to the system
    /// The i-th entry here corresponds to the capability at `untyped.start + i`
    pub untyped_list: [UntypedRegion; MAX_UNTYPED_REGIONS],

    pub cmdline: [u8; 256],
}

impl BootInfo {
    pub fn new() -> Self {
        Self {
            untyped_count: 0,
            untyped_list: [UntypedRegion::empty(); MAX_UNTYPED_REGIONS],
            initrd_paddr: 0,
            initrd_size: 0,
            platform_type: PlatformType::NULL,
            addr: 0,
            size: 0,
            version: 0,
            build: [0; 64],
            git_hash: [0; 8],
            cmdline: [0; 256],
        }
    }

    pub fn major(&self) -> u32 {
        (self.version >> 24) & 0xFF
    }

    pub fn minor(&self) -> u32 {
        (self.version >> 16) & 0xFF
    }

    pub fn patch(&self) -> u32 {
        self.version & 0xFFFF
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct UntypedRegion {
    pub start: usize,
    pub pages: usize,
    pub watermark: usize,
}

impl UntypedRegion {
    pub fn empty() -> Self {
        Self { start: 0, pages: 0, watermark: 0 }
    }
}

impl Display for BootInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "BootInfo {{")?;

        // Version & Build
        let build_str =
            core::str::from_utf8(&self.build).unwrap_or("Invalid UTF-8").trim_matches('\0');
        let git_hash_str =
            core::str::from_utf8(&self.git_hash).unwrap_or("Invalid UTF-8").trim_matches('\0');
        writeln!(f, "  Version: {}.{}.{}", self.major(), self.minor(), self.patch())?;
        writeln!(f, "  Build: {}", build_str)?;
        writeln!(f, "  Git Hash: {}", git_hash_str)?;

        // Cmdline
        let cmdline_str =
            core::str::from_utf8(&self.cmdline).unwrap_or("Invalid UTF-8").trim_matches('\0');
        writeln!(f, "  Cmdline: {}", cmdline_str)?;

        // Platform
        writeln!(
            f,
            "  Platform: {:?} (Addr: {:#x}, Size: {:#x})",
            self.platform_type, self.addr, self.size
        )?;

        // Initrd
        writeln!(f, "  Initrd: Addr {:#x}, Size {:#x}", self.initrd_paddr, self.initrd_size)?;

        // Untyped
        writeln!(f, "  Untyped Regions ({}):", self.untyped_count)?;
        for i in 0..self.untyped_count {
            if i >= MAX_UNTYPED_REGIONS {
                break;
            }
            let region = &self.untyped_list[i];
            writeln!(
                f,
                "    [{}] Start: {:#x}, Pages: {}, Watermark: {:#x}",
                i, region.start, region.pages, region.watermark
            )?;
        }

        write!(f, "}}")
    }
}
