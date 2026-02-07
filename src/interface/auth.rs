use crate::error::Error;
use crate::ipc::Badge;

/// AuthService provides authentication services.
pub trait AuthService {
    /// Perform an authentication RPC (Plan 9 style).
    /// Used for multi-step challenge-response protocols.
    fn auth_rpc(&mut self, data: &[u8]) -> Result<[u8; 1024], Error>;

    /// Check if a user (identified by badge) has specific permissions.
    fn check_permission(&self, badge: Badge, resource: &str, operation: &str) -> bool;

    /// Get a session token for a specific service.
    fn get_ticket(&mut self, service: &str) -> Result<[u8; 256], Error>;
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
