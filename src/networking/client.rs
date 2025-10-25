use std::collections::{HashMap, HashSet};

use bevy::{
    app::{AppExit, FixedPreUpdate, Plugin},
    ecs::{
        component::{Component, Mutable},
        entity::Entity,
        event::EventReader,
        query::With,
        schedule::IntoScheduleConfigs,
        system::{Commands, Query, ResMut},
    },
    transform::components::Transform,
};
use bevy_rapier2d::prelude::{ExternalForce, Velocity};

use crate::{
    networking::messages::{ClientBodyElement, NetHeader, ServerBodyElement},
    simulation::{
        input::{PlayerInput, input_quit, read_local_inputs},
        player::{LocalPlayerMarker, Player, PlayerId, PlayerNet},
        thingy::{Thingy, ThingyNet},
    },
};

use super::{
    ecs::{NetId, Networked},
    messages::{ClientMessage, MessageBuffer, ServerMessage},
    sockets::{NetClientSocket, debug::DebugNetSocket, real::RealNetClientSocket},
};

pub struct GameClientPlugin;

impl Plugin for GameClientPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        let mut socket = if let Ok(addr) = std::env::var("SERVER_ADDR") {
            NetClientSocket::Real(RealNetClientSocket::new(addr.parse().unwrap()).unwrap())
        } else {
            // TODO: load/deser actions from filesystem?
            NetClientSocket::Debug(DebugNetSocket::new())
        };
        let mut m = ClientMessage::new(NetHeader {});
        m.try_push_back(ClientBodyElement::Register)
            .expect("unable to fit register in client message");
        socket.send(&m).unwrap();

        app.insert_resource(socket);

        app.add_systems(
            FixedPreUpdate,
            (
                client_recv_messages.before(client_handle_messages),
                client_handle_messages,
                (
                    client_apply_networked::<Velocity>,
                    client_apply_networked::<Transform>,
                    client_apply_networked::<PlayerInput>,
                    client_apply_networked::<ExternalForce>,
                )
                    .after(client_handle_messages),
                (client_send_inputs.after(read_local_inputs)),
                client_send_disconnect.after(input_quit),
            ),
        );

        app.insert_resource(ClientBuffer::new());
    }
}

type ClientBuffer = MessageBuffer<ServerMessage<'static>>;

fn client_recv_messages(mut socket: ResMut<NetClientSocket>, mut buffer: ResMut<ClientBuffer>) {
    while let Ok(Some((msg, peer))) = socket.recv() {
        // println!("client recv msg from peer {}", peer);

        if peer != socket.server_addr() {
            // drop it
            println!("dropping msg from non-server addr");
            continue;
        }
        buffer.messages.push(msg);
    }
}

#[derive(Debug)]
struct NetsById<T>(HashMap<NetId, T>);

impl<T> Default for NetsById<T> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

#[derive(Debug, Default)]
struct HandleMessageState {
    // entities spawned so far, so we don't double-spawn them
    spawned: HashSet<NetId>,
    my_player_id: Option<PlayerId>,

    thingy: NetsById<ThingyNet>,
    player: NetsById<PlayerNet>,
}

// TODO: look into exclusive system instead of using commands
fn client_handle_messages(
    mut buffer: ResMut<ClientBuffer>,
    query: Query<(Entity, &NetId)>,
    mut commands: Commands,
) {
    let mut state = HandleMessageState::default();

    // println!("client handling messages");
    for msg in buffer.messages.drain(..) {
        // println!("client handling message: {:?}", msg);
        // TODO: process header

        // process and sort bodies

        for body in msg.body_elements() {
            // println!("client handling body: {:?}", body);
            match body {
                ServerBodyElement::Dummy(_cow) => {}
                ServerBodyElement::YourPlayerId(player_id) => {
                    state.my_player_id = Some(player_id);
                }
                ServerBodyElement::Thingy(thingy_net) => {
                    state.thingy.0.insert(thingy_net.net_id, thingy_net);
                }
                ServerBodyElement::Player(player_net) => {
                    state.player.0.insert(player_net.net_id, player_net);
                }
            }
        }
    }

    // update for pre-existing entity
    for (entity, net_id) in query {
        if let Some(thingy) = state.thingy.0.remove(net_id) {
            commands.entity(entity).insert(thingy);
        }

        if let Some(player) = state.player.0.remove(net_id) {
            let mut commands = commands.entity(entity);
            if state.my_player_id.is_some_and(|i| i == player.player_id.0) {
                commands.insert(LocalPlayerMarker);
            }

            commands.insert(player);
        }
    }

    // spawn new thingies
    for (net_id, thingy) in state.thingy.0 {
        // do not spawn twice
        if !state.spawned.contains(&net_id) {
            state.spawned.insert(net_id);
            commands.spawn(Thingy::bundle(thingy));
        }
    }

    // spawn new players
    for (net_id, player) in state.player.0 {
        // do not spawn twice
        if !state.spawned.contains(&net_id) {
            state.spawned.insert(net_id);

            if state.my_player_id.is_some_and(|i| i == player.player_id.0) {
                commands.spawn((Player::bundle(player), LocalPlayerMarker));
            } else {
                commands.spawn(Player::bundle(player));
            }
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

// TODO: client send inputs
fn client_send_inputs(
    mut socket: ResMut<NetClientSocket>,
    query: Query<&PlayerInput, With<LocalPlayerMarker>>,
) {
    let Ok(inputs) = query.single() else {
        return;
    };

    let mut message = ClientMessage::new(NetHeader {});

    if let Err(e) = message.try_push_back(ClientBodyElement::Input(*inputs)) {
        println!("cannot push inputs into client message: {:?}", e);
        return;
    }

    socket.send(&message).unwrap();
}

fn client_send_disconnect(
    mut socket: ResMut<NetClientSocket>,
    mut event_reader: EventReader<AppExit>,
) {
    if event_reader.read().next().is_some() {
        let mut message = ClientMessage::new(NetHeader {});
        if let Err(e) = message.try_push_back(ClientBodyElement::Unregister) {
            println!("cannot push unregister into client message: {:?}", e);
            return;
        }

        socket.send(&message).unwrap();
    }
}
