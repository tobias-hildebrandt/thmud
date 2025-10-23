use std::{collections::HashSet, net::SocketAddr};

use bevy::{
    app::{FixedPostUpdate, FixedPreUpdate, Plugin},
    ecs::{
        query::With,
        resource::Resource,
        schedule::IntoScheduleConfigs,
        system::{Query, ResMut},
    },
    math::Vec2,
    transform::components::Transform,
};
use bevy_rapier2d::prelude::Velocity;

use crate::{
    networking::{
        ecs::{NetId, Networked},
        messages::{ClientBodyElement, NetHeader, ServerBodyElement},
    },
    simulation::{
        player::Player,
        thingy::{ThingyMarker, ThingyNet},
    },
};

use super::{
    ecs::NetPhysicsObjectBundle,
    messages::{ClientMessage, MessageBuffer, ServerMessage},
    sockets::{NetServerSocket, real::RealNetServerSocket},
};

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        let port = std::env::var("SERVER_PORT")
            .ok()
            .and_then(|s| s.parse().ok());
        let socket = NetServerSocket(RealNetServerSocket::new(port).unwrap());
        println!("server bound to address: {}", socket.address().unwrap());
        app.insert_resource(socket);

        let clients = Clients(Default::default());
        app.insert_resource(clients);

        app.add_systems(
            FixedPreUpdate,
            (
                server_recv_messages.before(server_handle_messages),
                server_handle_messages,
            ),
        );

        app.add_systems(FixedPostUpdate, (server_send_thingies,));

        app.insert_resource(ServerBuffer::new());
    }
}

#[derive(Debug, Resource, Default)]
pub struct Clients(HashSet<SocketAddr>);

type ServerBuffer = MessageBuffer<(ClientMessage<'static>, SocketAddr)>;

fn server_recv_messages(mut server: ResMut<NetServerSocket>, mut buffer: ResMut<ServerBuffer>) {
    while let Ok(Some((msg, peer))) = server.recv() {
        println!("client recv msg from peer {peer}");
        buffer.messages.push((msg, peer));
    }
}

// TODO: btreemap of NetId -> Entity?
fn server_handle_messages(
    mut buffer: ResMut<ServerBuffer>,
    mut clients: ResMut<Clients>,
    mut socket: ResMut<NetServerSocket>,
) {
    for (msg, peer) in buffer.messages.drain(..) {
        // TODO: process header
        for element in msg.body {
            match element {
                ClientBodyElement::Dummy(_cow) => {}
                ClientBodyElement::Register => {
                    if !clients.0.contains(&peer) {
                        println!("registering peer {peer}");
                        clients.0.insert(peer);

                        // TODO:
                        // socket.send_to_all(message, addresses)
                    }
                }
            }
        }
    }
}

fn server_send_thingies(
    clients: ResMut<Clients>,
    mut socket: ResMut<NetServerSocket>,
    query: Query<(&NetId, &Transform, &Velocity), With<ThingyMarker>>,
) {
    let bodies = query
        .iter()
        .filter_map(|(net_id, transform, vel)| {
            rand::random_bool(0.05).then_some(ThingyNet {
                net_id: *net_id,
                physics: NetPhysicsObjectBundle {
                    transform: Networked(*transform),
                    velocity: Networked(*vel),
                },
            })
        })
        .map(ServerBodyElement::Thingy);

    let message = ServerMessage {
        header: NetHeader {},
        body: bodies.collect(),
    };

    socket
        .send_to_all(message, clients.0.iter().copied())
        .unwrap();
}
