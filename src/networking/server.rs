use std::{collections::HashSet, net::SocketAddr};

use bevy::{
    app::{FixedPostUpdate, Plugin},
    ecs::{resource::Resource, schedule::IntoScheduleConfigs, system::ResMut},
    math::Vec2,
};
use bevy_rapier2d::prelude::Velocity;

use crate::{
    networking::{
        ecs::NetId,
        messages::{ClientBodyElement, NetComponent, NetHeader, NetUpdate, ServerBodyElement},
    },
    simulation::{player::Player, thingy::CreateThingy},
};

use super::{
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
            FixedPostUpdate,
            (
                server_recv_messages.before(server_handle_messages),
                server_handle_messages,
                server_periodic_send.after(server_handle_messages),
            ),
        );

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

fn server_periodic_send(clients: ResMut<Clients>, mut socket: ResMut<NetServerSocket>) {
    if clients.0.is_empty() {
        return;
    }
    if rand::random_bool(0.95) {
        return;
    }

    const RAND_COORD: f32 = 1000.;

    let creates = (0..rand::random_range(0usize..=3))
        .map(|_| CreateThingy {
            net_id: NetId(rand::random()),
            x: rand::random_range(-RAND_COORD..=RAND_COORD),
            y: rand::random_range(-RAND_COORD..=RAND_COORD),
            rigid_body_fixed: rand::random(),
            is_circle: rand::random(),
            radius: rand::random_range(50.0..=100.0),
            density: rand::random_range(0.0..(Player::DENSITY * 2.)),
        })
        .collect::<Vec<_>>();

    const RAND_VEL: f32 = 1000.;

    let velocities = creates
        .iter()
        .map(|c| {
            let net_id = c.net_id;
            NetUpdate {
                net_id,
                component: NetComponent::Velocity(Velocity {
                    linvel: Vec2 {
                        x: rand::random_range(-RAND_VEL..RAND_VEL),
                        y: rand::random_range(-RAND_VEL..RAND_VEL),
                    },
                    ..Default::default()
                }),
            }
        })
        .collect::<Vec<_>>();

    let all = creates
        .into_iter()
        .map(ServerBodyElement::CreateThingy)
        .chain(velocities.into_iter().map(ServerBodyElement::NetUpdate))
        .collect();

    let message = ServerMessage {
        header: NetHeader {},
        body: all,
    };

    socket
        .send_to_all(message, clients.0.iter().copied())
        .unwrap();
}
