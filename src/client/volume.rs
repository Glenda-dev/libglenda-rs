use crate::arch::mem::PGSIZE;
use crate::cap::{CapPtr, Endpoint, Frame};
use crate::client::resource::ResourceClient;
use crate::error::Error;
use crate::interface::volume::VolumeService;
use crate::interface::{CSpaceService, VSpaceService};
use crate::io::uring::IoUringBuffer;
use crate::io::uring::{IoUringClient, RingParams};
use crate::ipc::{Badge, MsgFlags, MsgTag, UTCB};
use crate::mem::Perms;
use crate::mem::shm::{SharedMemory, ShmParams};
use crate::protocol::init::ServiceState;
use crate::protocol::volume;
use crate::utils::align::align_up;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone)]
pub struct VolumeClient {
    endpoint: Endpoint,
    notify_ep: Option<Endpoint>,
    ring: Option<IoUringClient>,
    shm: Option<SharedMemory>,
    block_size: u32,
    total_sectors: usize,
    next_id: Arc<AtomicUsize>,
    ring_params: RingParams,
    shm_params: ShmParams,
    res_client: ResourceClient,
}

impl VolumeClient {
    pub fn connect(
        &mut self,
        vm: &mut dyn VSpaceService,
        cm: &mut dyn CSpaceService,
    ) -> Result<(), Error> {
        let tag = MsgTag::new(crate::protocol::VOLUME_PROTO, volume::GET_INFO, MsgFlags::NONE);
        let u = unsafe { UTCB::new() };
        u.set_msg_tag(tag);
        self.endpoint.call(u)?;

        if !u.get_msg_tag().flags().contains(MsgFlags::OK) {
            return Err(Error::Generic);
        }

        self.block_size = u.get_mr(0) as u32;
        self.total_sectors = u.get_mr(1) as usize;

        self.setup_ring_internal(vm, cm)?;
        self.setup_shm_internal(vm, cm)?;

        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<(), Error> {
        Ok(())
    }

    pub fn new(
        endpoint: Endpoint,
        res_client: &ResourceClient,
        ring_params: RingParams,
        shm_params: ShmParams,
    ) -> Self {
        Self {
            endpoint,
            notify_ep: None,
            ring: None,
            shm: None,
            block_size: 0,
            total_sectors: 0,
            next_id: Arc::new(AtomicUsize::new(0x1000)),
            ring_params,
            shm_params,
            res_client: res_client.clone(),
        }
    }

    pub fn new_simple(endpoint: Endpoint, res_client: &ResourceClient) -> Self {
        Self {
            endpoint,
            notify_ep: None,
            ring: None,
            shm: None,
            block_size: 0,
            total_sectors: 0,
            next_id: Arc::new(AtomicUsize::new(0x1000)),
            ring_params: RingParams {
                sq_entries: 0,
                cq_entries: 0,
                notify_ep: Endpoint::from(CapPtr::null()),
                recv_slot: CapPtr::null(),
                vaddr: 0,
                size: 0,
            },
            shm_params: ShmParams {
                frame: Frame::from(CapPtr::null()),
                vaddr: 0,
                paddr: 0,
                size: 0,
                recv_slot: CapPtr::null(),
            },
            res_client: res_client.clone(),
        }
    }

    pub fn endpoint(&self) -> Endpoint {
        self.endpoint
    }

    pub fn set_shm(&mut self, shm: SharedMemory) {
        self.shm = Some(shm);
    }

    pub fn set_ring(&mut self, ring: IoUringClient) {
        self.ring = Some(ring);
    }

    pub fn ring(&self) -> Option<&IoUringClient> {
        self.ring.as_ref()
    }

    pub fn get_device(&self, badge: Badge, recv: CapPtr) -> Result<Endpoint, Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(crate::protocol::VOLUME_PROTO, volume::GET_DEVICE, MsgFlags::NONE);
        utcb.set_msg_tag(tag);
        utcb.set_mr(0, badge.bits());
        utcb.set_recv_window(recv);
        self.endpoint.call(&mut utcb)?;

        if !utcb.get_msg_tag().flags().contains(MsgFlags::HAS_CAP) {
            return Err(Error::Generic);
        }

        Ok(Endpoint::from(recv))
    }

    fn next_user_data(&self) -> usize {
        self.next_id.fetch_add(1, Ordering::SeqCst) as usize
    }

