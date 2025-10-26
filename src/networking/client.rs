use std::collections::{HashMap, HashSet};

use bevy::{
    app::{AppExit, FixedPreUpdate, Plugin},
    ecs::{
        component::{Component, Mutable},
        entity::Entity,
        event::EventReader,
        query::With,
        resource::Resource,
        schedule::IntoScheduleConfigs,
        system::{Commands, Query, Res, ResMut},
    },
    transform::components::Transform,
};
use bevy_rapier2d::prelude::{ExternalForce, Velocity};
use tracing::{debug, warn};

use crate::{
    networking::ecs::NetObj,
    simulation::{
        input::{PlayerInput, input_quit, read_local_inputs},
        player::{LocalPlayerMarker, Player, PlayerId, PlayerNet},
        thingy::{Thingy, ThingyNet},
    },
};

use super::{
    ecs::{LastNetUpdate, NetId, Networked},
    messages::{
        client_messages::{ClientDataMessageBuilder, ClientMessage, NetObjAck},
        common::{MessageBuffer, NetHeader},
        server_messages::{ServerBodyElement, ServerMessage},
    },
    netrate::NetTick,
    sockets::{NetClientSocket, debug::DebugNetSocket, real::RealNetClientSocket},
    tick::GameTick,
};

pub(crate) struct GameClientPlugin;

impl Plugin for GameClientPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        let mut socket = if let Ok(addr) = std::env::var("SERVER_ADDR") {
            NetClientSocket::Real(RealNetClientSocket::new(addr.parse().unwrap()).unwrap())
        } else {
            // TODO: load/deser actions from filesystem?
            NetClientSocket::Debug(DebugNetSocket::new())
        };
        let m = ClientMessage::register(NetHeader {});
        socket.send(&m).unwrap();

        app.insert_resource(socket);
        app.insert_resource(ClientBuffer::new());
        app.insert_resource(NetObjectUpdates::default());

        app.add_systems(
            FixedPreUpdate,
            (
                client_recv_messages.before(client_handle_messages),
                clear_net_object_updates.before(client_handle_messages),
                client_handle_messages,
                (
                    client_apply_networked::<Velocity>,
                    client_apply_networked::<Transform>,
                    client_apply_networked::<PlayerInput>,
                    client_apply_networked::<ExternalForce>,
                )
                    .after(client_handle_messages)
                    .after(clear_net_object_updates)
                    .after(
                        client_send_data, /* DON'T OVERWRITE UNTIL AFTER WE ACK */
                    ),
                (client_send_data.after(read_local_inputs)),
                client_send_disconnect.after(input_quit),
            ),
        );
    }
}

type ClientBuffer = MessageBuffer<ServerMessage>;

fn client_recv_messages(mut socket: ResMut<NetClientSocket>, mut buffer: ResMut<ClientBuffer>) {
    while let Ok(Some((msg, peer))) = socket.recv() {
        // debug!("client recv msg from peer {}", peer);

        if peer != socket.server_addr() {
            // drop it
            warn!("dropping msg from non-server addr");
            continue;
        }
        buffer.messages.push(msg);
    }
}

/// Stores IDs of all net objects that were changed this frame.
#[derive(Debug, Resource, Default)]
struct NetObjectUpdates(HashMap<NetId, GameTick>);

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

    thingy: NetsById<(ThingyNet, GameTick)>,
    player: NetsById<(PlayerNet, GameTick)>,
}

