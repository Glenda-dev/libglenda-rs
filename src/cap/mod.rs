pub mod method;
pub mod pagetable;

pub use method::*;

use crate::ipc::MAX_MRS;
use crate::ipc::MsgTag;
use crate::ipc::utcb;
use crate::syscall::{sys_invoke, sys_invoke_recv};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapPtr(pub usize);
pub type Args = [usize; MAX_MRS];

#[derive(Debug, Clone, Copy)]
#[repr(usize)]
pub enum CapType {
    CNode = 1,
    TCB = 2,
    Endpoint = 3,
    Frame = 4,
    PageTable = 5,
    Console = 6,
    IrqHandler = 7,
}

impl CapPtr {
    pub const fn null() -> Self {
        Self(0)
    }

    pub const fn new(cptr: usize) -> Self {
        Self(cptr)
    }

    pub fn bits(&self) -> usize {
        self.0
    }

    // --- Generic Invocation ---
    pub fn invoke(&self, method: usize, args: Args) -> usize {
        sys_invoke(self.0, method, args[0], args[1], args[2], args[3], args[4], args[5], args[6])
    }

    // --- TCB Methods ---
    pub fn tcb_configure(
        &self,
        cspace: CapPtr,
        vspace: CapPtr,
        utcb: CapPtr,
        trapframe: CapPtr,
        kstack: CapPtr,
    ) -> usize {
        self.invoke(tcbmethod::CONFIGURE, [cspace.0, vspace.0, utcb.0, trapframe.0, kstack.0, 0, 0])
    }

    pub fn tcb_set_priority(&self, priority: usize) -> usize {
        self.invoke(tcbmethod::SET_PRIORITY, [priority, 0, 0, 0, 0, 0, 0])
    }

    pub fn tcb_set_registers(&self, flags: usize, pc: usize, sp: usize) -> usize {
        self.invoke(tcbmethod::SET_REGISTERS, [flags, pc, sp, 0, 0, 0, 0])
    }

    pub fn tcb_set_fault_handler(&self, fault_ep: CapPtr) -> usize {
        self.invoke(tcbmethod::SET_FAULT_HANDLER, [fault_ep.0, 0, 0, 0, 0, 0, 0])
    }

    pub fn tcb_resume(&self) -> usize {
        self.invoke(tcbmethod::RESUME, [0, 0, 0, 0, 0, 0, 0])
    }

    pub fn tcb_suspend(&self) -> usize {
        self.invoke(tcbmethod::SUSPEND, [0, 0, 0, 0, 0, 0, 0])
    }

    // --- CNode Methods ---
    pub fn cnode_mint(&self, src: CapPtr, dest_slot: usize, badge: usize, rights: u8) -> usize {
        self.invoke(cnodemethod::MINT, [src.0, dest_slot, badge, rights as usize, 0, 0, 0])
    }

    pub fn cnode_copy(&self, src: CapPtr, dest_slot: usize, rights: u8) -> usize {
        self.invoke(cnodemethod::COPY, [src.0, dest_slot, rights as usize, 0, 0, 0, 0])
    }

    pub fn cnode_delete(&self, slot: usize) -> usize {
        self.invoke(cnodemethod::DELETE, [slot, 0, 0, 0, 0, 0, 0])
    }

