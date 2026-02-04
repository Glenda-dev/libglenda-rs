//! IOMMU Protocol (0x30C)

pub const MAP: usize = 1; // arg0: iova, arg1: paddr, arg2: size, arg3: flags
pub const UNMAP: usize = 2; // arg0: iova, arg1: size
pub const FLUSH: usize = 3; // Globally flush IOTLB

// Permission Flags
pub const IOMMU_READ: u32 = 1 << 0;
pub const IOMMU_WRITE: u32 = 1 << 1;
pub const IOMMU_EXEC: u32 = 1 << 2;
pub const IOMMU_CACHE: u32 = 1 << 3; // Coherent
