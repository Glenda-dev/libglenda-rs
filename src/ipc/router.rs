use crate::error::Error;
use crate::ipc::UTCB;
use alloc::vec::Vec;

/// Type definition for an IPC message handler.
///
/// * `ctx`: Mutable reference to the service context (state).
/// * `utcb`: Mutable reference to the UTCB containing the message.
pub type Handler<T> = fn(ctx: &mut T, utcb: &mut UTCB) -> Result<(), Error>;

/// A simple router to dispatch IPC messages based on Protocol and Label.
pub struct IpcRouter<T> {
    routes: Vec<((usize, usize), Handler<T>)>,
}

impl<T> IpcRouter<T> {
    /// Create a new empty router.
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    /// Register a handler for a specific protocol and label.
    pub fn register(&mut self, proto: usize, label: usize, handler: Handler<T>) {
        self.routes.push(((proto, label), handler));
    }

    /// Dispatch an incoming message to the appropriate handler.
    ///
    /// The protocol and label are extracted from the UTCB's message tag.
    pub fn dispatch(&self, ctx: &mut T, utcb: &mut UTCB) -> Result<(), Error> {
        let tag = utcb.get_msg_tag();
        let proto = tag.proto();
        let label = tag.label();

        for ((p, l), handler) in &self.routes {
            if *p == proto && *l == label {
                return handler(ctx, utcb);
            }
        }
        Err(Error::InvalidMethod)
    }
}
