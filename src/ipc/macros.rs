/// Macro to simplify IPC message dispatching based on Protocol and Label.
///
/// # Example
/// ```rust
/// ipc_dispatch! {
///     self, utcb,
///     (protocol::PROCESS_PROTO, protocol::process::FORK) => handle_fork,
///     (protocol::PROCESS_PROTO, protocol::process::EXIT) => handle_exit,
/// }
/// ```
#[macro_export]
macro_rules! ipc_dispatch {
    ($ctx:expr, $utcb:expr,
        $( ($proto:pat, $label:pat) => $handler:expr ),* $(,)?
    ) => {{
        let tag = $utcb.get_msg_tag();
        let p = tag.proto();
        let l = tag.label();

        match (p, l) {
            $(
                ($proto, $label) => $handler($ctx, $utcb),
            )*
            _ => Err($crate::error::Error::InvalidMethod),
        }
    }};
}
