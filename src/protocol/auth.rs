//! Authentication Protocol Definition
//!

/// Protocol version negotiation.
/// arg0: caller supported major/minor packed in 32-bit
/// arg1: caller feature flags
/// output: negotiated major/minor packed in 32-bit + feature flags
pub const NEGOTIATE: usize = 0x00;

/// Perform authentication RPC stage.
/// arg0: buffer size
/// input: data to auth-protocol
/// output: response from auth-protocol
pub const AUTH_RPC: usize = 0x01;

/// Get a session ticket for a service.
/// arg0: service name length
/// input: service name
/// output: ticket data
pub const GET_TICKET: usize = 0x02;

/// End session.
pub const LOGOUT: usize = 0x03;

/// Validate a ticket.
/// input: ticket blob
/// output: allow/deny
pub const VALIDATE_TICKET: usize = 0x05;

/// Proxy an IPC call.
/// arg0: target capability pointer
/// arg1: target label
/// arg2: target protocol
/// input: original message arguments
/// output: result from target
pub const PROXY_CALL: usize = 0x04;

// 身份管理命令
pub const GET_IDENTITY: usize = 0x10;
pub const SET_IDENTITY: usize = 0x11; // 对应 setuid/setgid

// 授权与策略管理命令
pub const CHECK_PERMISSION: usize = 0x20;
pub const UPSERT_POLICY: usize = 0x21;
pub const DELETE_POLICY: usize = 0x22;
pub const SET_POLICY_BACKEND: usize = 0x23;
pub const CLEAR_POLICY_BACKEND: usize = 0x24;
pub const GET_POLICY_BACKEND_STATUS: usize = 0x25;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct IdentityInfo {
    pub uid: u32,
    pub gid: u32,
    pub euid: u32,
    pub egid: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PermissionDecision {
    /// 1 = allow, 0 = deny
    pub allowed: u8,
    /// Reserved for future ABI extension.
    pub reserved: [u8; 3],
    /// Recommended cache TTL in milliseconds.
    pub ttl_ms: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PolicyRule {
    pub subject: u32,
    pub effect: u8,
    pub reserved: [u8; 3],
    pub ttl_ms: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PolicyBackendStatus {
    /// 1 = external backend attached, 0 = local policy engine
    pub external_attached: u8,
    pub reserved: [u8; 3],
    /// Monotonic generation for policy backend switch/update.
    pub generation: u32,
}
