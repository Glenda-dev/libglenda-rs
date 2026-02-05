//! Gopher Network Protocol Definition
//!
//! Protocol ID range: 0x600 - 0x6FF
//!
//! This protocol provides high-level socket operations for the Gopher network stack.

pub const PROTOCOL_ID: usize = 0x600;

// Network Management / Factory Operations (Invoked on Gopher service capability)
pub const SOCKET: usize = 1; // args: [domain, type, protocol] -> cap: socket

// Socket Operations (Invoked on open socket capability)
pub const BIND: usize = 10;
pub const LISTEN: usize = 11;
pub const ACCEPT: usize = 12; // -> cap: new_socket
pub const CONNECT: usize = 13;
pub const SEND: usize = 14;
pub const RECV: usize = 15;
pub const CLOSE: usize = 16;
pub const GET_SOCKNAME: usize = 17;
pub const GET_PEERNAME: usize = 18;
pub const SET_SOCKOPT: usize = 19;
pub const GET_SOCKOPT: usize = 20;

// Address Familes (Domain)
pub const AF_INET: i32 = 2;
pub const AF_INET6: i32 = 10;

// Socket Types
pub const SOCK_STREAM: i32 = 1;
pub const SOCK_DGRAM: i32 = 2;
pub const SOCK_RAW: i32 = 3;

// Protocol Constants
pub const IPPROTO_IP: i32 = 0;
pub const IPPROTO_ICMP: i32 = 1;
pub const IPPROTO_TCP: i32 = 6;
pub const IPPROTO_UDP: i32 = 17;
pub const IPPROTO_IPV6: i32 = 41;
pub const IPPROTO_RAW: i32 = 255;
