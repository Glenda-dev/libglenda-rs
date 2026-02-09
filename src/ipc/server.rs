use crate::cap::CapPtr;
use crate::error::Error;
use crate::ipc::{MsgFlags, MsgTag, UTCB};
use crate::protocol;
use crate::set_mrs;

/// Helper trait to convert return values into UTCB Message Registers.
pub trait IpcReturn {
    fn to_mrs(&self, utcb: &mut UTCB);
}

impl IpcReturn for () {
    fn to_mrs(&self, _utcb: &mut UTCB) {}
}

impl IpcReturn for usize {
    fn to_mrs(&self, utcb: &mut UTCB) {
        set_mrs!(utcb, *self);
    }
}

impl IpcReturn for isize {
    fn to_mrs(&self, utcb: &mut UTCB) {
        set_mrs!(utcb, *self as usize);
    }
}

impl IpcReturn for CapPtr {
    fn to_mrs(&self, utcb: &mut UTCB) {
        utcb.set_cap_transfer(*self);
    }
}

impl IpcReturn for (usize, usize) {
    fn to_mrs(&self, utcb: &mut UTCB) {
        set_mrs!(utcb, self.0, self.1);
    }
}

/// A wrapper to handle IPC requests.
///
/// It executes the provided closure `f`.
/// - If `f` returns `Ok(val)`, `val` is written to UTCB and `MsgTag::ok()` is set.
/// - If `f` returns `Err(e)`, the error is propagated (usually causing the caller to set `MsgTag::err()`).
pub fn handle_call<T, F>(utcb: &mut UTCB, f: F) -> Result<(), Error>
where
    F: FnOnce(&mut UTCB) -> Result<T, Error>,
    T: IpcReturn,
{
    match f(utcb) {
        Ok(val) => {
            val.to_mrs(utcb);
            // Only set OK if it's not already set?
            // Usually we overwrite whatever input tag was there with OK.
            // If the return type included a specific MsgTag, we might want to respect that.
            // But for standard RPC, OK is correct.
            // Note: CapPtr::to_mrs sets cap_transfer, but we also need to set HAS_CAP flag?
            // Existing logic in warren/server.rs manually constructs MsgTag for caps.
            // We might need to enhance IpcReturn to return (MsgTag, ...) or handle flags.

            // Simple default: OK
            utcb.set_msg_tag(MsgTag::ok());
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Special handler for responses that include Capabilities.
/// Should be used when the return value is meant to be transferred as a capability.
pub fn handle_cap_call<F>(utcb: &mut UTCB, f: F) -> Result<(), Error>
where
    F: FnOnce(&mut UTCB) -> Result<CapPtr, Error>,
{
    match f(utcb) {
        Ok(cap) => {
            utcb.set_cap_transfer(cap);
            utcb.set_msg_tag(MsgTag::new(
                protocol::GENERIC_PROTO,
                protocol::generic::REPLY,
                MsgFlags::OK | MsgFlags::HAS_CAP,
            ));
            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub fn handle_buffer_call<F>(utcb: &mut UTCB, f: F) -> Result<(), Error>
where
    F: FnOnce() -> Result<(), Error>,
{
    match f() {
        Ok(val) => {
            val.to_mrs(utcb);
            utcb.set_msg_tag(MsgTag::new(
                protocol::GENERIC_PROTO,
                protocol::generic::REPLY,
                MsgFlags::OK | MsgFlags::HAS_BUFFER,
            ));
            Ok(())
        }
        Err(e) => Err(e),
    }
}
