use super::condvar::Condvar;
use super::mutex::Mutex;
use alloc::collections::VecDeque;
use alloc::sync::Arc;

struct Shared<T> {
    queue: Mutex<VecDeque<T>>,
    capacity: usize,
    not_full: Condvar,
    not_empty: Condvar,
}

pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}

pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
}

// Clone for Sender (MP)
impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self { shared: self.shared.clone() }
    }
}

// Clone for Receiver (MC)
impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Self { shared: self.shared.clone() }
    }
}

pub fn bounded<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    let cap = if capacity == 0 { 1 } else { capacity };
    let shared = Arc::new(Shared {
        queue: Mutex::new(VecDeque::new()),
        capacity: cap,
        not_full: Condvar::new(),
        not_empty: Condvar::new(),
    });
    (Sender { shared: shared.clone() }, Receiver { shared })
}

impl<T> Sender<T> {
    pub fn send(&self, t: T) {
        let mut queue = self.shared.queue.lock();
        while queue.len() >= self.shared.capacity {
            queue = self.shared.not_full.wait(queue);
        }
        queue.push_back(t);
        self.shared.not_empty.notify_one();
    }
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> T {
        let mut queue = self.shared.queue.lock();
        while queue.is_empty() {
            queue = self.shared.not_empty.wait(queue);
        }
        let t = queue.pop_front().unwrap();
        self.shared.not_full.notify_one();
        t
    }

    pub fn try_recv(&self) -> Option<T> {
        let mut queue = self.shared.queue.lock();
        let t = queue.pop_front()?;
        self.shared.not_full.notify_one();
        Some(t)
    }
}
