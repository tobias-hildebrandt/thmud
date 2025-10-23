use std::collections::BTreeMap;

use bevy::{
    app::{FixedPreUpdate, Plugin},
    ecs::{
        component::{Component, Mutable},
        entity::Entity,
        schedule::IntoScheduleConfigs,
        system::{Commands, Query, ResMut},
    },
    transform::components::Transform,
};
use bevy_rapier2d::prelude::Velocity;

use crate::{
    networking::messages::{ClientBodyElement, NetHeader, ServerBodyElement},
    simulation::thingy::Thingy,
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
            // TODO: load/deser actions from filesystem?
            NetClientSocket::Debug(DebugNetSocket::new())
        };
        socket
            .send({
                let mut m = ClientMessage::new(NetHeader {});
                m.try_push(ClientBodyElement::Register)
                    .expect("unable to fit register in client message");
                m
            })
            .unwrap();

        app.insert_resource(socket);

        app.add_systems(
            FixedPreUpdate,
            (
                client_recv_messages.before(client_handle_messages),
                client_handle_messages,
                (
                    client_apply_networked::<Velocity>,
                    client_apply_networked::<Transform>,
                )
                    .after(client_handle_messages),
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

fn client_handle_messages(
    mut buffer: ResMut<ClientBuffer>,
    // TODO: btreemap of NetId -> Entity?
    query: Query<(Entity, &NetId)>,
    mut commands: Commands,
) {
    // println!("client handling messages");
    for msg in buffer.messages.drain(..) {
        // println!("client handling message: {:?}", msg);
        // TODO: process header

        // process and sort bodies

        // TODO: vec to support multiple updates per net_id
        let mut thingies = BTreeMap::new();

        for body in msg.body_elements() {
            // println!("client handling body: {:?}", body);
            match body {
                ServerBodyElement::Dummy(_cow) => {}
                ServerBodyElement::Thingy(thingy_net) => {
                    thingies.insert(thingy_net.net_id, thingy_net);
                }
            }
        }

        // update for pre-existing entity
        for (entity, our_net_id) in query {
            if let Some(t) = thingies.remove(our_net_id) {
                // println!("updating networked component");
                // override networked bundle
                commands.entity(entity).insert(t.physics);
            }
        }

        // spawn new entity
        for (_, new_thingy) in thingies {
            commands.spawn(Thingy::bundle(new_thingy));
        }
    }
}

fn client_apply_networked<T>(query: Query<(&mut T, &Networked<T>)>)
where
    T: Component<Mutability = Mutable> + Clone,
{
    for (mut real, networked) in query {
        *real = networked.0.clone();
    }
}
