use crate::cap::{CapPtr, Endpoint};
use core::sync::atomic::{AtomicU32, Ordering};

// Operation Codes
pub const IOURING_OP_NOP: u8 = 0;
pub const IOURING_OP_READ: u8 = 1;
pub const IOURING_OP_WRITE: u8 = 2;
pub const IOURING_OP_SYNC: u8 = 3;
pub const IOURING_OP_READV: u8 = 4;
pub const IOURING_OP_WRITEV: u8 = 5;
pub const IOURING_OP_MSG_RING: u8 = 40; // Glenda: Used for SHM Ring Buffer notification

// SQE Flags
pub const IOSQE_FIXED_FILE: u8 = 1 << 0;
pub const IOSQE_IO_DRAIN: u8 = 1 << 1;
pub const IOSQE_IO_LINK: u8 = 1 << 2;
pub const IOSQE_IO_HARDLINK: u8 = 1 << 3;
pub const IOSQE_ASYNC: u8 = 1 << 4;
pub const IOSQE_BUFFER_SELECT: u8 = 1 << 5;
pub const IOSQE_MULTISHOT: u8 = 1 << 6; // Glenda Extension for Multi-shot read

// CQE Flags
pub const IORING_CQE_F_MORE: u32 = 1 << 0; // More CQEs follow for this SQE

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct IoUringSqe {
    pub opcode: u8,
    pub flags: u8,
    pub ioprio: u16,
    pub fd: i32,
    pub off: usize,
    pub addr: usize,
    pub len: u32,
    pub user_data: usize,
    pub __pad: [usize; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct IoUringCqe {
    pub user_data: usize,
    pub res: i32,
    pub flags: u32,
}

// Legacy Alias for High-Level wrapper compatibility
pub type SQEntry = IoUringSqe;
pub type CQEntry = IoUringCqe;

pub const SQE_SIZE: usize = 64;
pub const CQE_SIZE: usize = 16;
pub const HEADER_SIZE: usize = 64; // round up to cache line

#[repr(C, align(64))] // Cache line alignment
pub struct IoUringGenericHeader {
    pub head: AtomicU32,
    pub tail: AtomicU32,
    pub mask: u32,
    pub flags: u32,
}

pub struct IoUringGeneric<'a> {
    pub header: &'a mut IoUringGenericHeader,
    pub sq_entries: &'a mut [IoUringSqe],
    pub cq_entries: &'a mut [IoUringCqe],
}

impl<'a> IoUringGeneric<'a> {
    pub unsafe fn new(base: *mut u8, entries: usize) -> Self {
        let header = unsafe { &mut *(base as *mut IoUringGenericHeader) };
        let sq_offset = core::mem::size_of::<IoUringGenericHeader>();
        let cq_offset = sq_offset + entries * core::mem::size_of::<IoUringSqe>();

        let sq_ptr = unsafe { base.add(sq_offset) } as *mut IoUringSqe;
        let cq_ptr = unsafe { base.add(cq_offset) } as *mut IoUringCqe;

        Self {
            header,
            sq_entries: unsafe { core::slice::from_raw_parts_mut(sq_ptr, entries) },
            cq_entries: unsafe { core::slice::from_raw_parts_mut(cq_ptr, entries) },
        }
    }
}

#[repr(C)]
pub struct IoUringLayout {
    pub sq_head: AtomicU32,
    pub sq_tail: AtomicU32,
    pub sq_mask: u32,
    pub sq_entries: u32,
    pub cq_head: AtomicU32,
    pub cq_tail: AtomicU32,
    pub cq_mask: u32,
    pub cq_entries: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct IoUringBuffer {
    ptr: *mut u8,
    sq_entries: u32,
    cq_entries: u32,
}

unsafe impl Send for IoUringBuffer {}
unsafe impl Sync for IoUringBuffer {}

impl IoUringBuffer {
    pub unsafe fn new(ptr: *mut u8, _size: usize, sq_entries: u32, cq_entries: u32) -> Self {
        unsafe {
            let header = &mut *(ptr as *mut IoUringLayout);
            header.sq_entries = sq_entries;
            header.sq_mask = sq_entries - 1;
            header.cq_entries = cq_entries;
            header.cq_mask = cq_entries - 1;
            header.sq_head.store(0, Ordering::Release);
            header.sq_tail.store(0, Ordering::Release);
            header.cq_head.store(0, Ordering::Release);
            header.cq_tail.store(0, Ordering::Release);
        }

        Self { ptr, sq_entries, cq_entries }
    }

    pub unsafe fn attach(ptr: *mut u8, _size: usize) -> Self {
        let header = unsafe { &*(ptr as *const IoUringLayout) };
        Self { ptr, sq_entries: header.sq_entries, cq_entries: header.cq_entries }
    }

    fn header(&self) -> &mut IoUringLayout {
        unsafe { &mut *(self.ptr as *mut IoUringLayout) }
    }

    fn sqes_mut(&self) -> *mut IoUringSqe {
        unsafe { self.ptr.add(HEADER_SIZE) as *mut IoUringSqe }
    }

    fn cqes_mut(&self) -> *mut IoUringCqe {
        unsafe {
            self.ptr.add(HEADER_SIZE + self.sq_entries as usize * SQE_SIZE) as *mut IoUringCqe
        }
    }

    pub fn push_sqe(&self, sqe: IoUringSqe) -> Result<(), ()> {
        let header = self.header();
        let tail = header.sq_tail.load(Ordering::Acquire);
        let head = header.sq_head.load(Ordering::Acquire);

        if tail.wrapping_sub(head) >= self.sq_entries {
            return Err(());
        }

        let index = tail & header.sq_mask;
        unsafe {
            let ptr = self.sqes_mut().add(index as usize);
            *ptr = sqe;
        }

        header.sq_tail.store(tail.wrapping_add(1), Ordering::Release);
        Ok(())
    }

    pub fn pop_sqe(&self) -> Option<IoUringSqe> {
        let header = self.header();
        let head = header.sq_head.load(Ordering::Acquire);
        let tail = header.sq_tail.load(Ordering::Acquire);

        if head == tail {
            return None;
        }

        let index = head & header.sq_mask;
        let sqe = unsafe { core::ptr::read_volatile(self.sqes_mut().add(index as usize)) };

        header.sq_head.store(head.wrapping_add(1), Ordering::Release);
        Some(sqe)
    }

    pub fn push_cqe(&self, cqe: IoUringCqe) -> Result<(), ()> {
        let header = self.header();
        let tail = header.cq_tail.load(Ordering::Acquire);
        let head = header.cq_head.load(Ordering::Acquire);

        if tail.wrapping_sub(head) >= self.cq_entries {
            return Err(());
        }

        let index = tail & header.cq_mask;
        unsafe {
            let ptr = self.cqes_mut().add(index as usize);
            *ptr = cqe;
        }

        header.cq_tail.store(tail.wrapping_add(1), Ordering::Release);

        // Glenda: Trigger notification after push if possible
        Ok(())
    }

    pub fn pop_cqe(&self) -> Option<IoUringCqe> {
        let header = self.header();
        let head = header.cq_head.load(Ordering::Acquire);
        let tail = header.cq_tail.load(Ordering::Acquire);

        if head == tail {
            return None;
        }

        let index = head & header.cq_mask;
        let cqe = unsafe { *self.cqes_mut().add(index as usize) };

        header.cq_head.store(head.wrapping_add(1), Ordering::Release);
        Some(cqe)
    }

    pub fn sq_len(&self) -> u32 {
        let header = self.header();
        let tail = header.sq_tail.load(Ordering::Relaxed);
        let head = header.sq_head.load(Ordering::Relaxed);
        tail.wrapping_sub(head)
    }

    pub fn cq_len(&self) -> u32 {
        let header = self.header();
        let tail = header.cq_tail.load(Ordering::Relaxed);
        let head = header.cq_head.load(Ordering::Relaxed);
        tail.wrapping_sub(head)
    }
}

use crate::error::Error;
use crate::ipc::Badge;

#[derive(Debug, Clone, Copy)]
pub struct RingParams {
    pub sq_entries: usize,
    pub cq_entries: usize,
    pub notify_ep: Endpoint,
    pub recv_slot: CapPtr,
    pub vaddr: usize,
    pub size: usize,
}

/// 默认 IO_URING 发送队列通知位
/// 默认 IO_URING 完成队列通知位

pub struct IoUringServer {
    pub ring: IoUringBuffer,
    pub client_ep: Option<Endpoint>,
}

impl IoUringServer {
    pub fn new(ring: IoUringBuffer) -> Self {
        Self { ring, client_ep: None }
    }

    pub fn set_client_notify(&mut self, ep: Endpoint) {
        self.client_ep = Some(ep);
    }

    pub fn next_request(&mut self) -> Option<IoUringSqe> {
        self.ring.pop_sqe()
    }

    pub fn complete(&mut self, user_data: usize, res: i32) -> Result<(), Error> {
        let cqe = IoUringCqe { user_data, res, flags: 0 };
        self.ring.push_cqe(cqe).map_err(|_| Error::OutOfMemory)?;

        if let Some(ep) = self.client_ep {
            ep.notify(Badge::new(NOTIFY_IO_URING_CQ))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IoUringClient {
    pub ring: IoUringBuffer,
    pub server_ep: Option<Endpoint>,
}

impl IoUringClient {
    pub fn new(ring: IoUringBuffer) -> Self {
        Self { ring, server_ep: None }
    }

    pub fn set_server_notify(&mut self, ep: Endpoint) {
        self.server_ep = Some(ep);
    }

    pub fn submit(&self, sqe: IoUringSqe) -> Result<(), Error> {
        self.ring.push_sqe(sqe).map_err(|_| Error::OutOfMemory)?;
        self.notify_sq()
    }

    pub fn notify_sq(&self) -> Result<(), Error> {
        if let Some(ep) = self.server_ep {
            ep.notify(Badge::new(NOTIFY_IO_URING_SQ))?;
        }
        Ok(())
    }

    pub fn pop_completion(&self) -> Option<IoUringCqe> {
        self.ring.pop_cqe()
    }

    pub fn peek_completion(&self) -> Option<IoUringCqe> {
        let header = self.ring.header();
        let head = header.cq_head.load(Ordering::Acquire);
        let tail = header.cq_tail.load(Ordering::Acquire);

        if head == tail {
            return None;
        }

        let index = head & header.cq_mask;
        let cqe = unsafe { *self.ring.cqes_mut().add(index as usize) };
        Some(cqe)
    }

    pub fn wait_for_completions(&self, ep: &Endpoint) -> Result<(), Error> {
        if self.ring.cq_len() > 0 {
            return Ok(());
        }
        let mut utcb = unsafe { crate::ipc::UTCB::new() };
        utcb.clear();
        ep.recv(&mut utcb)?;
        Ok(())
    }
}
#[cfg(target_pointer_width = "64")]
pub const NOTIFY_IO_URING_SQ: usize = 1 << 33;
#[cfg(target_pointer_width = "32")]
pub const NOTIFY_IO_URING_SQ: usize = 1 << 29;
#[cfg(target_pointer_width = "64")]
pub const NOTIFY_IO_URING_CQ: usize = 1 << 34;
#[cfg(target_pointer_width = "32")]
pub const NOTIFY_IO_URING_CQ: usize = 1 << 30;
