use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::task::Wake;
use alloc::vec::Vec;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::{Context, Poll, Waker};

use crate::client::ProcessClient;
use crate::error::Error;
use crate::runtime::thread::{RuntimeThreadConfig, set_current_task, try_current_thread_context};
use crate::runtime::worker::{RuntimeWorker, spawn_worker};
use crate::sync::channel::{Receiver, Sender, bounded};
use crate::sync::mutex::Mutex;

type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

struct Task {
    future: Mutex<Option<BoxFuture>>,
    scheduler: Arc<SchedulerShared>,
    queued: AtomicBool,
}

impl Task {
    fn schedule_local(self: &Arc<Self>, worker_id: usize) -> bool {
        self.scheduler.push_local(worker_id, self.clone())
    }

    fn schedule(self: &Arc<Self>) {
        if self.queued.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_ok() {
            if let Some(context) = try_current_thread_context()
                && context.executor_ptr == self.scheduler.as_ptr()
                && self.schedule_local(context.worker_id)
            {
                return;
            }
            self.scheduler.push_global(self.clone());
        }
    }

    fn poll(self: Arc<Self>) {
        let task_ptr = Arc::as_ptr(&self) as usize;
        set_current_task(task_ptr);
        self.queued.store(false, Ordering::Release);

        let waker = Waker::from(self.clone());
        let mut cx = Context::from_waker(&waker);
        let mut future_slot = self.future.lock();
        let Some(mut future) = future_slot.take() else {
            set_current_task(0);
            return;
        };

        match future.as_mut().poll(&mut cx) {
            Poll::Pending => {
                *future_slot = Some(future);
            }
            Poll::Ready(()) => {}
        }

        set_current_task(0);
    }
}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        self.schedule();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.schedule();
    }
}

struct LocalQueue {
    tasks: Mutex<VecDeque<Arc<Task>>>,
}

impl LocalQueue {
    fn new() -> Self {
        Self { tasks: Mutex::new(VecDeque::new()) }
    }

    fn push(&self, task: Arc<Task>) {
        self.tasks.lock().push_back(task);
    }

    fn pop(&self) -> Option<Arc<Task>> {
        self.tasks.lock().pop_front()
    }
}

struct SchedulerShared {
    global_tx: Sender<Arc<Task>>,
    global_rx: Receiver<Arc<Task>>,
    local_queues: Vec<LocalQueue>,
}

impl SchedulerShared {
    fn new(global_tx: Sender<Arc<Task>>, global_rx: Receiver<Arc<Task>>, workers: usize) -> Self {
        let mut local_queues = Vec::with_capacity(workers);
        for _ in 0..workers {
            local_queues.push(LocalQueue::new());
        }
        Self { global_tx, global_rx, local_queues }
    }

    fn as_ptr(&self) -> usize {
        self as *const Self as usize
    }

    fn push_global(&self, task: Arc<Task>) {
        self.global_tx.send(task);
    }

    fn try_pop_global(&self) -> Option<Arc<Task>> {
        self.global_rx.try_recv()
    }

    fn pop_global(&self) -> Arc<Task> {
        self.global_rx.recv()
    }

    fn push_local(&self, worker_id: usize, task: Arc<Task>) -> bool {
        let Some(queue) = self.local_queues.get(worker_id) else {
            return false;
        };
        queue.push(task);
        true
    }

    fn pop_local(&self, worker_id: usize) -> Option<Arc<Task>> {
        self.local_queues.get(worker_id)?.pop()
    }
}

pub struct JoinHandle<T> {
    receiver: Receiver<T>,
}

impl<T> JoinHandle<T> {
    pub fn join(self) -> T {
        self.receiver.recv()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkerThreadSpec {
    pub stack_top: usize,
    pub thread: RuntimeThreadConfig,
}

pub struct ThreadPool {
    scheduler: Arc<SchedulerShared>,
    worker_tids: alloc::vec::Vec<usize>,
}

impl ThreadPool {
    pub fn worker_tids(&self) -> &[usize] {
        &self.worker_tids
    }

    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let (tx, rx) = bounded(1);
        let task_future = async move {
            let output = future.await;
            tx.send(output);
        };
        let task = Arc::new(Task {
            future: Mutex::new(Some(Box::pin(task_future))),
            scheduler: self.scheduler.clone(),
            queued: AtomicBool::new(false),
        });
        task.schedule();
        JoinHandle { receiver: rx }
    }

    pub fn spawn_blocking<F, T>(&self, f: F) -> JoinHandle<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        self.spawn(async move { f() })
    }
}

pub struct ThreadPoolBuilder {
    queue_capacity: usize,
}

impl ThreadPoolBuilder {
    pub const fn new() -> Self {
        Self { queue_capacity: 256 }
    }

    pub const fn with_queue_capacity(mut self, queue_capacity: usize) -> Self {
        self.queue_capacity = queue_capacity;
        self
    }

    pub fn build(
        self,
        proc_client: &mut ProcessClient,
        workers: &[WorkerThreadSpec],
    ) -> Result<ThreadPool, Error> {
        if workers.is_empty() {
            return Err(Error::InvalidArgs);
        }

        let (tx, rx) = bounded(self.queue_capacity);
        let scheduler = Arc::new(SchedulerShared::new(tx, rx, workers.len()));
        let mut tids = alloc::vec::Vec::with_capacity(workers.len());

        for worker in workers {
            let worker_thread = worker.thread.with_executor_ptr(scheduler.as_ptr());
            let config = ExecutorWorkerConfig {
                scheduler: scheduler.clone(),
                worker_id: worker_thread.worker_id,
            };
            let tid = spawn_worker::<ExecutorWorker>(
                proc_client,
                worker_thread,
                config,
                worker.stack_top,
            )?;
            tids.push(tid);
        }

        Ok(ThreadPool { scheduler, worker_tids: tids })
    }
}

struct ExecutorWorkerConfig {
    scheduler: Arc<SchedulerShared>,
    worker_id: usize,
}

struct ExecutorWorker;

impl RuntimeWorker for ExecutorWorker {
    type Config = ExecutorWorkerConfig;

    fn run(config: Self::Config) -> ! {
        const BATCH_LIMIT: usize = 32;

        loop {
            let task = config
                .scheduler
                .pop_local(config.worker_id)
                .unwrap_or_else(|| config.scheduler.pop_global());
            task.poll();

            for _ in 1..BATCH_LIMIT {
                let Some(task) = config
                    .scheduler
                    .pop_local(config.worker_id)
                    .or_else(|| config.scheduler.try_pop_global())
                else {
                    break;
                };
                task.poll();
            }
        }
    }
}
