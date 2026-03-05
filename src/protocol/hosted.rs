use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum HostedMessage {
    SysInvoke {
        cptr: usize,
        method: usize,
        utcb_ptr: usize,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum HostedReply {
    Success {
        ret: usize,
    },
    Error(isize),
}
