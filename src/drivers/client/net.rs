use crate::arch::mem::PGSIZE;
use crate::cap::{Endpoint, Page};
use crate::client::ResourceClient;
use crate::drivers::client::{RingParams, ShmParams};
use crate::drivers::interface::{DriverClient, NetDriver};
use crate::drivers::protocol::net::MacAddress;
use crate::drivers::protocol::{NET_PROTO, net};
use crate::error::Error;
use crate::interface::{CSpaceService, VSpaceService};
use crate::io::uring::{IoUringBuffer, IoUringClient, IoUringCqe};
use crate::ipc::{MsgFlags, MsgTag, UTCB};
use crate::mem::Perms;
use crate::mem::shm::SharedMemory;
use crate::utils::align::align_up;
use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::sync::Arc;

#[derive(Clone)]
pub struct NetClient {
    endpoint: Endpoint,
    notify_ep: Option<Endpoint>,
    ring: Option<IoUringClient>,
    shm: Option<SharedMemory>,
    next_id: Arc<AtomicUsize>,
    mac: Option<MacAddress>,
    ring_params: RingParams,
    shm_params: ShmParams,
    res_client: ResourceClient,
}

impl DriverClient for NetClient {
    fn connect(
        &mut self,
        vm: &mut dyn VSpaceService,
        cm: &mut dyn CSpaceService,
    ) -> Result<(), Error> {
        let mac = self.mac_address();
        self.mac = Some(mac);

        self.setup_ring_internal(vm, cm)?;
        self.setup_shm_internal(vm, cm)?;

        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

impl NetClient {
    pub fn new(
        endpoint: Endpoint,
        res_client: &mut ResourceClient,
        ring_params: RingParams,
        shm_params: ShmParams,
    ) -> Self {
        Self {
            endpoint,
            notify_ep: None,
            ring: None,
            shm: None,
            next_id: Arc::new(AtomicUsize::new(0x1000)),
            mac: None,
            ring_params,
            shm_params,
            res_client: res_client.clone(),
        }
    }

    pub fn endpoint(&self) -> Endpoint {
        self.endpoint
    }

    pub fn set_shm(&mut self, shm: SharedMemory) {
        self.shm = Some(shm);
    }

    pub fn shm(&self) -> Option<&SharedMemory> {
        self.shm.as_ref()
    }

    pub fn set_ring(&mut self, mut ring: IoUringClient) {
        ring.set_server_notify(self.endpoint);
        self.ring = Some(ring);
    }

    pub fn ring(&self) -> Option<&IoUringClient> {
        self.ring.as_ref()
    }

    fn next_user_data(&self) -> usize {
        #[allow(clippy::useless_conversion)]
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    pub fn send_packet(&self, buf: &[u8]) -> Result<(), Error> {
        let ring = self.ring.as_ref().ok_or(Error::NotInitialized)?;
        let id = self.next_user_data();

        // Use SHM address if buffer is within SHM
        let addr = if let Some(shm) = &self.shm {
            if shm.contains_ptr(buf.as_ptr()) {
                shm.client_vaddr_at(buf.as_ptr()) as usize
            } else {
                buf.as_ptr() as usize
            }
        } else {
            buf.as_ptr() as usize
        };

        let sqe = net::sqe_send(addr, buf.len() as u32, id);

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

    pub fn submit_recv(&self, buf: &mut [u8], id: usize) -> Result<(), Error> {
        let ring = self.ring.as_ref().ok_or(Error::NotInitialized)?;
        let addr = if let Some(shm) = &self.shm {
            if shm.contains_ptr(buf.as_ptr()) {
                shm.client_vaddr_at(buf.as_ptr()) as usize
            } else {
                buf.as_ptr() as usize
            }
        } else {
            buf.as_ptr() as usize
        };

        let sqe = net::sqe_recv(addr, buf.len() as u32, id);
        ring.submit(sqe)?;
        Ok(())
    }

    pub fn peek_cqe(&self) -> Option<IoUringCqe> {
        self.ring.as_ref()?.pop_completion()
    }
}

impl NetDriver for NetClient {
    fn mac_address(&self) -> MacAddress {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(NET_PROTO, net::GET_MAC, MsgFlags::NONE);
        utcb.set_msg_tag(tag);
        if let Ok(_) = self.endpoint.call(&mut utcb) {
            let mut mac = [0u8; 6];
            for i in 0..6 {
                mac[i] = utcb.get_mr(i) as u8;
            }
            MacAddress { octets: mac }
        } else {
            MacAddress { octets: [0; 6] }
        }
    }
}

impl NetClient {
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
        utcb.set_recv_window(recv);
        utcb.set_cap_transfer(notify_ep.cap());
        let tag = MsgTag::new(NET_PROTO, net::SETUP_RING, MsgFlags::HAS_CAP);
        utcb.set_mr(0, sq_entries as usize);
        utcb.set_mr(1, cq_entries as usize);
        utcb.set_msg_tag(tag);

        self.endpoint.call(&mut utcb)?;

        let frame = Page::from(recv);
        vm.map_page(
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
        self.ring = Some(IoUringClient::new(ring_buf));
        Ok(())
    }

    fn setup_shm_internal(
        &mut self,
        _vm: &mut dyn VSpaceService,
        _cm: &mut dyn CSpaceService,
    ) -> Result<(), Error> {
        let frame = self.shm_params.frame.clone();
        let vaddr = self.shm_params.vaddr;
        let paddr = self.shm_params.paddr;
        let size = self.shm_params.size;
        let recv = self.shm_params.recv_slot;

        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_cap_transfer(frame.cap());
        let tag = MsgTag::new(NET_PROTO, net::SETUP_BUFFER, MsgFlags::HAS_CAP);
        utcb.set_mr(0, vaddr);
        utcb.set_mr(1, size);
        utcb.set_mr(2, paddr as usize);
        utcb.set_msg_tag(tag);
        utcb.set_recv_window(recv);

        self.endpoint.call(&mut utcb)?;

        let mut shm = SharedMemory::new(frame, vaddr, size);
        shm.set_paddr(paddr);
        self.set_shm(shm);
        Ok(())
    }
}
