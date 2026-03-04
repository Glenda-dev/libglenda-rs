use crate::arch::mem::PGSIZE;
use crate::cap::{Endpoint, Frame};
use crate::client::ResourceClient;
use crate::drivers::client::{RingParams, ShmParams};
use crate::drivers::interface::{DriverClient, UartDriver};
use crate::drivers::protocol::{UART_PROTO, uart};
use crate::error::Error;
use crate::interface::{CSpaceService, VSpaceService};
use crate::io::uring::{IoUringBuffer, IoUringClient, IoUringCqe};
use crate::ipc::IPC_BUFFER_SIZE;
use crate::ipc::{MsgFlags, MsgTag, UTCB};
use crate::mem::Perms;
use crate::mem::shm::SharedMemory;
use crate::utils::align::align_up;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone)]
pub struct UartClient {
    endpoint: Endpoint,
    notify_ep: Option<Endpoint>,
    ring: Option<IoUringClient>,
    shm: Option<SharedMemory>,
    next_id: Arc<AtomicU64>,
    ring_params: RingParams,
    shm_params: ShmParams,
    res_client: ResourceClient,
}

impl DriverClient for UartClient {
    fn connect(
        &mut self,
        vm: &mut dyn VSpaceService,
        cm: &mut dyn CSpaceService,
    ) -> Result<(), Error> {
        self.setup_ring_internal(vm, cm)?;
        self.setup_shm_internal(vm, cm)?;

        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

impl UartClient {
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
            next_id: Arc::new(AtomicU64::new(0x1000)),
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

    pub fn set_ring(&mut self, mut ring: IoUringClient) {
        ring.set_server_notify(self.endpoint);
        self.ring = Some(ring);
    }

    fn next_user_data(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
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
        utcb.set_recv_window(recv);
        utcb.set_cap_transfer(notify_ep.cap());
        let tag = MsgTag::new(UART_PROTO, uart::SETUP_RING, MsgFlags::HAS_CAP);
        utcb.set_mr(0, sq_entries as usize);
        utcb.set_mr(1, cq_entries as usize);
        utcb.set_msg_tag(tag);

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

        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_cap_transfer(frame.cap());
        let tag = MsgTag::new(UART_PROTO, uart::SETUP_BUFFER, MsgFlags::HAS_CAP);
        utcb.set_mr(0, vaddr);
        utcb.set_mr(1, size);
        utcb.set_mr(2, paddr as usize);
        utcb.set_msg_tag(tag);

        self.endpoint.call(&mut utcb)?;

        let mut shm = SharedMemory::new(frame, vaddr, size);
        shm.set_paddr(paddr);
        self.set_shm(shm);
        Ok(())
    }

    pub fn read_async(&self, addr: u64, len: u32, user_data: u64) -> Result<(), Error> {
        let ring = self.ring.as_ref().ok_or(Error::NotInitialized)?;
        let sqe = uart::sqe_read(addr, len, user_data);
        ring.submit(sqe)
    }

    pub fn write_async(&self, addr: u64, len: u32, user_data: u64) -> Result<(), Error> {
        let ring = self.ring.as_ref().ok_or(Error::NotInitialized)?;
        let sqe = uart::sqe_write(addr, len, user_data);
        ring.submit(sqe)
    }

    pub fn peek_cqe(&self) -> Option<IoUringCqe> {
        self.ring.as_ref()?.peek_completion()
    }

    pub fn wait_for_completions(&self) -> Result<(), Error> {
        let ring = self.ring.as_ref().ok_or(Error::NotInitialized)?;
        let ep = self.notify_ep.as_ref().unwrap_or(&self.endpoint);
        ring.wait_for_completions(ep)
    }

    pub fn shm_params(&self) -> &ShmParams {
        &self.shm_params
    }
}

impl UartDriver for UartClient {
    fn put_char(&mut self, c: u8) {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(UART_PROTO, uart::PUT_CHAR, MsgFlags::NONE);
        utcb.set_msg_tag(tag);
        utcb.set_mr(0, c as usize);

        let _ = self.endpoint.call(&mut utcb);
    }

    fn get_char(&mut self) -> Option<u8> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(UART_PROTO, uart::GET_CHAR, MsgFlags::NONE);
        utcb.set_msg_tag(tag);

        match self.endpoint.call(&mut utcb) {
            Ok(_) => Some(utcb.get_mr(0) as u8),
            Err(_) => None,
        }
    }

    fn put_str(&mut self, s: &str) {
        let bytes = s.as_bytes();
        for chunk in bytes.chunks(IPC_BUFFER_SIZE) {
            let mut utcb = unsafe { UTCB::new() };
            utcb.clear();
            let buf = &mut utcb.ipc_buffer();
            buf[..chunk.len()].copy_from_slice(chunk);
            let tag = MsgTag::new(UART_PROTO, uart::PUT_STR, MsgFlags::NONE);
            utcb.set_msg_tag(tag);
            utcb.set_size(chunk.len());

            let _ = self.endpoint.call(&mut utcb);
        }
    }

    fn set_baud_rate(&mut self, baud: u32) {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(UART_PROTO, uart::SET_BAUD_RATE, MsgFlags::NONE);
        utcb.set_msg_tag(tag);
        utcb.set_mr(0, baud as usize);

        let _ = self.endpoint.call(&mut utcb);
    }
}
