use crate::error::Error;

/// NetworkService provides a factory for creating network sockets.
pub trait NetworkService {
    /// Create a new socket.
    /// Returns a capability handle to the socket.
    fn socket(&mut self, domain: i32, socket_type: i32, protocol: i32) -> Result<usize, Error>;
}

/// SocketService provides operations on an open network socket.
pub trait SocketService {
    /// Bind a socket to a local address.
    fn bind(&mut self, address: &[u8]) -> Result<(), Error>;

    /// Listen for incoming connections on a socket.
    fn listen(&mut self, backlog: i32) -> Result<(), Error>;

    /// Accept an incoming connection.
    /// Returns a capability handle to the new socket.
    fn accept(&mut self) -> Result<usize, Error>;

    /// Connect to a remote address.
    fn connect(&mut self, address: &[u8]) -> Result<(), Error>;

    /// Send data through a socket.
    fn send(&mut self, data: &[u8], flags: i32) -> Result<usize, Error>;

    /// Receive data from a socket.
    fn recv(&mut self, buffer: &mut [u8], flags: i32) -> Result<usize, Error>;

    /// Close a socket.
    fn close(&mut self) -> Result<(), Error>;

    /// Get local address of a socket.
    fn get_sockname(&self, address: &mut [u8]) -> Result<usize, Error>;

    /// Get remote address of a socket.
    fn get_peername(&self, address: &mut [u8]) -> Result<usize, Error>;

    /// Set socket options.
    fn setsockopt(&mut self, level: i32, optname: i32, optval: &[u8]) -> Result<(), Error>;

    /// Get socket options.
    fn getsockopt(&self, level: i32, optname: i32, optval: &mut [u8]) -> Result<usize, Error>;

    /// Setup io_uring for zero-copy data transfer.
    fn setup_iouring(
        &mut self,
        client_vaddr: usize,
        size: usize,
        frame: Option<crate::cap::Page>,
    ) -> Result<(), Error>;

    /// Notify server to process entries in io_uring.
    fn process_iouring(&mut self) -> Result<(), Error>;
}
