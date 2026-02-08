use super::mutex::MutexGuard;
use super::spinlock::SpinLock;
use super::{current_thread_park_endpoint, park, unpark};
use crate::cap::Endpoint;
use alloc::collections::VecDeque;

pub struct Condvar {
    waiters: SpinLock<VecDeque<Endpoint>>,
}

impl Condvar {
    pub const fn new() -> Self {
        Self { waiters: SpinLock::new(VecDeque::new()) }
    }

    pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
        let mutex = guard.lock;
        let my_ep = current_thread_park_endpoint();

        {
            let mut waiters = self.waiters.lock();
            waiters.push_back(my_ep);
        }

        // Drop guard (releasing lock) without running destructor if we manually unlock?
        // Actually, guard.drop() calls unlock().
        // So dropping guard is exactly what we want.
        drop(guard);

        park();

        mutex.lock()
    }

    pub fn notify_one(&self) {
        let mut waiters = self.waiters.lock();
        if let Some(ep) = waiters.pop_front() {
            unpark(ep);
        }
    }

    pub fn notify_all(&self) {
        let mut waiters = self.waiters.lock();
        while let Some(ep) = waiters.pop_front() {
            unpark(ep);
        }
    }
}
