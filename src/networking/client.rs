use std::{collections::BTreeMap, time::Duration};

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
    networking::{
        messages::{ClientBodyElement, NetHeader, ServerBodyElement},
        sockets::debug::DebugAction,
    },
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
            // TODO: load/deser actions from filesystem
            // let net = DebugNetSocket::new_with_actions(vec![
            //     DebugAction::Wait(Duration::from_secs(2)),
            //     DebugAction::Message(ServerMessage {
            //         header: NetHeader {},
            //         body: vec![ServerBodyElement::CreateThingy(CreateThingy {
            //             net_id: NetId(123),
            //             x: 100.,
            //             y: 100.,
            //             rigid_body_fixed: false,
            //             is_circle: true,
            //             radius: 20.,
            //             density: 20.,
            //         })],
            //     }),
            //     DebugAction::Wait(Duration::from_secs(2)),
            //     DebugAction::Message(ServerMessage {
            //         header: NetHeader {},
            //         body: vec![ServerBodyElement::CreateThingy(CreateThingy {
            //             net_id: NetId(123),
            //             x: -100.,
            //             y: 100.,
            //             rigid_body_fixed: false,
            //             is_circle: true,
            //             radius: 20.,
            //             density: 20.,
            //         })],
            //     }),
            // ]);
            NetClientSocket::Debug(DebugNetSocket::new())
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
    println!("client handling messages");
    for msg in buffer.messages.drain(..) {
        println!("client handling message: {:?}", msg);
        // TODO: process header

        // process and sort bodies

        // TODO: vec to support multiple updates per net_id
        let mut thingies = BTreeMap::new();

        for body in msg.body {
            println!("client handling body: {:?}", body);
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
                println!("updating networked component");
                // override networked bundle
                commands.entity(entity).insert(t.physics);
            }
        }

        // spawn and update at same time
        for (_, new_thingy) in thingies {
            commands.spawn(Thingy::networked_bundle(new_thingy));
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

// // TODO: macroize/dyn-ize
// fn client_apply_networked_vel(query: Query<(&mut Velocity, &Networked<Velocity>)>) {
//     for (mut real, networked) in query {
//         *real = networked.0;
//     }
// }

// fn client_apply_networked_transform(query: Query<(&mut Velocity, &Networked<Velocity>)>) {
//     for (mut real, networked) in query {
//         *real = networked.0;
//     }
// }
