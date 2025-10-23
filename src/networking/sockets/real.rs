use std::{
    io,
    net::{SocketAddr, UdpSocket},
};

use bevy::ecs::resource::Resource;
use serde::{Deserialize, Serialize};

// TODO: restrict send type and return types?

#[derive(Debug)]
pub(super) struct NetSocketAndBuffer {
    pub(super) socket: UdpSocket,
    pub(super) buffer: [u8; Self::BUFFER_SIZE],
}

impl NetSocketAndBuffer {
    const BUFFER_SIZE: usize = 1500;

    pub(super) fn new(socket: UdpSocket) -> Self {
        socket
            .set_nonblocking(true)
            .expect("unable to set nonblocking socket, platform unsupported");

        Self {
            socket,
            buffer: [0; Self::BUFFER_SIZE],
        }
    }

    // TODO: allow client to filter packets from non-server address instead of immediately deserializing
    pub(super) fn recv<'a, T: Deserialize<'a>>(
        &'a mut self,
    ) -> Result<Option<(T, SocketAddr)>, SendOrSerializeError> {
        let (size, peer) = match self.socket.recv_from(&mut self.buffer) {
            Ok(size) => size,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let msg = postcard::from_bytes(&self.buffer[0..size])?;
        Ok(Some((msg, peer)))
    }

    pub(super) fn send_to(
        &mut self,
        message: impl Serialize,
        target: SocketAddr,
    ) -> Result<(), SendOrSerializeError> {
        let len = postcard::to_slice(&message, &mut self.buffer)?.len();
        self.socket.send_to(&self.buffer[0..len], target)?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum SendOrSerializeError {
    #[error("Send error: {0:?}")]
    Send(#[from] io::Error),
    #[error("Serialize error: {0:?}")]
    Serialize(#[from] postcard::Error),
}

#[derive(Debug, Resource)]
pub(crate) struct RealNetServerSocket {
    pub(super) socket_and_buffer: NetSocketAndBuffer,
}

impl RealNetServerSocket {
    pub(crate) fn new(port: Option<u16>) -> Result<Self, io::Error> {
        let socket = UdpSocket::bind(("0.0.0.0", port.unwrap_or(0)))?;
        Ok(Self {
            socket_and_buffer: NetSocketAndBuffer::new(socket),
        })
    }
}

#[derive(Debug)]
pub(crate) struct RealNetClientSocket {
    pub(super) socket_and_buffer: NetSocketAndBuffer,
    pub(super) server_addr: SocketAddr,
}

impl RealNetClientSocket {
    pub(crate) fn new(server_addr: SocketAddr) -> Result<Self, io::Error> {
        let socket = UdpSocket::bind(("0.0.0.0", 0))?;

        Ok(Self {
            socket_and_buffer: NetSocketAndBuffer::new(socket),
            server_addr,
        })
    }
}
