use crate::ipc::Badge;
use crate::sync::mutex::Mutex;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};

/// A reactor that maps kernel notification badges to async wakers.
pub struct NotificationReactor {
    waiters: Mutex<BTreeMap<Badge, Waker>>,
}

impl NotificationReactor {
    pub fn new() -> Self {
        Self { waiters: Mutex::new(BTreeMap::new()) }
    }

    /// Register a waker for a specific badge.
    pub fn register(&self, badge: Badge, waker: Waker) {
        self.waiters.lock().insert(badge, waker);
    }

    /// Dispatch a notification to the registered waker.
    pub fn dispatch(&self, badge: Badge) -> bool {
        let mut waiters = self.waiters.lock();
        if let Some(waker) = waiters.remove(&badge) {
            waker.wake();
            true
        } else {
            false
        }
    }
}

/// A future that resolves when a specific kernel notification (badge) is received.
pub struct NotificationFuture {
    reactor: Arc<NotificationReactor>,
    badge: Badge,
    registered: bool,
}

impl NotificationFuture {
    pub fn new(reactor: Arc<NotificationReactor>, badge: Badge) -> Self {
        Self { reactor, badge, registered: false }
    }
}

impl Future for NotificationFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let was_registered = self.registered;
        self.registered = true;

        let mut waiters = self.reactor.waiters.lock();
        // If the badge is no longer in the map, it means dispatch() was called.
        if was_registered && !waiters.contains_key(&self.badge) {
            return Poll::Ready(());
        }

        // Register/Re-register waker
        waiters.insert(self.badge, cx.waker().clone());
        Poll::Pending
    }
}
