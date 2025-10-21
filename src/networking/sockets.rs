use std::{
    io,
    net::{SocketAddr, UdpSocket},
};

use serde::Serialize;

use super::messages::NetMessage;

#[derive(Debug)]
pub struct NetSocketAndBuffer {
    pub(crate) socket: UdpSocket,
    pub(crate) buffer: [u8; Self::BUFFER_SIZE],
}

impl NetSocketAndBuffer {
    pub(crate) const BUFFER_SIZE: usize = 1500;

    pub(crate) fn new(socket: UdpSocket) -> Self {
        socket
            .set_nonblocking(true)
            .expect("unable to set nonblocking socket, platform unsupported");

        Self {
            socket,
            buffer: [0; Self::BUFFER_SIZE],
        }
    }

    pub(crate) fn recv(&mut self) -> Result<Option<NetMessage<'static>>, SendOrSerializeError> {
        let size = match self.socket.recv(&mut self.buffer) {
            Ok(size) => size,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let msg = postcard::from_bytes(&self.buffer[0..size])?;
        Ok(Some(msg))
    }

    pub(crate) fn send(
        &mut self,
        message: &impl Serialize,
        target: SocketAddr,
    ) -> Result<(), SendOrSerializeError> {
        // TODO: postcard::ser_flavors::Size
        let len = postcard::to_slice(message, &mut self.buffer)?.len();
        self.socket.send_to(&self.buffer[0..len], target)?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SendOrSerializeError {
    #[error("Send error: {0:?}")]
    Send(#[from] io::Error),
    #[error("Serialize error: {0:?}")]
    Serialize(#[from] postcard::Error),
}

#[derive(Debug)]
pub struct NetServer {
    pub(crate) node: NetSocketAndBuffer,
    pub(crate) client_addresses: Vec<SocketAddr>,
}

impl NetServer {
    pub(crate) fn new(port: Option<u16>) -> Result<Self, io::Error> {
        let socket = UdpSocket::bind(("0.0.0.0", port.unwrap_or(0)))?;
        Ok(Self {
            node: NetSocketAndBuffer::new(socket),
            client_addresses: Default::default(),
        })
    }
}

#[derive(Debug)]
pub struct NetClient {
    pub(crate) node: NetSocketAndBuffer,
    pub(crate) server_addr: SocketAddr,
}

impl NetClient {
    pub(crate) fn new(server_addr: SocketAddr) -> Result<Self, io::Error> {
        let socket = UdpSocket::bind(("0.0.0.0", 0))?;

        Ok(Self {
            node: NetSocketAndBuffer::new(socket),
            server_addr,
        })
    }
}
