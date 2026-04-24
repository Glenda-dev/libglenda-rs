use crate::cap::{CapPtr, Endpoint};
use crate::error::Error;
use crate::ipc::ThreadControlBlock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeThreadConfig {
    pub park_endpoint: Endpoint,
    pub park_recv_slot: CapPtr,
    pub park_reply_slot: CapPtr,
    pub worker_id: usize,
    pub executor_ptr: usize,
}

impl RuntimeThreadConfig {
    pub const fn new(
        park_endpoint: Endpoint,
        park_recv_slot: CapPtr,
        park_reply_slot: CapPtr,
    ) -> Self {
        Self { park_endpoint, park_recv_slot, park_reply_slot, worker_id: 0, executor_ptr: 0 }
    }

    pub const fn with_worker_id(mut self, worker_id: usize) -> Self {
        self.worker_id = worker_id;
        self
    }

    pub const fn with_executor_ptr(mut self, executor_ptr: usize) -> Self {
        self.executor_ptr = executor_ptr;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeThreadContext {
    pub tid: usize,
    pub park_endpoint: Endpoint,
    pub park_recv_slot: CapPtr,
    pub park_reply_slot: CapPtr,
    pub worker_id: usize,
    pub executor_ptr: usize,
    pub current_task: usize,
}

fn current_tcb_mut() -> &'static mut ThreadControlBlock {
    let tp = crate::arch::thread::get_thread_pointer();
    assert!(tp != 0, "runtime thread APIs require an initialized thread pointer");
    unsafe { &mut *(tp as *mut ThreadControlBlock) }
}

fn current_tcb() -> &'static ThreadControlBlock {
    let tp = crate::arch::thread::get_thread_pointer();
    assert!(tp != 0, "runtime thread APIs require an initialized thread pointer");
    unsafe { &*(tp as *const ThreadControlBlock) }
}

fn try_current_tcb() -> Option<&'static ThreadControlBlock> {
    let tp = crate::arch::thread::get_thread_pointer();
    if tp == 0 {
        return None;
    }
    Some(unsafe { &*(tp as *const ThreadControlBlock) })
}

pub fn init_current_thread(config: RuntimeThreadConfig) -> Result<(), Error> {
    if config.park_endpoint.cap().is_null() {
        return Err(Error::InvalidCapability);
    }

    let tcb = current_tcb_mut();
    tcb.park_ep = config.park_endpoint;
    tcb.park_recv_slot = config.park_recv_slot;
    tcb.park_reply_slot = config.park_reply_slot;
    tcb.worker_id = config.worker_id;
    tcb.executor_ptr = config.executor_ptr;
    Ok(())
}

pub fn current_thread_context() -> RuntimeThreadContext {
    let tcb = current_tcb();
    RuntimeThreadContext {
        tid: tcb.tid,
        park_endpoint: tcb.park_ep,
        park_recv_slot: tcb.park_recv_slot,
        park_reply_slot: tcb.park_reply_slot,
        worker_id: tcb.worker_id,
        executor_ptr: tcb.executor_ptr,
        current_task: tcb.current_task,
    }
}

pub fn try_current_thread_context() -> Option<RuntimeThreadContext> {
    let tcb = try_current_tcb()?;
    Some(RuntimeThreadContext {
        tid: tcb.tid,
        park_endpoint: tcb.park_ep,
        park_recv_slot: tcb.park_recv_slot,
        park_reply_slot: tcb.park_reply_slot,
        worker_id: tcb.worker_id,
        executor_ptr: tcb.executor_ptr,
        current_task: tcb.current_task,
    })
}

pub fn current_thread_id() -> usize {
    current_tcb().tid
}

pub fn set_current_task(task_ptr: usize) {
    current_tcb_mut().current_task = task_ptr;
}

pub fn set_current_task_ptr(task_ptr: usize) {
    set_current_task(task_ptr);
}
