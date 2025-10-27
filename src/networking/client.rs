use std::collections::{HashMap, HashSet};

use bevy::{
    app::{AppExit, FixedPreUpdate, Plugin},
    ecs::{
        component::{Component, Mutable},
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        query::With,
        schedule::IntoScheduleConfigs,
        system::{Commands, Query, ResMut},
    },
    transform::components::Transform,
};
use bevy_rapier2d::prelude::{ExternalForce, Velocity};
use tracing::{debug, warn};

use crate::{
    networking::{ecs::NetObj, sockets::NetClient},
    simulation::{
        input::{PlayerInput, input_quit, read_local_inputs},
        player::{LocalPlayerMarker, Player, PlayerId},
        thingy::Thingy,
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
    tick::GameTick,
};

/// Plugin for the network client.
pub(crate) struct GameClientPlugin;

impl Plugin for GameClientPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        let addr = std::env::var("SERVER_ADDR")
            .expect("no SERVER_ADDR env var")
            .parse()
            .expect("SERVER_ADDR not valid SocketAddr");
        let mut net_client = NetClient::new(addr).unwrap();

        let m = ClientMessage::register(NetHeader {});
        net_client.send_to_server(&m).unwrap();

        app.insert_resource(net_client);
        app.insert_resource(ClientBuffer::new());

        app.add_event::<NetObjectUpdateEvent>();

        app.add_systems(
            FixedPreUpdate,
            (
                client_recv_messages.before(client_handle_messages),
                client_handle_messages,
                (client_send_data.after(read_local_inputs)),
                (
                    client_apply_networked::<Velocity>,
                    client_apply_networked::<Transform>,
                    client_apply_networked::<PlayerInput>,
                    client_apply_networked::<ExternalForce>,
                )
                    .after(client_handle_messages)
                    // DON'T OVERWRITE UNTIL AFTER WE ACK
                    .after(client_send_data),
                client_send_disconnect.after(input_quit),
            ),
        );
    }
}

/// Buffer for client messages.
type ClientBuffer = MessageBuffer<ServerMessage>;

/// Read messages off of the socket and push them into the buffer.
fn client_recv_messages(mut net_client: ResMut<NetClient>, mut buffer: ResMut<ClientBuffer>) {
    while let Ok(Some((msg, peer))) = net_client.socket.recv() {
        // debug!("client recv msg from peer {}", peer);

        if peer != net_client.server_addr() {
            // drop it
            warn!("dropping msg from non-server addr");
            continue;
        }
        buffer.messages.push(msg);
    }
}

/// An event that is written whenever a [`NetObj`] is updated due to a server message.
#[derive(Debug, Event)]
struct NetObjectUpdateEvent {
    entity: Entity,
    net_id: NetId,
    tick: GameTick,
}

/// State necessary for handling messages during a tick.
///
/// (Only used locally in [`client_handle_messages`]).
#[derive(Debug, Default)]
struct HandleMessageState {
    /// Entities that we have spawned already (so we don't double-spawn them).
    spawned: HashSet<NetId>,
    /// This client's player ID, which is sent by the server in every message.
    my_player_id: Option<PlayerId>,
    /// Cache for net objects that we need to spawn.
    net_objs: HashMap<NetId, (NetObj, GameTick)>,
}

