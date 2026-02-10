//! Timer Device Protocol

/// Get current time in nanoseconds or ticks (depends on implementation)
pub const GET_TIME: usize = 0x01;
/// Set an alarm. arg0: low, arg1: high
pub const SET_ALARM: usize = 0x02;
/// Stop the alarm
pub const STOP_ALARM: usize = 0x03;
