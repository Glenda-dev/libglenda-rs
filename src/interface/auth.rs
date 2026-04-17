use crate::error::Error;
use crate::cap::Endpoint;
use crate::ipc::Badge;
use crate::protocol::auth::{
    IdentityInfo, PermissionDecision, PolicyBackendStatus, PolicyRule,
};

/// AuthService mirrors AUTH protocol capabilities.
pub trait AuthService {
    fn negotiate(&self, major: u16, minor: u16, flags: u32) -> Result<(u16, u16, u32), Error>;

    fn auth_rpc(&self, data: &[u8]) -> Result<[u8; 1024], Error>;

    fn get_ticket(&self, service: &str) -> Result<[u8; 256], Error>;

    fn logout(&self) -> Result<(), Error>;

    fn validate_ticket(&self, ticket: &[u8]) -> Result<bool, Error>;

    fn get_identity(&self, subject: usize) -> Result<IdentityInfo, Error>;

    fn set_identity(&self, subject: usize, identity: IdentityInfo) -> Result<(), Error>;

    fn check_permission(
        &self,
        subject: usize,
        resource: &str,
        operation: &str,
    ) -> Result<PermissionDecision, Error>;

    fn upsert_policy(&self, policy: PolicyRule, resource: &str, operation: &str)
        -> Result<(), Error>;

    fn delete_policy(&self, subject: usize, resource: &str, operation: &str)
        -> Result<(), Error>;

    fn set_policy_backend(&self, backend: Endpoint) -> Result<(), Error>;

    fn clear_policy_backend(&self) -> Result<(), Error>;

    fn get_policy_backend_status(&self) -> Result<PolicyBackendStatus, Error>;

    fn proxy_call(
        &self,
        target_cap: usize,
        label: usize,
        proto: usize,
        payload: &[u8],
    ) -> Result<[u8; 1024], Error>;
}

/// ProxyService allows a service to act as a security gateway.
pub trait ProxyService {
    /// Proxy an IPC call to another capability.
    /// The implementation should check permissions before forwarding.
    fn proxy_call(
        &mut self,
        badge: Badge,
        target_cap: usize,
        label: usize,
        proto: usize,
    ) -> Result<(), Error>;
}