// TODO: look into exclusive system instead of using commands
fn client_handle_messages(
    mut buffer: ResMut<ClientBuffer>,
    query: Query<(Entity, &NetId, &mut LastNetUpdate)>,
    mut commands: Commands,
    mut updates: ResMut<NetObjectUpdates>,
    mut net_tick: EventReader<NetTick>,
) {
    if net_tick.read().count() == 0 {
        return;
    }

    let mut state = HandleMessageState::default();

    debug!("client handling messages");
    for msg in buffer.messages.drain(..) {
        // TODO: process header

        let tick = msg.tick;

        // process bodies
        for body in msg.body_elements() {
            debug!("client handling body: {:?}", body);
            match body {
                ServerBodyElement::YourPlayerId(player_id) => {
                    state.my_player_id = Some(player_id);
                }
                ServerBodyElement::NetObj(net_obj) => match net_obj {
                    NetObj::Player(player_net) => {
                        state.player.0.insert(player_net.net_id, (player_net, tick));
                    }
                    NetObj::Thingy(thingy_net) => {
                        state.thingy.0.insert(thingy_net.net_id, (thingy_net, tick));
                    }
                },
            }
        }
    }

    // update for pre-existing entity
    for (entity, net_id, mut last_update) in query {
        let mut this_update = None;
        if let Some((thingy, tick)) = state.thingy.0.remove(net_id)
            && tick > last_update.0
        {
            commands.entity(entity).insert(thingy);
            last_update.0 = tick;
            this_update = Some(tick);
        }

        if let Some((player, tick)) = state.player.0.remove(net_id)
            && tick > last_update.0
        {
            let mut commands = commands.entity(entity);
            if state.my_player_id.is_some_and(|i| i == player.player_id.0) {
                commands.insert(LocalPlayerMarker);
            }
            commands.insert(player);

            last_update.0 = tick;
            this_update = Some(tick);
        }

        // make sure the later systems actually apply the change
        if let Some(tick) = this_update {
            updates.0.insert(*net_id, tick);
        }
    }

    // spawn new thingies
    for (net_id, (thingy, tick)) in state.thingy.0 {
        // do not spawn twice
        if !state.spawned.contains(&net_id) {
            state.spawned.insert(net_id);

            commands.spawn(Thingy::client_bundle(thingy, tick));
        }
    }

    // spawn new players
    for (net_id, (player, tick)) in state.player.0 {
        // do not spawn twice
        if !state.spawned.contains(&net_id) {
            state.spawned.insert(net_id);

            let player_is_me = state.my_player_id.is_some_and(|i| i == player.player_id.0);

            let bundle = Player::client_bundle(player, tick);

            if player_is_me {
                commands.spawn((bundle, LocalPlayerMarker));
            } else {
                commands.spawn(bundle);
            }
        }
    }
}

fn clear_net_object_updates(mut updates: ResMut<NetObjectUpdates>) {
    updates.0.clear();
}

/// Applies the value inside a [`Networked`] component to the non-networked equivalent component,
/// if the network object was updated this frame.
// TODO: interpolation/extrapolation
fn client_apply_networked<T>(
    updates: Res<NetObjectUpdates>,
    query: Query<(&NetId, &mut T, &Networked<T>, Option<&LocalPlayerMarker>)>,
) where
    T: Component<Mutability = Mutable> + Clone,
{
    for (net_id, mut real, networked, local) in query {
        if updates.0.contains_key(net_id) {
            if local.is_some() {
                debug!(
                    "applying networked local player's {:?}",
                    std::any::type_name_of_val(&networked.0)
                        .split("::")
                        .last()
                        .unwrap()
                );
            }
            *real = networked.0.clone();
        }
    }
}

fn client_send_data(
    mut socket: ResMut<NetClientSocket>,
    query: Query<&PlayerInput, With<LocalPlayerMarker>>,
    updates: Res<NetObjectUpdates>,
    mut net_tick: EventReader<NetTick>,
) {
    if net_tick.read().count() == 0 {
        return;
    }

    let Ok(inputs) = query.single() else {
        return;
    };

    let mut message = ClientDataMessageBuilder::new(NetHeader {}, *inputs);

    // TODO: push acks
    for (net_id, tick) in updates.0.iter() {
        if message
            .try_add_ack(NetObjAck {
                net_id: *net_id,
                tick: *tick,
            })
            .is_err()
        {
            break;
        }
    }

    let message = message.build();
    socket.send(&message).unwrap();
    debug!("client sent message: {message:?}");
}

fn client_send_disconnect(
    mut socket: ResMut<NetClientSocket>,
    mut event_reader: EventReader<AppExit>,
) {
    if event_reader.read().next().is_some() {
        let message = ClientMessage::unregister(NetHeader {});

        socket.send(&message).unwrap();
    }
}