    pub fn cnode_revoke(&self, slot: usize) -> usize {
        self.invoke(cnodemethod::REVOKE, [slot, 0, 0, 0, 0, 0, 0])
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
            [obj_type as usize, size_bits, n_objs, dest_cnode.0, dest_offset, 0, 0],
        )
    }

    pub fn untyped_retype_with_offset(
        &self,
        obj_type_raw: usize,
        size_bits: usize,
        n_objs: usize,
        dest_cnode: CapPtr,
        dest_offset: usize,
        offset: usize,
    ) -> usize {
        self.invoke(
            untypedmethod::RETYPE_WITH_OFFSET,
            [obj_type_raw, size_bits, n_objs, dest_cnode.0, dest_offset, offset, 0],
        )
    }

    // --- PageTable Methods ---
    pub fn pagetable_map(&self, frame: CapPtr, vaddr: usize, rights: usize) -> usize {
        self.invoke(pagetablemethod::MAP, [frame.0, vaddr, rights, 0, 0, 0, 0])
    }

    pub fn pagetable_map_table(&self, table: CapPtr, vaddr: usize, level: usize) -> usize {
        self.invoke(pagetablemethod::MAP_TABLE, [table.0, vaddr, level, 0, 0, 0, 0])
    }

    pub fn pagetable_unmap(&self, vaddr: usize) -> usize {
        self.invoke(pagetablemethod::UNMAP, [vaddr, 0, 0, 0, 0, 0, 0])
    }

    pub fn pagetable_map_trampoline(&self) ->usize{
        self.invoke(pagetablemethod::MAP_TRAMPOLINE, [0, 0, 0, 0, 0, 0, 0])
    }

    // --- IPC Methods ---
    pub fn ipc_send(&self, msg_info: MsgTag, args: Args) -> usize {
        let utcb = utcb::get();
        utcb.msg_tag = msg_info;
        self.invoke(ipcmethod::SEND, args)
    }

    pub fn ipc_recv(&self) -> usize {
        let (ret, badge) = sys_invoke_recv(self.0, ipcmethod::RECV, 0, 0, 0, 0, 0, 0, 0);
        if ret == 0 {
            badge
        } else {
            // TODO: Return Result
            0
        }
    }

    pub fn ipc_call(&self, msg_info: MsgTag, args: Args) -> usize {
        let utcb = utcb::get();
        utcb.msg_tag = msg_info;
        self.invoke(ipcmethod::CALL, args)
    }

    pub fn ipc_notify(&self, badge: usize) -> usize {
        self.invoke(ipcmethod::NOTIFY, [badge, 0, 0, 0, 0, 0, 0])
    }

    pub fn ipc_reply(&self, msg_info: MsgTag, args: Args) -> usize {
        let utcb = utcb::get();
        utcb.msg_tag = msg_info;
        self.invoke(replymethod::REPLY, args)
    }

    // --- Console Methods ---
    pub fn console_put_char(&self, c: char) -> usize {
        self.invoke(consolemethod::PUT_CHAR, [c as usize, 0, 0, 0, 0, 0, 0])
    }

    // --- IrqHandler Methods ---
    pub fn irq_handler_ack(&self) -> usize {
        self.invoke(irqmethod::ACK, [0, 0, 0, 0, 0, 0, 0])
    }

    pub fn irq_handler_set_notification(&self, notification: CapPtr) -> usize {
        self.invoke(irqmethod::SET_NOTIFICATION, [notification.0, 0, 0, 0, 0, 0, 0])
    }

    pub fn console_put_str(&self, s: &str) -> usize {
        let utcb = utcb::get();
        if let Some((offset, len)) = utcb.append_str(s) {
            self.invoke(consolemethod::PUT_STR, [offset, len, 0, 0, 0, 0, 0])
        } else {
            // Buffer overflow
            1
        }
    }
}

pub const CSPACE_SLOT: usize = 0;
pub const VSPACE_SLOT: usize = 1;
pub const TCB_SLOT: usize = 2;
pub const UTCB_SLOT: usize = 3;
pub const MEM_SLOT: usize = 4;
pub const MMIO_SLOT: usize = 5;
pub const IRQ_SLOT: usize = 6;
pub const FAULT_SLOT: usize = 7;
pub const CONSOLE_SLOT: usize = 8;
pub const MANIFEST_SLOT: usize = 9;

pub mod rights {
    pub const READ: u8 = 1 << 0;
    pub const WRITE: u8 = 1 << 1;
    pub const GRANT: u8 = 1 << 2;
    pub const SEND: u8 = 1 << 3;
    pub const RECV: u8 = 1 << 4;
    pub const CALL: u8 = 1 << 5;
    pub const ALL: u8 = 0xFF;
    pub const RW: u8 = READ | WRITE;
    pub const MASTER: u8 = ALL;
}
