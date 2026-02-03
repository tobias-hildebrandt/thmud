use super::super::messages::client_messages::ClientMessage;
use super::super::messages::server_messages::ServerMessage;
use crate::networking::sockets::raw::{IoOrSerializeError, NetSocketAndBuffer};

use bevy::ecs::resource::Resource;

use std::net::{SocketAddr, UdpSocket};

/// Network server.
#[derive(Debug, Resource)]
pub(crate) struct NetServer {
    pub(crate) socket: NetSocketAndBuffer<ServerMessage, ClientMessage>,
}

impl NetServer {
    /// Create a new socket by binding to the given port.
    pub(crate) fn new(port: Option<u16>) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(("0.0.0.0", port.unwrap_or(0)))?;
        Ok(Self {
            socket: NetSocketAndBuffer::new(socket),
        })
    }
}

/// Network client.
#[derive(Debug, Resource)]
pub(crate) struct NetClient {
    pub(crate) socket: NetSocketAndBuffer<ClientMessage, ServerMessage>,
    server_addr: SocketAddr,
}

impl NetClient {
    /// Create a new socket, targeting the given server address.
    pub(crate) fn new(server_addr: SocketAddr) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(("0.0.0.0", 0))?;

        Ok(Self {
            socket: NetSocketAndBuffer::new(socket),
            server_addr,
        })
    }

    /// Send a message to the server.
    pub(crate) fn send_to_server(
        &mut self,
        message: &ClientMessage,
    ) -> Result<(), IoOrSerializeError> {
        self.socket.send_to(message, self.server_addr)
    }

    /// Return the server address.
    pub(crate) fn server_addr(&self) -> SocketAddr {
        self.server_addr
    }
}
