use alloc::boxed::Box;

use crate::cap::{CapPtr, Endpoint};
use crate::client::ProcessClient;
use crate::error::Error;
use crate::interface::ThreadService;
use crate::ipc::{Badge, ThreadControlBlock};
use crate::vfs::{FsNamespace, FsRpcServer};

pub trait VfsWorkerServer: Send {
    fn run(&self, endpoint: Endpoint, reply_slot: CapPtr, recv_slot: CapPtr) -> Result<(), Error>;
}

pub struct FsRpcWorker<N: FsNamespace> {
    inner: FsRpcServer<N>,
}

impl<N: FsNamespace> FsRpcWorker<N> {
    pub fn new(namespace: N) -> Self {
        Self { inner: FsRpcServer::new(namespace) }
    }
}

impl<N: FsNamespace> VfsWorkerServer for FsRpcWorker<N> {
    fn run(&self, endpoint: Endpoint, reply_slot: CapPtr, recv_slot: CapPtr) -> Result<(), Error> {
        self.inner.run(endpoint, reply_slot, recv_slot)
    }
}

pub trait VfsWorkerFactory: Send + 'static {
    type Kind: Copy + Send + 'static;
    fn create_server(kind: Self::Kind) -> Box<dyn VfsWorkerServer>;
}

#[repr(C)]
pub struct VfsWorkerConfig<F: VfsWorkerFactory> {
    pub endpoint: Endpoint,
    pub reply_slot: CapPtr,
    pub recv_slot: CapPtr,
    pub kind: F::Kind,
}

impl<F: VfsWorkerFactory> VfsWorkerConfig<F> {
    pub fn leak(self) -> usize {
        Box::leak(Box::new(self)) as *mut VfsWorkerConfig<F> as usize
    }
}

pub extern "C" fn vfs_worker_entry<F: VfsWorkerFactory>(arg: usize, tid: usize) -> ! {
    unsafe {
        let tp = crate::arch::thread::get_thread_pointer();
        if tp != 0 {
            let tcb = &mut *(tp as *mut ThreadControlBlock);
            tcb.self_ptr = tp;
            tcb.tid = tid;
        }
    }

    let cfg = unsafe { &*(arg as *const VfsWorkerConfig<F>) };
    let server = F::create_server(cfg.kind);

    loop {
        let _ = server.run(cfg.endpoint, cfg.reply_slot, cfg.recv_slot);
    }
}

pub fn spawn_vfs_worker<F: VfsWorkerFactory>(
    proc_client: &mut ProcessClient,
    cfg: VfsWorkerConfig<F>,
    stack_top: usize,
) -> Result<usize, Error> {
    let tls = Box::leak(Box::new(ThreadControlBlock::new())) as *mut ThreadControlBlock as usize;

    proc_client.thread_create(
        Badge::null(),
        vfs_worker_entry::<F> as *const () as usize,
        cfg.leak(),
        stack_top,
        tls,
    )
}
