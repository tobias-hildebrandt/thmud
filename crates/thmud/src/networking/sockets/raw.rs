use serde::{Deserialize, Serialize};
use std::{
    marker::PhantomData,
    net::{SocketAddr, UdpSocket},
};
use tracing::info;

/// Size of the buffer.
pub(crate) const BUFFER_SIZE: usize = 1500;

/// A UDP socket and a buffer.
///
/// Can only send one type of message and receive another type of message.
#[derive(Debug)]
pub(crate) struct NetSocketAndBuffer<Sends, Receives>
where
    Sends: Serialize,
    Receives: for<'a> Deserialize<'a>,
{
    pub(super) socket: UdpSocket,
    pub(super) buffer: [u8; BUFFER_SIZE],
    _phantom: PhantomData<(Sends, Receives)>,
}

impl<Sends, Receives> NetSocketAndBuffer<Sends, Receives>
where
    Sends: Serialize,
    Receives: for<'a> Deserialize<'a>,
{
    /// Create a new socket and buffer using the given socket.
    pub(crate) fn new(socket: UdpSocket) -> Self {
        socket
            .set_nonblocking(true)
            .expect("unable to set nonblocking socket, platform unsupported");

        let addr = socket.local_addr().expect("socket has no address");

        info!("socket bound to {addr:?}");

        Self {
            socket,
            buffer: [0; BUFFER_SIZE],
            _phantom: PhantomData,
        }
    }

    /// Receive a message from the socket.
    ///
    /// Does not block -- returns `Ok(None)` if no message is available.
    ///
    /// # Errors
    /// Returns error if the IO fails or the deserialization fails.
    // TODO: allow client to filter packets from non-server address instead of immediately deserializing
    pub(crate) fn recv(&mut self) -> Result<Option<(Receives, SocketAddr)>, IoOrSerializeError> {
        let (size, peer) = match self.socket.recv_from(&mut self.buffer) {
            Ok(size) => size,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let msg = postcard::from_bytes(&self.buffer[0..size])?;
        Ok(Some((msg, peer)))
    }

    /// Send a message to the given address.
    ///
    /// # Errors
    /// Returns error if the IO fails or the deserialization fails.
    pub(crate) fn send_to(
        &mut self,
        message: &Sends,
        target: SocketAddr,
    ) -> Result<(), IoOrSerializeError> {
        let len = postcard::to_slice(&message, &mut self.buffer)?.len();
        self.socket.send_to(&self.buffer[0..len], target)?;

        Ok(())
    }

    /// Return the local address of the socket.
    pub(crate) fn address(&self) -> Result<SocketAddr, std::io::Error> {
        self.socket.local_addr()
    }
}

/// Either an IO error or a de/serialization error
#[derive(Debug, thiserror::Error)]
pub(crate) enum IoOrSerializeError {
    #[error("Send error: {0:?}")]
    Io(#[from] std::io::Error),
    #[error("Serialize error: {0:?}")]
    Serialize(#[from] postcard::Error),
}
