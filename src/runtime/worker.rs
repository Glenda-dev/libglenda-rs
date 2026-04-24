use alloc::boxed::Box;
use core::marker::PhantomData;

use crate::client::ProcessClient;
use crate::error::Error;
use crate::interface::ThreadService;
use crate::ipc::ThreadControlBlock;
use crate::runtime::thread::{RuntimeThreadConfig, init_current_thread};

pub trait RuntimeWorker: Send + 'static {
    type Config: Send + 'static;

    fn run(config: Self::Config) -> !;
}

#[repr(C)]
struct WorkerBootstrap<W: RuntimeWorker> {
    thread: RuntimeThreadConfig,
    config: W::Config,
    _marker: PhantomData<W>,
}

impl<W: RuntimeWorker> WorkerBootstrap<W> {
    fn leak(self) -> usize {
        Box::leak(Box::new(self)) as *mut WorkerBootstrap<W> as usize
    }
}

extern "C" fn runtime_worker_entry<W: RuntimeWorker>(arg: usize, tid: usize) -> ! {
    unsafe {
        let tp = crate::arch::thread::get_thread_pointer();
        if tp != 0 {
            let tcb = &mut *(tp as *mut ThreadControlBlock);
            tcb.self_ptr = tp;
            tcb.tid = tid;
        }
    }

    let bootstrap = unsafe { Box::from_raw(arg as *mut WorkerBootstrap<W>) };
    let _ = init_current_thread(bootstrap.thread);
    W::run(bootstrap.config)
}

pub fn spawn_worker<W: RuntimeWorker>(
    proc_client: &mut ProcessClient,
    thread: RuntimeThreadConfig,
    config: W::Config,
    stack_top: usize,
) -> Result<usize, Error> {
    let tls = Box::leak(Box::new(ThreadControlBlock::new())) as *mut ThreadControlBlock as usize;
    let bootstrap = WorkerBootstrap::<W> { thread, config, _marker: PhantomData };

    proc_client.thread_create(
        crate::ipc::Badge::null(),
        runtime_worker_entry::<W> as *const () as usize,
        bootstrap.leak(),
        stack_top,
        tls,
    )
}
