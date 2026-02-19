use crate::cap::Frame;
use crate::cap::VSpace;
use crate::error::Error;
use crate::mem::Perms;
use core::slice;

/// A shared memory region.
#[derive(Debug, Clone, Copy)]
pub struct SharedMemory {
    frame: Frame,
    vaddr: usize,
    size: usize,
}

impl SharedMemory {
    /// Create a SharedMemory instance from an existing Frame.
    pub const fn from_frame(frame: Frame, vaddr: usize, size: usize) -> Self {
        Self { frame, vaddr, size }
    }

    /// Map the shared memory into a VSpace.
    pub fn map(&self, vspace: &VSpace, perms: Perms) -> Result<(), Error> {
        vspace.map(self.frame, self.vaddr, perms)
    }

    /// Get the virtual address of the shared memory.
    pub fn vaddr(&self) -> usize {
        self.vaddr
    }

    /// Get the size of the shared memory.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Get the capability to the underlying Frame.
    pub fn frame(&self) -> Frame {
        self.frame
    }

    /// Get a byte slice of the shared memory.
    ///
    /// # Safety
    /// The memory must be mapped and valid at the current vaddr.
    pub unsafe fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.vaddr as *const u8, self.size) }
    }

    /// Get a mutable byte slice of the shared memory.
    ///
    /// # Safety
    /// The memory must be mapped and valid at the current vaddr with write permissions.
    pub unsafe fn as_mut_slice(&self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.vaddr as *mut u8, self.size) }
    }

    /// Get a pointer to the shared memory.
    pub fn as_ptr(&self) -> *mut u8 {
        self.vaddr as *mut u8
    }
}