    /// Read data from disk directly to a shared memory address.
    /// This assumes shm_vaddr is within the shm region provided to set_shm.
    pub fn read_shm(&self, sector: usize, len: u32, shm_vaddr: usize) -> Result<(), Error> {
        let ring = self.ring.as_ref().ok_or(Error::NotInitialized)?;
        let _shm = self.shm.as_ref().ok_or(Error::NotInitialized)?;

        // Ensure alignment to block_size
        if self.block_size == 0 || len % self.block_size != 0 {
            return Err(Error::InvalidArgs);
        }

        let id = self.next_user_data();

        let sqe = volume::sqe_read(sector, shm_vaddr as usize, len, id);
        ring.submit(sqe)?;

        // Block until completion
        let wait_ep = self.notify_ep.as_ref().unwrap_or(&self.endpoint);
        loop {
            if let Some(cqe) = ring.pop_completion() {
                if cqe.user_data == id {
                    if cqe.res < 0 {
                        return Err(Error::Generic);
                    }
                    return Ok(());
                }
            }
            ring.wait_for_completions(wait_ep)?;
        }
    }

    /// Read data at sector offset and count.
    pub fn read_at(&self, sector: usize, len: u32, buf: &mut [u8]) -> Result<(), Error> {
        let ring = self.ring.as_ref().ok_or(Error::NotInitialized)?;
        let shm = self.shm.as_ref().ok_or(Error::NotInitialized)?;

        // Ensure alignment to block_size
        if self.block_size == 0 || len % self.block_size != 0 {
            return Err(Error::InvalidArgs);
        }

        if len as usize > shm.size() {
            return Err(Error::InvalidArgs);
        }

        let id = self.next_user_data();

        // Use the beginning of SHM for synchronous operations
        // We use client_vaddr because that's what the server expects.
        let src_addr = shm.client_vaddr() as usize;

        let sqe = volume::sqe_read(sector, src_addr, len, id);
        ring.submit(sqe)?;

        // Block until completion
        let wait_ep = self.notify_ep.as_ref().unwrap_or(&self.endpoint);
        loop {
            if let Some(cqe) = ring.pop_completion() {
                if cqe.user_data == id {
                    if cqe.res < 0 {
                        return Err(Error::Generic);
                    }
                    // Copy back from SHM
                    let shm_buf = unsafe {
                        core::slice::from_raw_parts(shm.vaddr() as *const u8, len as usize)
                    };
                    let copy_len = core::cmp::min(len as usize, buf.len());
                    buf[..copy_len].copy_from_slice(&shm_buf[..copy_len]);
                    return Ok(());
                }
            }
            ring.wait_for_completions(wait_ep)?;
        }
    }

    /// Write data at sector offset and count.
    pub fn write_at(&self, sector: usize, len: u32, buf: &[u8]) -> Result<(), Error> {
        let ring = self.ring.as_ref().ok_or(Error::NotInitialized)?;
        let shm = self.shm.as_ref().ok_or(Error::NotInitialized)?;

        // Ensure alignment to block_size
        if self.block_size == 0 || len % self.block_size != 0 {
            return Err(Error::InvalidArgs);
        }

        if len as usize > shm.size() {
            return Err(Error::InvalidArgs);
        }

        let id = self.next_user_data();

        // Copy to SHM first
        let shm_buf =
            unsafe { core::slice::from_raw_parts_mut(shm.vaddr() as *mut u8, len as usize) };
        let copy_len = core::cmp::min(len as usize, buf.len());
        shm_buf[..copy_len].copy_from_slice(&buf[..copy_len]);

        // Use the beginning of SHM for synchronous operations
        // We use client_vaddr because that's what the server expects.
        let dst_addr = shm.client_vaddr() as usize;

        let sqe = volume::sqe_write(sector, dst_addr, len, id);
        ring.submit(sqe)?;

        let wait_ep = self.notify_ep.as_ref().unwrap_or(&self.endpoint);
        loop {
            if let Some(cqe) = ring.pop_completion() {
                if cqe.user_data == id {
                    if cqe.res < 0 {
                        return Err(Error::Generic);
                    }
                    return Ok(());
                }
            }
            ring.wait_for_completions(wait_ep)?;
        }
    }

    /// Synchronous read using io_uring (compat).
    pub fn read_blocks(&self, sector: usize, count: u32, buf: &mut [u8]) -> Result<(), Error> {
        self.read_at(sector, count * self.block_size, buf)
    }

    /// Synchronous write using io_uring (compat).
    pub fn write_blocks(&self, sector: usize, count: u32, buf: &[u8]) -> Result<(), Error> {
        self.write_at(sector, count * self.block_size, buf)
    }

