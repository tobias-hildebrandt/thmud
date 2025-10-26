use std::{
    collections::{HashMap, HashSet, hash_map::Entry},
    net::SocketAddr,
};

use bevy::{
    app::{FixedPostUpdate, FixedPreUpdate, Plugin},
    ecs::{
        entity::Entity,
        event::EventReader,
        query::With,
        resource::Resource,
        schedule::IntoScheduleConfigs,
        system::{Commands, Query, Res, ResMut},
    },
};
use tracing::{debug, error, info};

use super::{
    ecs::NetId,
    messages::{
        client_messages::ClientMessage,
        common::{MessageBuffer, NetHeader},
        server_messages::{ServerBodyElement, ServerMessage},
    },
    netrate::NetTick,
    sockets::{NetServerSocket, real::RealNetServerSocket},
    tick::GameTick,
};
use crate::{
    networking::{messages::client_messages::ClientMessageBody, tick::increment_tick},
    simulation::{
        input::PlayerInput,
        player::{Player, PlayerMarker, PlayerNet, PlayerNetQuery},
        thingy::{ThingyMarker, ThingyNet, ThingyNetQuery},
    },
};

pub(crate) struct GameServerPlugin;

impl Plugin for GameServerPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        let port = std::env::var("SERVER_PORT")
            .ok()
            .and_then(|s| s.parse().ok());
        let socket = NetServerSocket(RealNetServerSocket::new(port).unwrap());
        info!("server bound to address: {}", socket.address().unwrap());
        app.insert_resource(socket);

        let clients = Clients(Default::default());
        app.insert_resource(clients);

        app.add_systems(
            FixedPreUpdate,
            (
                server_recv_messages.before(server_handle_messages),
                server_handle_messages,
            )
                .after(increment_tick),
        );

        app.add_systems(FixedPostUpdate, (server_send,));

        app.insert_resource(ServerBuffer::new());
    }
}

#[derive(Debug, Resource, Default)]
pub(crate) struct Clients(HashMap<SocketAddr, ClientInfo>);

#[derive(Debug)]
struct ClientInfo {
    player_entity: Entity,
    net_obj_states: ClientNetObjStateMap,
}

impl ClientInfo {
    fn new(entity: Entity) -> Self {
        Self {
            player_entity: entity,
            net_obj_states: Default::default(),
        }
    }
}

type ServerBuffer = MessageBuffer<(ClientMessage, SocketAddr)>;

fn server_recv_messages(mut server: ResMut<NetServerSocket>, mut buffer: ResMut<ServerBuffer>) {
    while let Ok(Some((msg, peer))) = server.recv() {
        debug!("server recv msg from peer {peer:?}");
        buffer.messages.push((msg, peer));
    }
}

// TODO: improve parallelism, each message type into separate resources queues,
//       then run a system for each type
fn server_handle_messages(
    mut buffer: ResMut<ServerBuffer>,
    mut clients: ResMut<Clients>,
    mut commands: Commands,
    mut input_query: Query<&mut PlayerInput>,
    mut net_tick: EventReader<NetTick>,
) {
    if net_tick.read().count() == 0 {
        return;
    }

    let mut despawned = HashSet::new();
    for (msg, peer) in buffer.messages.drain(..) {
        debug!("server handling msg from peer {peer:?}: {msg:?}");

        match msg.body() {
            ClientMessageBody::Register => {
                clients.0.entry(peer).or_insert_with(|| {
                    info!("registering peer {peer}, spawning");

                    let player_net = PlayerNet::new_random();

                    let mut entity_commands = commands.spawn_empty();

                    entity_commands.insert(Player::server_bundle(player_net));

                    let entity = entity_commands.id();

                    ClientInfo::new(entity)
                });
            }
            ClientMessageBody::Unregister => {
                info!("unregistering peer {peer}, despawning");

                // remove from clients
                let client_info = clients.0.remove(&peer);

                // despawn entity
                if let Some(client_info) = client_info {
                    commands.entity(client_info.player_entity).despawn();
                }

                despawned.insert(peer);
            }
            ClientMessageBody::Data(client_data_body) => {
                // ignore despawned peers
                if despawned.contains(&peer) {
                    continue;
                }

                let Some(client_info) = clients.0.get_mut(&peer) else {
                    error!("peer {peer:?} has no player entity");
                    continue;
                };

                let Ok(mut player_input) = input_query.get_mut(client_info.player_entity) else {
                    error!("peer {peer:?} player entity has no PlayerInput");
                    continue;
                };

                *player_input = client_data_body.input;

                // read and update acks
                for ack in client_data_body.acks {
                    client_info
                        .net_obj_states
                        .update_recv_ack_tick(ack.net_id, ack.tick);
                }
            }
        }
    }
}

