use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use crate::client::TimeClient;
use crate::ipc::Badge;
use crate::cap::{Endpoint, CapPtr};
use crate::interface::time::TimeService;

// In Glenda, typical root tasks have these reserved slots.
// For user services, the time capability is often provided during bootstrap.
// We use a common convention here, or it should be passed in.
const TIME_EP: Endpoint = Endpoint::from(CapPtr::from(11));

/// A future that resolves after a specified duration in milliseconds.
pub struct SleepFuture {
    ms: usize,
    started: bool,
}

impl SleepFuture {
    pub const fn new(ms: usize) -> Self {
        Self { ms, started: false }
    }
}

impl Future for SleepFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.started {
            // In a real async system, we would register a timer callback with a reactor.
            // For Glenda's current architecture, we use the TimeClient's synchronous sleep
            // if we are running in a dedicated worker thread, OR we need a reactor.
            let mut client = TimeClient::new(TIME_EP);
            let _ = client.sleep(Badge::null(), self.ms);
            self.started = true;
            Poll::Ready(())
        } else {
            Poll::Ready(())
        }
    }
}

/// Asynchronously sleep for the given duration in milliseconds.
pub fn sleep(ms: usize) -> SleepFuture {
    SleepFuture::new(ms)
}
