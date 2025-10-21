use std::{
    io,
    net::{SocketAddr, UdpSocket},
};

use bevy::ecs::component::Component;

#[derive(Debug)]
pub struct NetId(pub u32);

#[derive(Debug, Component)]
pub struct NetObject {
    id: NetId,
}

#[derive(Debug)]
pub struct NetPacket<'data> {
    header: NetHeader,
    body: Vec<NetPacketBody<'data>>,
}

#[derive(Debug)]
pub struct NetHeader {}

#[derive(Debug)]
pub enum NetPacketBody<'data> {
    Ping(&'data [u8]),
    Pong(&'data [u8]),
}

#[derive(Debug)]
pub struct NetSocketAndBuffer {
    socket: UdpSocket,
    buffer: [u8; Self::BUFFER_SIZE],
}

impl NetSocketAndBuffer {
    const BUFFER_SIZE: usize = 1500;

    fn new(socket: UdpSocket) -> Self {
        Self {
            socket,
            buffer: [0; Self::BUFFER_SIZE],
        }
    }

    fn recv(&mut self) -> Result<&[u8], io::Error> {
        let size = self.socket.recv(&mut self.buffer)?;
        Ok(&self.buffer[0..size])
    }

    fn send(&self, message: &[u8], target: SocketAddr) -> Result<(), io::Error> {
        self.socket.send_to(message, target)?;
        Ok(())
    }
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

        client.node.send(b"hello", client.server_addr).unwrap();

        let msg = server.node.recv().unwrap();
        println!("server sees message: {:?}", str::from_utf8(msg).unwrap());
    }
}
