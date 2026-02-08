use super::spinlock::SpinLock;
use super::{current_thread_park_endpoint, park, unpark};
use crate::cap::Endpoint;
use alloc::collections::VecDeque;
use core::sync::atomic::{AtomicIsize, Ordering};

pub struct Semaphore {
    count: AtomicIsize,
    waiters: SpinLock<VecDeque<Endpoint>>,
}

impl Semaphore {
    pub const fn new(count: isize) -> Self {
        Self { count: AtomicIsize::new(count), waiters: SpinLock::new(VecDeque::new()) }
    }

    pub fn down(&self) {
        // Fast path
        if self.count.fetch_sub(1, Ordering::Acquire) > 0 {
            return;
        }

        // Slow path
        let my_ep = current_thread_park_endpoint();
        {
            let mut guard = self.waiters.lock();
            guard.push_back(my_ep);
        }

        // Block
        // In a robust implementation, we handled spurious wakeups by re-checking a condition
        // or assuming strict handoff (1 signal = 1 wakeup allowed).
        // Here we assume strict handoff from up().
        park();
    }

    pub fn up(&self) {
        // Fast path (if no waiters) / Indication of wake needed
        if self.count.fetch_add(1, Ordering::Release) < 0 {
            let mut guard = self.waiters.lock();
            if let Some(ep) = guard.pop_front() {
                unpark(ep);
            }
        }
    }
}