fn server_send(
    mut clients: ResMut<Clients>,
    mut socket: ResMut<NetServerSocket>,
    thingies: Query<ThingyNetQuery, With<ThingyMarker>>,
    players: Query<PlayerNetQuery, With<PlayerMarker>>,
    tick: Res<GameTick>,
    mut net_tick: EventReader<NetTick>,
) {
    if net_tick.read().count() == 0 {
        return;
    }

    // TODO: individual priorities
    for (peer, client_info) in clients.0.iter_mut() {
        let mut message = ServerMessage::new(NetHeader {}, *tick);

        let (player_id, player_transform) =
            if let Ok(peers_player) = players.get(client_info.player_entity) {
                let player_net = PlayerNet::from(peers_player);
                let player_id = player_net.player_id.0;
                let player_transform = player_net.physics.transform.0;
                let net_id = player_net.net_id;

                if let Err(e) =
                    message.try_push(ServerBodyElement::YourPlayerId(player_net.player_id.0))
                {
                    error!("can't include peer {peer:?} player ID in server message: {e:?}");
                    continue;
                }
                if let Err(e) = message.try_push(ServerBodyElement::NetObj(player_net.into())) {
                    error!("can't include peer {peer:?} player element in server message: {e:?}");
                    continue;
                }

                client_info.net_obj_states.set_sent_tick(net_id, *tick);

                (player_id, player_transform)
            } else {
                error!("peer {peer:?} player entity doesn't exist?");
                continue;
            };

        // TODO: sort by priority
        for player_query_item in players.iter() {
            let player_net = PlayerNet::from(player_query_item);
            let net_id = player_net.net_id;

            // skip peers player based on ID, since we already put it first
            if player_id == player_net.player_id.0 {
                continue;
            }

            let elem = ServerBodyElement::NetObj(player_net.into());
            if message.try_push(elem).is_err() {
                break;
            }
            client_info.net_obj_states.set_sent_tick(net_id, *tick);
        }

        // TODO: sort by priority

        for thingy_query_item in thingies.iter().sort_by::<ThingyNetQuery>(|a, b| {
            // TODO: move into another function, add stale-ness, etc.
            let distance_a = a
                .transform()
                .translation
                .distance(player_transform.translation);
            let distance_b = b
                .transform()
                .translation
                .distance(player_transform.translation);

            f32::total_cmp(&distance_a, &distance_b)
        }) {
            let thingy_net = ThingyNet::from(thingy_query_item);
            let net_id = thingy_net.net_id;

            let elem = ServerBodyElement::NetObj(thingy_net.into());
            if message.try_push(elem).is_err() {
                break;
            }
            client_info.net_obj_states.set_sent_tick(net_id, *tick);
        }

        socket.send_to(&message, *peer).unwrap();
        debug!(
            "sent packet to {peer:?} with {} body elements",
            message.body_elements().len()
        );
    }
}

// TODO: clear player input for clients who haven't sent a message in X amount of time
// TODO: auto-disconnect clients who haven't sent a message in Y amount of time

/// Server's knowledge of a client's net objects' states.
#[derive(Debug, Default)]
struct ClientNetObjStateMap(HashMap<NetId, ClientNetObjState>);

#[derive(Debug)]
struct ClientNetObjState {
    last_sent: GameTick,
    last_ack: Option<GameTick>,
}

impl ClientNetObjStateMap {
    fn set_sent_tick(&mut self, net_id: NetId, tick: GameTick) {
        match self.0.entry(net_id) {
            Entry::Occupied(mut occupied_entry) => {
                occupied_entry.get_mut().last_sent = tick;
            }
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(ClientNetObjState {
                    last_sent: tick,
                    last_ack: None,
                });
            }
        }
    }

    fn update_recv_ack_tick(&mut self, net_id: NetId, tick: GameTick) {
        // ignore acks to net ids that we never sent
        if let Some(state) = self.0.get_mut(&net_id) {
            // only advance forward
            if state.last_ack.is_none_or(|last| tick > last) {
                state.last_ack = Some(tick);
            }
        }
    }
}
