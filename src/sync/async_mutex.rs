use alloc::collections::VecDeque;
use core::cell::UnsafeCell;
use core::future::Future;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::{Context, Poll, Waker};

use super::mutex::Mutex;

/// An asynchronous mutual exclusion primitive.
///
/// Unlike a standard Mutex, this does not block the thread when contested.
/// Instead, it returns a Future that yields a Guard when the lock is acquired.
pub struct AsyncMutex<T: ?Sized> {
    locked: AtomicBool,
    waiters: Mutex<VecDeque<Waker>>,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Sync for AsyncMutex<T> {}
unsafe impl<T: ?Sized + Send> Send for AsyncMutex<T> {}

pub struct AsyncMutexGuard<'a, T: ?Sized> {
    lock: &'a AsyncMutex<T>,
}

impl<T> AsyncMutex<T> {
    pub const fn new(t: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            waiters: Mutex::new(VecDeque::new()),
            data: UnsafeCell::new(t),
        }
    }
}

impl<T: ?Sized> AsyncMutex<T> {
    pub fn lock(&self) -> AsyncMutexLockFuture<'_, T> {
        AsyncMutexLockFuture { lock: self }
    }

    pub fn try_lock(&self) -> Option<AsyncMutexGuard<'_, T>> {
        if self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            Some(AsyncMutexGuard { lock: self })
        } else {
            None
        }
    }

    pub fn unlock(&self) {
        let mut waiters = self.waiters.lock();
        if let Some(waker) = waiters.pop_front() {
            // Hand off the lock to the next waiter.
            // We keep 'locked' as true so no one else can steal it.
            waker.wake();
        } else {
            self.locked.store(false, Ordering::Release);
        }
    }
}

pub struct AsyncMutexLockFuture<'a, T: ?Sized> {
    lock: &'a AsyncMutex<T>,
}

impl<'a, T: ?Sized> Future for AsyncMutexLockFuture<'a, T> {
    type Output = AsyncMutexGuard<'a, T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Try to acquire the lock
        if self
            .lock
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            return Poll::Ready(AsyncMutexGuard { lock: self.lock });
        }

        // Contended: push waker to the queue
        let mut waiters = self.lock.waiters.lock();
        // Double check after taking the waiters lock
        if self
            .lock
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            return Poll::Ready(AsyncMutexGuard { lock: self.lock });
        }

        // Check if waker already in queue? For simplicity, we just push.
        waiters.push_back(cx.waker().clone());
        Poll::Pending
    }
}

impl<'a, T: ?Sized> Deref for AsyncMutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T: ?Sized> DerefMut for AsyncMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T: ?Sized> Drop for AsyncMutexGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}
