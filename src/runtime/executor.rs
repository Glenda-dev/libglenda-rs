use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::task::Wake;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};

use crate::client::ProcessClient;
use crate::error::Error;
use crate::runtime::thread::{set_current_task, RuntimeThreadConfig};
use crate::runtime::worker::{spawn_worker, RuntimeWorker};
use crate::sync::channel::{bounded, Receiver, Sender};
use crate::sync::mutex::Mutex;

type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

struct Task {
    future: Mutex<Option<BoxFuture>>,
    queue: Sender<Arc<Task>>,
}

impl Task {
    fn poll(self: Arc<Self>) {
        let task_ptr = Arc::as_ptr(&self) as usize;
        set_current_task(task_ptr);

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
        self.queue.send(self.clone());
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.queue.send(self.clone());
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
    queue: Sender<Arc<Task>>,
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
        let task = Arc::new(Task { future: Mutex::new(Some(Box::pin(task_future))), queue: self.queue.clone() });
        self.queue.send(task);
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
        let mut tids = alloc::vec::Vec::with_capacity(workers.len());

        for worker in workers {
            let config = ExecutorWorkerConfig { receiver: rx.clone() };
            let tid = spawn_worker::<ExecutorWorker>(
                proc_client,
                worker.thread,
                config,
                worker.stack_top,
            )?;
            tids.push(tid);
        }

        Ok(ThreadPool { queue: tx, worker_tids: tids })
    }
}

struct ExecutorWorkerConfig {
    receiver: Receiver<Arc<Task>>,
}

struct ExecutorWorker;

impl RuntimeWorker for ExecutorWorker {
    type Config = ExecutorWorkerConfig;

    fn run(config: Self::Config) -> ! {
        loop {
            let task = config.receiver.recv();
            task.poll();
        }
    }
}
