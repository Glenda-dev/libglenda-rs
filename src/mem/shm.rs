use crate::arch::mem::PGSIZE;
use crate::cap::{Frame, VSpace, CapPtr};
use crate::error::Error;
use crate::mem::Perms;
use crate::utils::align::align_up;
use core::slice;

/// A shared memory region.
#[derive(Debug, Clone, Copy)]
pub struct SharedMemory {
    frame: Frame,
    vaddr: usize,
    client_vaddr: usize,
    paddr: u64,
    size: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct ShmParams {
    pub frame: Frame,
    pub vaddr: usize,
    pub paddr: u64,
    pub size: usize,
    pub recv_slot: CapPtr,
}

impl SharedMemory {
    /// Create a SharedMemory instance from an existing Frame.
    pub const fn from_frame(frame: Frame, vaddr: usize, size: usize) -> Self {
        Self { frame, vaddr, client_vaddr: vaddr, paddr: 0, size }
    }

    pub const fn new(frame: Frame, vaddr: usize, size: usize) -> Self {
        Self { frame, vaddr, client_vaddr: vaddr, paddr: 0, size }
    }

    pub fn set_client_vaddr(&mut self, vaddr: usize) {
        self.client_vaddr = vaddr;
    }

    pub fn client_vaddr(&self) -> usize {
        self.client_vaddr
    }

    pub fn set_paddr(&mut self, paddr: u64) {
        self.paddr = paddr;
    }

    pub fn paddr(&self) -> u64 {
        self.paddr
    }

    /// Map the shared memory into a VSpace.
    pub fn map(&self, vspace: &VSpace, perms: Perms) -> Result<(), Error> {
        vspace.map(self.frame, self.vaddr, perms, align_up(self.size, PGSIZE) / PGSIZE)
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

    /// Check if the pointer is within the shared memory region.
    pub fn contains_ptr(&self, ptr: *const u8) -> bool {
        let p = ptr as usize;
        p >= self.vaddr && p < self.vaddr + self.size
    }

    /// Get the client virtual address for a pointer within the shared memory region.
    ///
    /// # Panics
    /// Panics if the pointer is not within the shared memory region.
    pub fn client_vaddr_at(&self, ptr: *const u8) -> usize {
        assert!(self.contains_ptr(ptr));
        let offset = ptr as usize - self.vaddr;
        self.client_vaddr + offset
    }
}
