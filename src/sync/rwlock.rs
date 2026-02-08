use super::condvar::Condvar;
use super::mutex::Mutex;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

pub struct RwLock<T: ?Sized> {
    state: Mutex<State>,
    read_cond: Condvar,
    write_cond: Condvar,
    data: UnsafeCell<T>,
}

struct State {
    readers: usize,
    writer: bool,
    pending_writers: usize,
}

unsafe impl<T: ?Sized + Send + Sync> Sync for RwLock<T> {}
unsafe impl<T: ?Sized + Send + Sync> Send for RwLock<T> {}

pub struct RwLockReadGuard<'a, T: ?Sized> {
    lock: &'a RwLock<T>,
}

pub struct RwLockWriteGuard<'a, T: ?Sized> {
    lock: &'a RwLock<T>,
}

impl<T> RwLock<T> {
    pub const fn new(t: T) -> Self {
        Self {
            state: Mutex::new(State { readers: 0, writer: false, pending_writers: 0 }),
            read_cond: Condvar::new(),
            write_cond: Condvar::new(),
            data: UnsafeCell::new(t),
        }
    }
}

impl<T: ?Sized> RwLock<T> {
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        let mut state = self.state.lock();
        while state.writer || state.pending_writers > 0 {
            state = self.read_cond.wait(state);
        }
        state.readers += 1;
        RwLockReadGuard { lock: self }
    }

    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        let mut state = self.state.lock();
        state.pending_writers += 1;
        while state.writer || state.readers > 0 {
            state = self.write_cond.wait(state);
        }
        state.pending_writers -= 1;
        state.writer = true;
        RwLockWriteGuard { lock: self }
    }

    fn unlock_read(&self) {
        let mut state = self.state.lock();
        state.readers -= 1;
        if state.readers == 0 && state.pending_writers > 0 {
            self.write_cond.notify_one();
        }
    }

    fn unlock_write(&self) {
        let mut state = self.state.lock();
        state.writer = false;
        if state.pending_writers > 0 {
            self.write_cond.notify_one();
        } else {
            self.read_cond.notify_all();
        }
    }
}

impl<'a, T: ?Sized> Deref for RwLockReadGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}
impl<T: ?Sized> Drop for RwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.unlock_read();
    }
}

impl<'a, T: ?Sized> Deref for RwLockWriteGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}
impl<'a, T: ?Sized> DerefMut for RwLockWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}
impl<T: ?Sized> Drop for RwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.unlock_write();
    }
}
