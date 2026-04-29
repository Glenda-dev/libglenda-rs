use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt;
use core::future::Future;
use core::pin::Pin;

use crate::cap::{CNode, CSPACE_CAP, CapPtr, Endpoint, Reply, Rights};
use crate::error::Error;
use crate::ipc::{Badge, MAX_MRS, MsgFlags, MsgTag, UTCB};
use crate::runtime::executor::ThreadPool;
use crate::sync::notification::NotificationReactor;

/// Metadata about the current RPC request.
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub pid: Option<usize>,
    pub badge: Badge,
    pub deadline_ns: Option<u64>,
}

/// Trait implemented by services to handle asynchronous RPC requests.
pub trait AsyncRpcHandler: Send + Sync + 'static {
    fn handle(
        &self,
        ctx: RequestContext,
        request: RpcRequest,
    ) -> Pin<Box<dyn Future<Output = Result<RpcReply, Error>> + Send>>;
}

/// Trait for allocating and freeing capability slots.
pub trait SlotAllocator: Send + Sync + fmt::Debug + 'static {
    fn alloc(&self) -> Result<CapPtr, Error>;
    fn free(&self, slot: CapPtr);
}

/// A router that dispatches requests to different handlers based on their MsgTag.
pub struct RpcRouter {
    handlers: BTreeMap<(usize, usize), Arc<dyn AsyncRpcHandler>>,
    default_handler: Option<Arc<dyn AsyncRpcHandler>>,
}

impl RpcRouter {
    pub fn new() -> Self {
        Self { handlers: BTreeMap::new(), default_handler: None }
    }

    pub fn register<H: AsyncRpcHandler>(&mut self, proto: usize, label: usize, handler: H) {
        self.handlers.insert((proto, label), Arc::new(handler));
    }

    pub fn set_default<H: AsyncRpcHandler>(&mut self, handler: H) {
        self.default_handler = Some(Arc::new(handler));
    }

    pub fn route(&self, tag: MsgTag) -> Option<Arc<dyn AsyncRpcHandler>> {
        self.handlers
            .get(&(tag.proto(), tag.label()))
            .cloned()
            .or_else(|| self.default_handler.clone())
    }
}

#[derive(Debug, Clone)]
pub struct RpcRequest {
    badge: Badge,
    tag: MsgTag,
    mrs: [usize; MAX_MRS],
    mrs_count: usize,
    reply: DeferredReply,
}

impl RpcRequest {
    pub fn from_utcb(utcb: &UTCB, reply: DeferredReply) -> Self {
        Self {
            badge: utcb.get_badge(),
            tag: utcb.get_msg_tag(),
            mrs: utcb.get_mrs(),
            mrs_count: utcb.get_mrs_count(),
            reply,
        }
    }

    pub fn badge(&self) -> Badge {
        self.badge
    }

    pub fn tag(&self) -> MsgTag {
        self.tag
    }

    pub fn mrs(&self) -> [usize; MAX_MRS] {
        self.mrs
    }

    pub fn mr(&self, index: usize) -> usize {
        self.mrs.get(index).copied().unwrap_or(0)
    }

    pub fn mrs_count(&self) -> usize {
        self.mrs_count
    }

    pub fn reply_handle(&self) -> DeferredReply {
        self.reply.clone()
    }
}

#[derive(Debug, Clone)]
pub struct DeferredReply {
    inner: Arc<DeferredReplyInner>,
}

struct DeferredReplyInner {
    slot: CapPtr,
    allocator: Arc<dyn SlotAllocator>,
}

impl fmt::Debug for DeferredReplyInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeferredReplyInner")
            .field("slot", &self.slot)
            .field("allocator", &"Arc<dyn SlotAllocator>")
            .finish()
    }
}

impl DeferredReply {
    pub fn new(slot: CapPtr, allocator: Arc<dyn SlotAllocator>) -> Self {
        Self { inner: Arc::new(DeferredReplyInner { slot, allocator }) }
    }

    pub fn reply(self, message: RpcReply) -> Result<(), Error> {
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(message.tag);
        for i in 0..message.mrs_count.min(MAX_MRS) {
            utcb.set_mr(i, message.mrs[i]);
        }
        if !message.buffer.is_empty() {
            utcb.write(&message.buffer);
        }
        Reply::from(self.inner.slot).reply(utcb)
    }

