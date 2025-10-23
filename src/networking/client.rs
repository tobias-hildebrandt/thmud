use std::{collections::BTreeMap, time::Duration};

use bevy::{
    app::{FixedPreUpdate, Plugin},
    ecs::{
        entity::Entity,
        schedule::IntoScheduleConfigs,
        system::{Commands, Query, ResMut},
    },
};
use bevy_rapier2d::prelude::Velocity;

use crate::{
    networking::{
        messages::{ClientBodyElement, NetComponent, NetHeader, NetUpdate, ServerBodyElement},
        sockets::debug::DebugAction,
    },
    simulation::thingy::{CreateThingy, Thingy},
};

use super::{
    ecs::{NetId, Networked},
    messages::{ClientMessage, MessageBuffer, ServerMessage},
    sockets::{NetClientSocket, debug::DebugNetSocket, real::RealNetClientSocket},
};

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        let mut socket = if let Ok(addr) = std::env::var("SERVER_ADDR") {
            NetClientSocket::Real(RealNetClientSocket::new(addr.parse().unwrap()).unwrap())
        } else {
            // TODO: load/deser actions from filesystem
            let net = DebugNetSocket::new_with_actions(vec![
                DebugAction::Wait(Duration::from_secs(2)),
                DebugAction::Message(ServerMessage {
                    header: NetHeader {},
                    body: vec![ServerBodyElement::CreateThingy(CreateThingy {
                        net_id: NetId(123),
                        x: 100.,
                        y: 100.,
                        rigid_body_fixed: false,
                        is_circle: true,
                        radius: 20.,
                        density: 20.,
                    })],
                }),
                DebugAction::Wait(Duration::from_secs(2)),
                DebugAction::Message(ServerMessage {
                    header: NetHeader {},
                    body: vec![ServerBodyElement::CreateThingy(CreateThingy {
                        net_id: NetId(123),
                        x: -100.,
                        y: 100.,
                        rigid_body_fixed: false,
                        is_circle: true,
                        radius: 20.,
                        density: 20.,
                    })],
                }),
            ]);
            NetClientSocket::Debug(net)
        };
        socket
            .send(&ClientMessage {
                header: NetHeader {},
                body: vec![ClientBodyElement::Register],
            })
            .unwrap();

        app.insert_resource(socket);

        app.add_systems(
            FixedPreUpdate,
            (
                client_recv_messages.before(client_handle_messages),
                client_handle_messages,
                client_apply_networked_vel.after(client_handle_messages),
            ),
        );

        app.insert_resource(ClientBuffer::new());
    }
}

type ClientBuffer = MessageBuffer<ServerMessage<'static>>;

fn client_recv_messages(mut socket: ResMut<NetClientSocket>, mut buffer: ResMut<ClientBuffer>) {
    while let Ok(Some((msg, peer))) = socket.recv() {
        println!("client recv msg from peer {}", peer);

        if peer != socket.server_addr() {
            // drop it
            println!("dropping msg from non-server addr");
            continue;
        }
        buffer.messages.push(msg);
    }
}

// TODO: btreemap of NetId -> Entity?
fn client_handle_messages(
    mut buffer: ResMut<ClientBuffer>,
    query: Query<(Entity, &NetId)>,
    mut commands: Commands,
) {
    for msg in buffer.messages.drain(..) {
        // TODO: process header

        // process and sort bodies

        // TODO: vec to support multiple updates per net_id
        let mut net_updates = BTreeMap::new();

        let mut creates = Vec::new();

        for body in msg.body {
            println!("client handling body: {:?}", body);
            match body {
                ServerBodyElement::Dummy(_cow) => {}
                ServerBodyElement::CreateThingy(create) => {
                    // TODO: check for entity with same net id, delete?
                    creates.push(create);
                }
                ServerBodyElement::NetUpdate(NetUpdate { net_id, component }) => {
                    net_updates.insert(net_id, component);
                }
            }
        }

        // spawn and update at same time
        for create in creates {
            if let Some(update) = net_updates.remove(&create.net_id) {
                match update {
                    NetComponent::Transform(c) => {
                        commands.spawn((Thingy::create_bundle(create), c));
                    }
                    NetComponent::Velocity(c) => {
                        commands.spawn((Thingy::create_bundle(create), c));
                    }
                }
            } else {
                commands.spawn(Thingy::create_bundle(create));
            }
        }

        // update for pre-existing entity
        for (entity, our_net_id) in query {
            if let Some(t) = net_updates.remove(our_net_id) {
                println!("updating networked component");
                match t {
                    NetComponent::Transform(c) => commands.entity(entity).insert(Networked::new(c)),
                    NetComponent::Velocity(c) => commands.entity(entity).insert(Networked::new(c)),
                };
            }
        }
    }
}

// TODO: macroize/dyn-ize
fn client_apply_networked_vel(mut query: Query<(&mut Velocity, &Networked<Velocity>)>) {
    for (mut real, networked) in query {
        if let Some(net) = networked.component {
            *real = net;
        }
    }
}
