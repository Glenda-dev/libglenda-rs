use crate::arch::mem::PGSIZE;
use crate::cap::{CapPtr, CapType, Frame};
use crate::client::ResourceClient;
use crate::error::Error;
use crate::interface::{CSpaceService, ResourceService, VSpaceProvider, VSpaceService};
use crate::ipc::Badge;
use crate::mem::Perms;
use crate::mem::shm::SharedMemory;
use crate::utils::align::align_up;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};

/// Shared Memory Types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShmType {
    /// Physically contiguous memory for hardware DMA (Used by Drivers).
    DMA,
    /// Standard shared memory for inter-process communication (Used by Clients).
    Regular,
}

/// A reusable memory pool for managing shared memory regions within a service.
/// It tracks dynamic virtual address allocation.
pub struct MemoryPool {
    next_vaddr: AtomicUsize,
    shms: Vec<SharedMemory>,
}

impl MemoryPool {
    /// Create a new MemoryPool starting at the specified base virtual address.
    pub fn new(base_vaddr: usize) -> Self {
        Self { next_vaddr: AtomicUsize::new(base_vaddr), shms: Vec::new() }
    }

    /// Allocate and map a shared memory region.
    ///
    /// - `res_client`: Client to talk to the resource manager.
    /// - `size`: Requested size in bytes.
    /// - `shm_type`: Whether to allocate DMA-capable (contiguous) or regular memory.
    /// - `recv_slot`: Slot where the new frame capability will be received.
    pub fn alloc_shm(
        &mut self,
        vm: &mut dyn VSpaceService,
        cm: &mut dyn CSpaceService,
        res_client: &mut ResourceClient,
        size: usize,
        shm_type: ShmType,
        recv_slot: CapPtr,
    ) -> Result<SharedMemory, Error> {
        let size_aligned = (size + PGSIZE - 1) & !(PGSIZE - 1);
        let vaddr = self.next_vaddr.fetch_add(size_aligned, Ordering::SeqCst);

        let shm = match shm_type {
            ShmType::DMA => {
                let pages = size_aligned / PGSIZE;
                let (paddr, frame_cap) = res_client.dma_alloc(Badge::null(), pages, recv_slot)?;

                let mut shm = SharedMemory::new(frame_cap, vaddr, size);
                shm.set_paddr(paddr as u64);
                shm
            }
            ShmType::Regular => {
                let frame_cap = res_client.alloc(Badge::null(), CapType::Frame, size, recv_slot)?;
                SharedMemory::new(Frame::from(frame_cap), vaddr, size)
            }
        };

        vm.map_frame(
            shm.frame(),
            vaddr,
            Perms::READ | Perms::WRITE,
            size_aligned / PGSIZE,
            res_client,
            cm,
        )?;

        self.shms.push(shm);
        Ok(shm)
    }

    /// Get all allocated shared memory regions.
    pub fn shms(&self) -> &[SharedMemory] {
        &self.shms.as_slice()
    }

    /// Find an SHM by its vaddr.
    pub fn find_by_vaddr(&self, vaddr: usize) -> Option<&SharedMemory> {
        self.shms.iter().find(|s| s.vaddr() <= vaddr && vaddr < s.vaddr() + s.size())
    }

    /// Return the next available virtual address.
    pub fn next_vaddr(&self) -> usize {
        self.next_vaddr.load(Ordering::SeqCst)
    }

    /// Map an existing frame into the pool and manage it.
    pub fn map_shm(
        &mut self,
        vm: &mut dyn VSpaceService,
        cm: &mut dyn CSpaceService,
        provider: &mut dyn VSpaceProvider,
        frame: Frame,
        size: usize,
        perms: Perms,
    ) -> Result<SharedMemory, Error> {
        let size_aligned = align_up(size, PGSIZE);
        let vaddr = self.next_vaddr.fetch_add(size_aligned, Ordering::SeqCst);

        let shm = SharedMemory::new(frame, vaddr, size);

        // Map locally in the service's VSpace via VSpaceManager
        vm.map_frame(frame, vaddr, perms, size_aligned / PGSIZE, provider, cm)?;

        self.shms.push(shm);
        Ok(shm)
    }

    /// Reserve a virtual address range in the pool.
    pub fn reserve(&self, size: usize) -> usize {
        let size_aligned = (size + PGSIZE - 1) & !(PGSIZE - 1);
        self.next_vaddr.fetch_add(size_aligned, Ordering::SeqCst)
    }
}
