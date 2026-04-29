use super::spinlock::SpinLock;
use super::{current_thread_park_endpoint, park, unpark};
use crate::cap::Endpoint;
use alloc::collections::VecDeque;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug)]
pub struct Mutex<T: ?Sized> {
    state: AtomicU8, // 0: unlocked, 1: locked, 2: locked with waiters
    waiters: SpinLock<VecDeque<Endpoint>>,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}

pub struct MutexGuard<'a, T: ?Sized> {
    pub lock: &'a Mutex<T>,
}

impl<T> Mutex<T> {
    pub const fn new(t: T) -> Self {
        Self {
            state: AtomicU8::new(0),
            waiters: SpinLock::new(VecDeque::new()),
            data: UnsafeCell::new(t),
        }
    }
}

impl<T: ?Sized> Mutex<T> {
    pub fn lock(&self) -> MutexGuard<'_, T> {
        if self.state.compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed).is_ok() {
            return MutexGuard { lock: self };
        }
        self.lock_slow();
        MutexGuard { lock: self }
    }

    fn lock_slow(&self) {
        let my_ep = current_thread_park_endpoint();
        loop {
            let s = self.state.load(Ordering::Relaxed);
            if s == 0 {
                if self.state.compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed).is_ok() {
                    return;
                }
                continue;
            }
            // Mark as contended
            if s == 1 {
                if self.state.compare_exchange(1, 2, Ordering::Relaxed, Ordering::Relaxed).is_err()
                {
                    continue;
                }
            }

            {
                let mut guard = self.waiters.lock();
                // Check again in case unlock happened before we pushed?
                // Standard double-checked locking
                if self.state.load(Ordering::Relaxed) == 0 {
                    continue;
                }
                guard.push_back(my_ep);
            }

            park();
        }
    }

    pub fn unlock(&self) {
        if self.state.swap(0, Ordering::Release) == 1 {
            return;
        }

        // Wake up a waiter
        let mut guard = self.waiters.lock();
        if let Some(ep) = guard.pop_front() {
            unpark(ep);
        }
    }
}

impl<'a, T: ?Sized> Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}
impl<'a, T: ?Sized> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}
impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}