/// Process client messages. Updates and spawns net objects based on server messages.
///
/// Only runs on [`NetTick`]s.
// TODO: look into exclusive system instead of using commands
fn client_handle_messages(
    mut buffer: ResMut<ClientBuffer>,
    // TODO: use QueryData struct for this?
    query: Query<(Entity, &NetId, &mut LastNetUpdate)>,
    mut commands: Commands,
    mut update_writer: EventWriter<NetObjectUpdateEvent>,
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
                ServerBodyElement::NetObj(net_obj) => {
                    state.net_objs.insert(net_obj.net_id(), (net_obj, tick));
                }
            }
        }
    }

    // update for pre-existing entity
    for (entity, net_id, mut last_update) in query {
        if let Some((net_obj, tick)) = state.net_objs.remove(net_id)
            && tick > last_update.0
        {
            let mut commands = commands.entity(entity);

            // check if local player
            // TODO: technically unnecessary if player id never changes?
            if let Some(net_player_id) = net_obj.player_id()
                && let Some(my_player_id) = state.my_player_id
                && my_player_id == net_player_id
            {
                commands.insert(LocalPlayerMarker);
            }

            // overwrite the net components
            match net_obj {
                NetObj::Player(player_net) => commands.insert(player_net),
                NetObj::Thingy(thingy_net) => commands.insert(thingy_net),
            };

            // make sure to update
            last_update.0 = tick;

            // make sure the later systems actually apply the change
            update_writer.write(NetObjectUpdateEvent {
                entity,
                net_id: *net_id,
                tick,
            });
        }
    }

    // spawn new net objs
    for (net_id, (net_obj, tick)) in state.net_objs {
        // do not spawn twice
        if !state.spawned.contains(&net_id) {
            state.spawned.insert(net_id);

            let mut commands = commands.spawn_empty();
            let entity = commands.id();

            // check if local player
            if let Some(net_player_id) = net_obj.player_id()
                && let Some(my_player_id) = state.my_player_id
                && my_player_id == net_player_id
            {
                commands.insert(LocalPlayerMarker);
            }

            // spawn entire bundle
            match net_obj {
                NetObj::Player(player_net) => {
                    commands.insert(Player::client_bundle(player_net, tick))
                }
                NetObj::Thingy(thingy_net) => {
                    commands.insert(Thingy::client_bundle(thingy_net, tick))
                }
            };

            // make sure the later systems actually apply the change
            // (not technically needed to update the components, but needed for sending ACKs)
            update_writer.write(NetObjectUpdateEvent {
                entity,
                net_id,
                tick,
            });
        }
    }
}

/// Apply the value inside a [`Networked`] component to the non-networked equivalent component,
/// if the network object was updated this frame.
// TODO: interpolation/extrapolation
fn client_apply_networked<T>(
    mut updates: EventReader<NetObjectUpdateEvent>,
    mut query: Query<(&mut T, &Networked<T>, Option<&LocalPlayerMarker>)>,
) where
    T: Component<Mutability = Mutable> + Clone,
{
    for update_event in updates.read() {
        let Ok((mut real, networked, local)) = query.get_mut(update_event.entity) else {
            continue;
        };

        if local.is_some() {
            debug!(
                "applying networked local player's {:?}",
                std::any::type_name::<T>().split("::").last().unwrap()
            );
        }

        *real = networked.0.clone();
    }
}

/// Send client data to the server.
///
/// Only runs on [`NetTick`]s.
fn client_send_data(
    mut net_client: ResMut<NetClient>,
    query: Query<&PlayerInput, With<LocalPlayerMarker>>,
    mut updates: EventReader<NetObjectUpdateEvent>,
    mut net_tick: EventReader<NetTick>,
) {
    if net_tick.read().count() == 0 {
        return;
    }

    let Ok(inputs) = query.single() else {
        return;
    };

    let mut message = ClientDataMessageBuilder::new(NetHeader {}, *inputs);

    // push acks
    for update_event in updates.read() {
        if message
            .try_add_ack(NetObjAck {
                net_id: update_event.net_id,
                tick: update_event.tick,
            })
            .is_err()
        {
            break;
        }
    }

    let message = message.build();
    net_client.send_to_server(&message).unwrap();
    debug!("client sent message: {message:?}");
}

/// Send an unregister message if the bevy app is about to exit.
fn client_send_disconnect(
    mut net_client: ResMut<NetClient>,
    mut event_reader: EventReader<AppExit>,
) {
    if event_reader.read().next().is_some() {
        let message = ClientMessage::unregister(NetHeader {});

        net_client.send_to_server(&message).unwrap();
    }
}
