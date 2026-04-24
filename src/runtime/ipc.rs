use alloc::vec::Vec;

use crate::cap::Reply;
use crate::error::Error;
use crate::ipc::{Badge, MAX_MRS, MsgFlags, MsgTag, UTCB};

#[derive(Debug, Clone)]
pub struct RpcRequest {
    badge: Badge,
    tag: MsgTag,
    mrs: [usize; MAX_MRS],
    mrs_count: usize,
    buffer: Vec<u8>,
    reply: DeferredReply,
}

impl RpcRequest {
    pub fn from_utcb(utcb: &UTCB, reply: Reply) -> Self {
        let buffer = utcb.buffer().to_vec();
        Self {
            badge: utcb.get_badge(),
            tag: utcb.get_msg_tag(),
            mrs: utcb.get_mrs(),
            mrs_count: utcb.get_mrs_count(),
            buffer,
            reply: DeferredReply::new(reply),
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

    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    pub fn into_reply(self) -> DeferredReply {
        self.reply
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DeferredReply {
    reply: Reply,
}

impl DeferredReply {
    pub const fn new(reply: Reply) -> Self {
        Self { reply }
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
        self.reply.reply(utcb)
    }

    pub fn reply_ok(self) -> Result<(), Error> {
        self.reply(RpcReply::ok())
    }

    pub fn reply_error(self, error: Error) -> Result<(), Error> {
        self.reply(RpcReply::err(error))
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