    fn setup_ring_internal(
        &mut self,
        vm: &mut dyn VSpaceService,
        cm: &mut dyn CSpaceService,
    ) -> Result<(), Error> {
        let sq_entries = self.ring_params.sq_entries;
        let cq_entries = self.ring_params.cq_entries;
        let notify_ep = self.ring_params.notify_ep;
        let recv = self.ring_params.recv_slot;
        let vaddr = self.ring_params.vaddr;
        let size = self.ring_params.size;

        self.notify_ep = Some(notify_ep);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(crate::protocol::VOLUME_PROTO, volume::SETUP_RING, MsgFlags::HAS_CAP);
        utcb.set_mr(0, sq_entries as usize);
        utcb.set_mr(1, cq_entries as usize);
        utcb.set_msg_tag(tag);
        utcb.set_cap_transfer(notify_ep.cap());
        utcb.set_recv_window(recv);
        self.endpoint.call(&mut utcb)?;

        let frame = Frame::from(recv);
        vm.map_frame(
            frame.clone(),
            vaddr,
            Perms::READ | Perms::WRITE,
            align_up(size, PGSIZE) / PGSIZE,
            &mut self.res_client,
            cm,
        )?;

        let ring_buf = unsafe {
            IoUringBuffer::new(vaddr as *mut u8, size, sq_entries as u32, cq_entries as u32)
        };
        let mut ring = IoUringClient::new(ring_buf);
        ring.set_server_notify(self.endpoint);
        self.ring = Some(ring);
        Ok(())
    }

    fn setup_shm_internal(
        &mut self,
        vm: &mut dyn VSpaceService,
        cm: &mut dyn CSpaceService,
    ) -> Result<(), Error> {
        let vaddr = self.shm_params.vaddr;
        let recv = self.shm_params.recv_slot;

        if recv.is_null() {
            return Err(Error::InvalidArgs);
        }

        // 1. Request shared memory from Fossil (the sole allocator/server)
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(crate::protocol::VOLUME_PROTO, volume::ACQUIRE_SHM, MsgFlags::NONE);
        // We can pass our preferred vaddr in MR0 if we have one.
        utcb.set_mr(0, vaddr);
        utcb.set_msg_tag(tag);
        utcb.set_recv_window(recv);
        self.endpoint.call(&mut utcb)?;

        if !utcb.get_msg_tag().flags().contains(MsgFlags::OK | MsgFlags::HAS_CAP) {
            return Err(Error::Generic);
        }

        // 2. Get the frame and Fossil's suggested view (the master mapping)
        let actual_frame = Frame::from(recv);
        let srv_vaddr = utcb.get_mr(0);
        let srv_size = utcb.get_mr(1);
        // Physical address is NOT leaked to us anymore.

        // 3. Map it locally (use server's suggested vaddr as default)
        let local_vaddr = if vaddr != 0 { vaddr } else { srv_vaddr };

        vm.map_frame(
            actual_frame.clone(),
            local_vaddr,
            Perms::READ | Perms::WRITE,
            align_up(srv_size, PGSIZE) / PGSIZE,
            &mut self.res_client,
            cm,
        )?;

        // 4. Register our local mapping with Fossil to allow SQE translation
        utcb.clear();
        let tag = MsgTag::new(crate::protocol::VOLUME_PROTO, volume::REGISTER_SHM, MsgFlags::NONE);
        utcb.set_mr(0, local_vaddr);
        utcb.set_mr(1, srv_size);
        utcb.set_msg_tag(tag);
        // Note: Registration confirmed
        self.endpoint.call(&mut utcb)?;

        let mut shm = SharedMemory::new(actual_frame, local_vaddr, srv_size);
        shm.set_client_vaddr(local_vaddr);
        self.shm = Some(shm);

        Ok(())
    }

    pub fn capacity(&self) -> usize {
        #[allow(clippy::useless_conversion)]
        self.total_sectors
    }

    pub fn block_size(&self) -> u32 {
        self.block_size
    }
}

impl VolumeService for VolumeClient {
    fn get_device(&mut self, pid: Badge, recv: CapPtr) -> Result<Endpoint, Error> {
        VolumeClient::get_device(self, pid, recv)
    }

    fn probe_device(&mut self, _pid: Badge, device_name: &str) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        unsafe { utcb.write_str(device_name)? };
        let tag =
            MsgTag::new(crate::protocol::VOLUME_PROTO, volume::PROBE_DEVICE, MsgFlags::HAS_BUFFER);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }

    fn report_state(
        &mut self,
        _pid: Badge,
        state: ServiceState,
        endpoint: Option<CapPtr>,
    ) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_mr(0, state as usize);

        let mut flags = MsgFlags::NONE;
        if let Some(ep) = endpoint {
            utcb.set_cap_transfer(ep);
            flags |= MsgFlags::HAS_CAP;
        }

        let tag = MsgTag::new(crate::protocol::VOLUME_PROTO, volume::REPORT_STATE, flags);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }

    fn mount_partition(
        &mut self,
        _pid: Badge,
        partition_name: &str,
        recv: CapPtr,
    ) -> Result<Endpoint, Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        unsafe { utcb.write_str(partition_name)? };
        let tag = MsgTag::new(
            crate::protocol::VOLUME_PROTO,
            volume::MOUNT_PARTITION,
            MsgFlags::HAS_BUFFER,
        );
        utcb.set_recv_window(recv);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;

        if !utcb.get_msg_tag().flags().contains(MsgFlags::HAS_CAP) {
            return Err(Error::Generic);
        }

        Ok(Endpoint::from(recv))
    }
}
