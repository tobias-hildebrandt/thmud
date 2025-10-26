use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use bevy::ecs::resource::Resource;
use debug::DebugNetSocket;
use real::{RealNetClientSocket, RealNetServerSocket, SendOrSerializeError};
use tracing::warn;

use super::messages::{client_messages::ClientMessage, server_messages::ServerMessage};

pub(crate) mod debug;
pub(crate) mod real;

#[derive(Debug, Resource)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum NetClientSocket {
    Real(RealNetClientSocket),
    Debug(DebugNetSocket<ServerMessage>),
}

const FAKE_DEBUG_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 9999);

impl NetClientSocket {
    pub(crate) fn recv(
        &mut self,
    ) -> Result<Option<(ServerMessage, SocketAddr)>, SendOrSerializeError> {
        match self {
            NetClientSocket::Real(real) => real.socket_and_buffer.recv(),
            NetClientSocket::Debug(debug) => Ok(debug.next_message().map(|m| (m, FAKE_DEBUG_ADDR))),
        }
    }

    pub(crate) fn send(&mut self, message: &ClientMessage) -> Result<(), SendOrSerializeError> {
        match self {
            NetClientSocket::Real(real) => {
                real.socket_and_buffer.send_to(message, real.server_addr)
            }
            NetClientSocket::Debug(_debug) => {
                // TODO:
                warn!("debug client sending not implemented");
                Ok(())
            }
        }
    }

    pub(crate) fn server_addr(&self) -> SocketAddr {
        match self {
            NetClientSocket::Real(real) => real.server_addr,
            NetClientSocket::Debug(_debug) => FAKE_DEBUG_ADDR,
        }
    }
}

#[derive(Debug, Resource)]
pub(crate) struct NetServerSocket(pub(crate) RealNetServerSocket);

impl NetServerSocket {
    pub(crate) fn recv(
        &mut self,
    ) -> Result<Option<(ClientMessage, SocketAddr)>, SendOrSerializeError> {
        self.0.socket_and_buffer.recv()
    }

    pub(crate) fn send_to(
        &mut self,
        message: &ServerMessage,
        address: SocketAddr,
    ) -> Result<(), SendOrSerializeError> {
        self.0.socket_and_buffer.send_to(message, address)
    }

    pub(crate) fn address(&self) -> Result<SocketAddr, std::io::Error> {
        self.0.socket_and_buffer.socket.local_addr()
    }
}
