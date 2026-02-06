//! Authentication Protocol Definition
//!

/// Perform authentication RPC stage.
/// arg0: buffer size
/// input: data to auth-protocol
/// output: response from auth-protocol
pub const AUTH_RPC: usize = 1;

/// Get a session ticket for a service.
/// arg0: service name length
/// input: service name
/// output: ticket data
pub const GET_TICKET: usize = 2;

/// End session.
pub const LOGOUT: usize = 3;

/// Proxy an IPC call.
/// arg0: target capability pointer
/// arg1: target label
/// arg2: target protocol
/// input: original message arguments
/// output: result from target
pub const PROXY_CALL: usize = 10;

// 身份管理命令
pub const GET_IDENTITY: usize = 11;
pub const SET_IDENTITY: usize = 12; // 对应 setuid/setgid
pub const SET_GROUPS: usize = 13;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct IdentityInfo {
    pub uid: u32,
    pub gid: u32,
    pub euid: u32,
    pub egid: u32,
    // Linux groups 通常是变长的，这里简化处理或通过单独调用获取
}
