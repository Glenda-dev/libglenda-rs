use crate::arch::mem::PGSIZE;
use crate::cap::{Endpoint, Page};
use crate::client::ResourceClient;
use crate::drivers::interface::{DriverClient, FrameBufferDriver};
use crate::drivers::protocol::fb::FbInfo;
use crate::drivers::protocol::{FB_PROTO, fb};
use crate::error::Error;
use crate::interface::{CSpaceService, VSpaceService};
use crate::io::uring::{IoUringBuffer, IoUringClient, RingParams};
use crate::ipc::{MsgFlags, MsgTag, UTCB};
use crate::mem::Perms;
use crate::mem::shm::{SharedMemory, ShmParams};
use crate::utils::align::align_up;

pub struct FbClient {
    endpoint: Endpoint,
    info: FbInfo,
    ring: Option<IoUringClient>,
    shm: Option<SharedMemory>,
    ring_params: Option<RingParams>,
    shm_params: Option<ShmParams>,
    res_client: Option<ResourceClient>,
}

impl DriverClient for FbClient {
    fn connect(
        &mut self,
        vm: &mut dyn VSpaceService,
        cm: &mut dyn CSpaceService,
    ) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(FB_PROTO, fb::GET_INFO, MsgFlags::NONE);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        self.info = unsafe { utcb.read_obj::<FbInfo>().unwrap_or(FbInfo::default()) };

        if let Some(rp) = self.ring_params.clone() {
            self.setup_ring_internal(vm, cm, rp)?;
        }
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

impl FbClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self {
            endpoint,
            info: FbInfo { width: 0, height: 0, pitch: 0, format: 0, bpp: 0, paddr: 0, size: 0 },
            ring: None,
            shm: None,
            ring_params: None,
            shm_params: None,
            res_client: None,
        }
    }

    pub fn with_params(
        endpoint: Endpoint,
        res_client: ResourceClient,
        ring_params: RingParams,
        shm_params: ShmParams,
    ) -> Self {
        Self {
            endpoint,
            info: FbInfo::default(),
            ring: None,
            shm: None,
            ring_params: Some(ring_params),
            shm_params: Some(shm_params),
            res_client: Some(res_client),
        }
    }

    fn setup_ring_internal(
        &mut self,
        vm: &mut dyn VSpaceService,
        cm: &mut dyn CSpaceService,
        params: RingParams,
    ) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(FB_PROTO, fb::SETUP_RING, MsgFlags::HAS_CAP);
        utcb.set_mr(0, params.sq_entries);
        utcb.set_mr(1, params.cq_entries);
        utcb.set_msg_tag(tag);
        utcb.set_cap_transfer(params.notify_ep.cap());
        utcb.set_recv_window(params.recv_slot);
        self.endpoint.call(&mut utcb)?;

        let frame = Page::from(params.recv_slot);
        let res_client = self.res_client.as_mut().ok_or(Error::InvalidArgs)?;
        vm.map_page(
            frame,
            params.vaddr,
            Perms::READ | Perms::WRITE,
            align_up(params.size, PGSIZE) / PGSIZE,
            res_client,
            cm,
        )?;

        let ring_buf = unsafe {
            IoUringBuffer::new(
                params.vaddr as *mut u8,
                params.size,
                params.sq_entries as u32,
                params.cq_entries as u32,
            )
        };
        let mut ring = IoUringClient::new(ring_buf);
        ring.set_server_notify(self.endpoint);
        self.ring = Some(ring);
        Ok(())
    }

    pub fn setup_shm(
        &mut self,
        vm: &mut dyn VSpaceService,
        cm: &mut dyn CSpaceService,
        res_client: &mut ResourceClient,
        vaddr: usize,
        recv_slot: crate::cap::CapPtr,
    ) -> Result<SharedMemory, Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(FB_PROTO, fb::SETUP_BUFFER, MsgFlags::NONE);
        utcb.set_msg_tag(tag);
        utcb.set_recv_window(recv_slot);
        self.endpoint.call(&mut utcb)?;

        if !utcb.get_msg_tag().flags().contains(MsgFlags::OK) {
            return Err(Error::Generic);
        }

        let paddr = utcb.get_mr(0);
        let size = utcb.get_mr(1);
        let frame = Page::from(recv_slot);

        vm.map_page(
            frame.clone(),
            vaddr,
            Perms::READ | Perms::WRITE,
            align_up(size, PGSIZE) / PGSIZE,
            res_client,
            cm,
        )?;

        let mut shm = SharedMemory::new(frame, vaddr, size);
        shm.set_client_vaddr(vaddr);
        shm.set_paddr(paddr);
        self.shm = Some(shm.clone());
        self.info.paddr = paddr;
        self.info.size = size;
        Ok(shm)
    }

    pub fn info(&self) -> &FbInfo {
        &self.info
    }

    pub fn shm(&self) -> Option<&SharedMemory> {
        self.shm.as_ref()
    }
}

impl FrameBufferDriver for FbClient {
    fn get_info(&self) -> FbInfo {
        self.info.clone()
    }

    fn flush(&mut self, x: usize, y: usize, w: usize, h: usize) -> Result<(), Error> {
        if let Some(ring) = self.ring.as_ref() {
            let id = 0x2000; // unique id for flush
            let sqe = fb::sqe_flush(x, y, w, h, id);
            ring.submit(sqe)?;
            // For FB flush, we might not always want to wait synchronously if it's high freq,
            // but for now let's keep it simple.
            loop {
                if let Some(cqe) = ring.pop_completion() {
                    if cqe.user_data == id {
                        return if cqe.res >= 0 { Ok(()) } else { Err(Error::Generic) };
                    }
                }
                ring.wait_for_completions(&self.endpoint)?;
            }
        } else {
            let mut utcb = unsafe { UTCB::new() };
            utcb.clear();
            let tag = MsgTag::new(FB_PROTO, fb::FLUSH, MsgFlags::NONE);
            utcb.set_mr(0, x);
            utcb.set_mr(1, y);
            utcb.set_mr(2, w);
            utcb.set_mr(3, h);
            utcb.set_msg_tag(tag);
            self.endpoint.call(&mut utcb)
        }
    }

    fn set_scanout(&mut self, paddr: usize) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(FB_PROTO, fb::SET_SCANOUT, MsgFlags::NONE);
        utcb.set_mr(0, paddr);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }
}
