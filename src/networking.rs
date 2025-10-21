use std::{
    borrow::Cow,
    io,
    net::{SocketAddr, UdpSocket},
};

use bevy::ecs::component::Component;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NetId(pub u32);

#[derive(Debug, Component, Serialize, Deserialize)]
pub struct NetObject {
    id: NetId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetMessage<'data> {
    header: NetHeader,
    body: Vec<NetPacketBody<'data>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetHeader {}

#[derive(Debug, Serialize, Deserialize)]
pub enum NetPacketBody<'data> {
    Ping(Cow<'data, [u8]>),
    Pong(Cow<'data, [u8]>),
}

#[derive(Debug)]
pub struct NetSocketAndBuffer {
    socket: UdpSocket,
    buffer: [u8; Self::BUFFER_SIZE],
}

impl NetSocketAndBuffer {
    const BUFFER_SIZE: usize = 1500;

    fn new(socket: UdpSocket) -> Self {
        socket
            .set_nonblocking(true)
            .expect("unable to set nonblocking socket, platform unsupported");

        Self {
            socket,
            buffer: [0; Self::BUFFER_SIZE],
        }
    }

    fn recv(&mut self) -> Result<Option<NetMessage<'static>>, SendOrSerializeError> {
        let size = match self.socket.recv(&mut self.buffer) {
            Ok(size) => size,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let msg = postcard::from_bytes(&self.buffer[0..size])?;
        Ok(Some(msg))
    }

    fn send(
        &mut self,
        message: &impl Serialize,
        target: SocketAddr,
    ) -> Result<(), SendOrSerializeError> {
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

#[derive(Debug, Component)]
pub struct NetServer {
    node: NetSocketAndBuffer,
    client_addresses: Vec<SocketAddr>,
}

impl NetServer {
    fn new(port: Option<u16>) -> Result<Self, io::Error> {
        let socket = UdpSocket::bind(("0.0.0.0", port.unwrap_or(0)))?;
        Ok(Self {
            node: NetSocketAndBuffer::new(socket),
            client_addresses: Default::default(),
        })
    }
}

#[derive(Debug, Component)]
pub struct NetClient {
    node: NetSocketAndBuffer,
    server_addr: SocketAddr,
}

impl NetClient {
    fn new(server_addr: SocketAddr) -> Result<Self, io::Error> {
        let socket = UdpSocket::bind(("0.0.0.0", 0))?;

        Ok(Self {
            node: NetSocketAndBuffer::new(socket),
            server_addr,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aaa() {
        let mut server = NetServer::new(Some(5555)).unwrap();
        let mut client = NetClient::new("127.0.0.1:5555".parse().unwrap()).unwrap();

        let client_message = NetMessage {
            header: NetHeader {},
            body: vec![NetPacketBody::Ping(Cow::Borrowed(b"hello"))],
        };

        client
            .node
            .send(&client_message, client.server_addr)
            .unwrap();

        let msg = server.node.recv().unwrap();
        println!("server sees message: {:?}", msg);
    }
}
