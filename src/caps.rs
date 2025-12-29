use crate::syscall::*;
use crate::types::*;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapPtr(pub usize);

impl CapPtr {
    pub const fn new(cptr: usize) -> Self {
        Self(cptr)
    }

    pub fn bits(&self) -> usize {
        self.0
    }

    // --- Generic Invocation ---
    pub fn invoke(&self, method: usize, args: [usize; 6]) -> usize {
        sys_invoke(self.0, method, args[0], args[1], args[2], args[3], args[4], args[5])
    }

    // --- TCB Methods ---
    pub fn tcb_configure(
        &self,
        cspace: CapPtr,
        vspace: CapPtr,
        utcb: usize,
        fault_ep: CapPtr,
        utcb_frame: CapPtr,
    ) -> usize {
        self.invoke(tcbmethod::CONFIGURE, [cspace.0, vspace.0, utcb, fault_ep.0, utcb_frame.0, 0])
    }

    pub fn tcb_set_priority(&self, priority: usize) -> usize {
        self.invoke(tcbmethod::SET_PRIORITY, [priority, 0, 0, 0, 0, 0])
    }

    pub fn tcb_resume(&self) -> usize {
        self.invoke(tcbmethod::RESUME, [0, 0, 0, 0, 0, 0])
    }

    pub fn tcb_suspend(&self) -> usize {
        self.invoke(tcbmethod::SUSPEND, [0, 0, 0, 0, 0, 0])
    }

    // --- CNode Methods ---
    pub fn cnode_mint(&self, src: CapPtr, dest_slot: usize, badge: usize, rights: u8) -> usize {
        self.invoke(cnodemethod::MINT, [src.0, dest_slot, badge, rights as usize, 0, 0])
    }

    pub fn cnode_copy(&self, src: CapPtr, dest_slot: usize, rights: u8) -> usize {
        self.invoke(cnodemethod::COPY, [src.0, dest_slot, rights as usize, 0, 0, 0])
    }

    pub fn cnode_delete(&self, slot: usize) -> usize {
        self.invoke(cnodemethod::DELETE, [slot, 0, 0, 0, 0, 0])
    }

    pub fn cnode_revoke(&self, slot: usize) -> usize {
        self.invoke(cnodemethod::REVOKE, [slot, 0, 0, 0, 0, 0])
    }

    // --- Untyped Methods ---
    pub fn untyped_retype(
        &self,
        obj_type: CapType,
        size_bits: usize,
        n_objs: usize,
        dest_cnode: CapPtr,
        dest_offset: usize,
    ) -> usize {
        self.invoke(
            untypedmethod::RETYPE,
            [obj_type as usize, size_bits, n_objs, dest_cnode.0, dest_offset, 0],
        )
    }

    // --- PageTable Methods ---
    pub fn pagetable_map(&self, frame: CapPtr, vaddr: usize, rights: usize) -> usize {
        self.invoke(pagetablemethod::MAP, [frame.0, vaddr, rights, 0, 0, 0])
    }

    pub fn pagetable_unmap(&self, vaddr: usize) -> usize {
        self.invoke(pagetablemethod::UNMAP, [vaddr, 0, 0, 0, 0, 0])
    }

    // --- IPC Methods ---
    pub fn ipc_send(&self, msg_info: MsgTag, args: &[usize]) -> usize {
        self.invoke(
            ipcmethod::SEND,
            [msg_info.as_usize(), args[0], args[1], args[2], args[3], args[4]],
        )
    }

    pub fn ipc_recv(&self) -> usize {
        self.invoke(ipcmethod::RECV, [0, 0, 0, 0, 0, 0])
    }

    pub fn ipc_call(&self, msg_info: MsgTag, args: &[usize]) -> usize {
        self.invoke(
            ipcmethod::CALL,
            [msg_info.as_usize(), args[0], args[1], args[2], args[3], args[4]],
        )
    }

    pub fn ipc_notify(&self, badge: usize) -> usize {
        self.invoke(ipcmethod::NOTIFY, [badge, 0, 0, 0, 0, 0])
    }

    pub fn ipc_reply(&self, msg_info: MsgTag, args: &[usize]) -> usize {
        self.invoke(
            replymethod::REPLY,
            [msg_info.as_usize(), args[0], args[1], args[2], args[3], args[4]],
        )
    }
}
