use crate::cap::{CSPACE_CAP, Endpoint, RECV_SLOT, Rights};
use crate::error::Error;
use crate::interface::AuthService;
use crate::ipc::{MsgFlags, MsgTag, UTCB};
use crate::protocol::{AUTH_PROTO, auth};
use crate::set_mrs;

#[derive(Debug, Clone, Copy)]
pub struct AuthClient {
    endpoint: Endpoint,
}

impl AuthClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }

    pub const fn endpoint(&self) -> Endpoint {
        self.endpoint
    }
}

impl AuthService for AuthClient {
    fn negotiate(&self, major: u16, minor: u16, flags: u32) -> Result<(u16, u16, u32), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, ((major as usize) << 16) | minor as usize, flags as usize);
        utcb.set_msg_tag(MsgTag::new(AUTH_PROTO, auth::NEGOTIATE, MsgFlags::NONE));
        self.endpoint.call(&mut utcb)?;

        let packed = utcb.get_mr(0);
        let negotiated_major = ((packed >> 16) & 0xffff) as u16;
        let negotiated_minor = (packed & 0xffff) as u16;
        let negotiated_flags = utcb.get_mr(1) as u32;
        Ok((negotiated_major, negotiated_minor, negotiated_flags))
    }

    fn get_identity(&self, subject: usize) -> Result<auth::IdentityInfo, Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, subject);
        utcb.set_msg_tag(MsgTag::new(AUTH_PROTO, auth::GET_IDENTITY, MsgFlags::NONE));
        self.endpoint.call(&mut utcb)?;

        if utcb.get_size() >= core::mem::size_of::<auth::IdentityInfo>() {
            unsafe { utcb.read_obj::<auth::IdentityInfo>() }
        } else {
            Ok(auth::IdentityInfo {
                uid: utcb.get_mr(0) as u32,
                gid: utcb.get_mr(1) as u32,
                euid: utcb.get_mr(2) as u32,
                egid: utcb.get_mr(3) as u32,
            })
        }
    }

    fn set_identity(&self, subject: usize, identity: auth::IdentityInfo) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, subject);
        unsafe {
            utcb.write_obj(&identity)?;
        }
        utcb.set_msg_tag(MsgTag::new(AUTH_PROTO, auth::SET_IDENTITY, MsgFlags::HAS_BUFFER));
        self.endpoint.call(&mut utcb)?;
        Ok(())
    }

    fn check_permission(
        &self,
        subject: usize,
        resource: &str,
        operation: &str,
    ) -> Result<auth::PermissionDecision, Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, subject);
        unsafe {
            utcb.write_postcard(&(resource, operation))?;
        }
        utcb.set_msg_tag(MsgTag::new(AUTH_PROTO, auth::CHECK_PERMISSION, MsgFlags::HAS_BUFFER));
        self.endpoint.call(&mut utcb)?;

        if utcb.get_size() >= core::mem::size_of::<auth::PermissionDecision>() {
            unsafe { utcb.read_obj::<auth::PermissionDecision>() }
        } else {
            Ok(auth::PermissionDecision {
                allowed: if utcb.get_mr(0) == 0 { 0 } else { 1 },
                reserved: [0; 3],
                ttl_ms: utcb.get_mr(1) as u32,
            })
        }
    }

    fn upsert_policy(
        &self,
        policy: auth::PolicyRule,
        resource: &str,
        operation: &str,
    ) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, policy.subject as usize, policy.effect as usize, policy.ttl_ms as usize);
        unsafe {
            utcb.write_postcard(&(resource, operation))?;
        }
        utcb.set_msg_tag(MsgTag::new(AUTH_PROTO, auth::UPSERT_POLICY, MsgFlags::HAS_BUFFER));
        self.endpoint.call(&mut utcb)?;
        Ok(())
    }

    fn delete_policy(&self, subject: usize, resource: &str, operation: &str) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, subject);
        unsafe {
            utcb.write_postcard(&(resource, operation))?;
        }
        utcb.set_msg_tag(MsgTag::new(AUTH_PROTO, auth::DELETE_POLICY, MsgFlags::HAS_BUFFER));
        self.endpoint.call(&mut utcb)?;
        Ok(())
    }

    fn set_policy_backend(&self, backend: Endpoint) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let transfer_slot = RECV_SLOT;
        let _ = CSPACE_CAP.delete(transfer_slot);
        CSPACE_CAP.copy_self(backend.cap(), transfer_slot, Rights::ALL)?;
        utcb.set_cap_transfer(transfer_slot);
        utcb.set_recv_window(transfer_slot);
        utcb.set_msg_tag(MsgTag::new(AUTH_PROTO, auth::SET_POLICY_BACKEND, MsgFlags::HAS_CAP));
        self.endpoint.call(&mut utcb)?;
        Ok(())
    }

    fn clear_policy_backend(&self) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(MsgTag::new(AUTH_PROTO, auth::CLEAR_POLICY_BACKEND, MsgFlags::NONE));
        self.endpoint.call(&mut utcb)?;
        Ok(())
    }

    fn get_policy_backend_status(&self) -> Result<auth::PolicyBackendStatus, Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(MsgTag::new(AUTH_PROTO, auth::GET_POLICY_BACKEND_STATUS, MsgFlags::NONE));
        self.endpoint.call(&mut utcb)?;

        if utcb.get_size() >= core::mem::size_of::<auth::PolicyBackendStatus>() {
            unsafe { utcb.read_obj::<auth::PolicyBackendStatus>() }
        } else {
            Ok(auth::PolicyBackendStatus {
                external_attached: if utcb.get_mr(0) == 0 { 0 } else { 1 },
                reserved: [0; 3],
                generation: utcb.get_mr(1) as u32,
            })
        }
    }

    fn get_ticket(&self, service: &str) -> Result<[u8; 256], Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, service.len());
        unsafe {
            utcb.write_str(service)?;
        }
        utcb.set_msg_tag(MsgTag::new(AUTH_PROTO, auth::GET_TICKET, MsgFlags::HAS_BUFFER));
        self.endpoint.call(&mut utcb)?;

        let mut ticket = [0u8; 256];
        let data = utcb.buffer();
        let copy_len = core::cmp::min(data.len(), ticket.len());
        ticket[..copy_len].copy_from_slice(&data[..copy_len]);
        Ok(ticket)
    }

    fn validate_ticket(&self, ticket: &[u8]) -> Result<bool, Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.write(ticket);
        utcb.set_msg_tag(MsgTag::new(AUTH_PROTO, auth::VALIDATE_TICKET, MsgFlags::HAS_BUFFER));
        self.endpoint.call(&mut utcb)?;
        Ok(utcb.get_mr(0) != 0)
    }

    fn logout(&self) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(MsgTag::new(AUTH_PROTO, auth::LOGOUT, MsgFlags::NONE));
        self.endpoint.call(&mut utcb)?;
        Ok(())
    }

    fn auth_rpc(&self, data: &[u8]) -> Result<[u8; 1024], Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, data.len());
        utcb.write(data);
        utcb.set_msg_tag(MsgTag::new(AUTH_PROTO, auth::AUTH_RPC, MsgFlags::HAS_BUFFER));
        self.endpoint.call(&mut utcb)?;

        let mut out = [0u8; 1024];
        let resp = utcb.buffer();
        let copy_len = core::cmp::min(resp.len(), out.len());
        out[..copy_len].copy_from_slice(&resp[..copy_len]);
        Ok(out)
    }

    fn proxy_call(
        &self,
        target_cap: usize,
        label: usize,
        proto: usize,
        payload: &[u8],
    ) -> Result<[u8; 1024], Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, target_cap, label, proto);
        utcb.write(payload);
        utcb.set_msg_tag(MsgTag::new(AUTH_PROTO, auth::PROXY_CALL, MsgFlags::HAS_BUFFER));
        self.endpoint.call(&mut utcb)?;

        let mut out = [0u8; 1024];
        let resp = utcb.buffer();
        let copy_len = core::cmp::min(resp.len(), out.len());
        out[..copy_len].copy_from_slice(&resp[..copy_len]);
        Ok(out)
    }
}