    pub fn reply_ok(self) -> Result<(), Error> {
        self.reply(RpcReply::ok())
    }

    pub fn reply_error(self, error: Error) -> Result<(), Error> {
        self.reply(RpcReply::err(error))
    }
}

impl Drop for DeferredReplyInner {
    fn drop(&mut self) {
        // Ensure the capability is deleted and slot is freed
        let _ = CSPACE_CAP.delete(self.slot);
        self.allocator.free(self.slot);
    }
}

#[derive(Debug, Clone)]
pub struct RpcReply {
    tag: MsgTag,
    mrs: [usize; MAX_MRS],
    mrs_count: usize,
    buffer: Vec<u8>,
}

impl RpcReply {
    pub fn ok() -> Self {
        Self {
            tag: MsgTag::new(
                crate::protocol::GENERIC_PROTO,
                crate::protocol::generic::REPLY,
                MsgFlags::OK,
            ),
            mrs: [0; MAX_MRS],
            mrs_count: 0,
            buffer: Vec::new(),
        }
    }

    pub fn err(error: Error) -> Self {
        let mut reply = Self {
            tag: MsgTag::new(
                crate::protocol::GENERIC_PROTO,
                crate::protocol::generic::REPLY,
                MsgFlags::ERROR | MsgFlags::HAS_MRS,
            ),
            mrs: [0; MAX_MRS],
            mrs_count: 1,
            buffer: Vec::new(),
        };
        reply.mrs[0] = error as usize;
        reply
    }

    pub fn with_mr(mut self, value: usize) -> Self {
        if self.mrs_count < MAX_MRS {
            self.mrs[self.mrs_count] = value;
            self.mrs_count += 1;
            self.tag = MsgTag::new(
                self.tag.proto(),
                self.tag.label(),
                self.tag.flags() | MsgFlags::HAS_MRS,
            );
        }
        self
    }

    pub fn with_buffer(mut self, buffer: &[u8]) -> Self {
        self.buffer.clear();
        self.buffer.extend_from_slice(buffer);
        self.tag = MsgTag::new(
            self.tag.proto(),
            self.tag.label(),
            self.tag.flags() | MsgFlags::HAS_BUFFER,
        );
        self
    }
}

pub struct AsyncRpcServer {
    router: Arc<RpcRouter>,
    pool: Arc<ThreadPool>,
    allocator: Arc<dyn SlotAllocator>,
    notification_reactor: Arc<NotificationReactor>,
}

impl AsyncRpcServer {
    pub fn new(
        router: RpcRouter,
        pool: Arc<ThreadPool>,
        allocator: Arc<dyn SlotAllocator>,
        notification_reactor: Arc<NotificationReactor>,
    ) -> Self {
        Self { router: Arc::new(router), pool, allocator, notification_reactor }
    }

    pub fn run(&self, endpoint: Endpoint) -> ! {
        let mut utcb = unsafe { UTCB::new() };
        loop {
            // Wait for request or notification
            let res = endpoint.recv(&mut utcb);
            if res.is_err() {
                continue;
            }

            let tag = utcb.get_msg_tag();
            let badge = utcb.get_badge();

            // Check if it's a notification (No label, and has badge)
            if tag.label() == 0 && badge.bits() != 0 {
                self.notification_reactor.dispatch(badge);
                continue;
            }

            // It's a Call: Atomic Reply Transfer
            let Ok(target_slot) = self.allocator.alloc() else {
                continue; // Drop request if no slots available
            };

            // Move reply cap from REPLY_SLOT to the task-private slot
            if CSPACE_CAP.transfer_self(crate::cap::REPLY_SLOT, target_slot).is_err() {
                self.allocator.free(target_slot);
                continue;
            }

            let reply_handle = DeferredReply::new(target_slot, self.allocator.clone());
            let request = RpcRequest::from_utcb(&utcb, reply_handle);
            let router = self.router.clone();

            if let Some(handler) = router.route(tag) {
                let ctx = RequestContext { pid: None, badge, deadline_ns: None };

                self.pool.spawn(async move {
                    let rh = request.reply_handle();
                    match handler.handle(ctx, request).await {
                        Ok(reply) => {
                            let _ = rh.reply(reply);
                        }
                        Err(e) => {
                            let _ = rh.reply_error(e);
                        }
                    }
                });
            } else {
                let rh = request.reply_handle();
                let _ = rh.reply_error(Error::NotFound);
            }
        }
    }
}
