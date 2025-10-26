use std::{
    collections::{HashMap, HashSet},
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
    transform::components::Transform,
};
use tracing::{debug, error, info};

use super::{
    ecs::{NetId, NetObjQuery, Networked},
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
    networking::{
        ecs::{NetObj, NetObjQueryItem},
        messages::client_messages::ClientMessageBody,
        priority::{Priority, net_obj_priority},
        tick::increment_tick,
    },
    simulation::{
        input::PlayerInput,
        player::{Player, PlayerId, PlayerMarker, PlayerNet},
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

        app.add_systems(FixedPostUpdate, server_send);

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
    net_objs: Query<NetObjQuery>,
    tick: Res<GameTick>,
    mut net_tick: EventReader<NetTick>,
    player_coords: Query<(&Transform, &Networked<PlayerId>), With<PlayerMarker>>,
) {
    if net_tick.read().count() == 0 {
        return;
    }

    // TODO: individual priorities
    for (peer, client_info) in clients.0.iter_mut() {
        let mut message = ServerMessage::new(NetHeader {}, *tick);

        // get peer player info
        let Ok((player_transform, &Networked(player_id))) =
            player_coords.get(client_info.player_entity)
        else {
            error!("peer {peer:?} has no player");
            continue;
        };

        // push player ID
        if let Err(e) = message.try_push(ServerBodyElement::YourPlayerId(player_id)) {
            error!("can't include peer {peer:?} player ID in server message: {e:?}");
            continue;
        }

        // local binds for closure
        let player_coords = player_transform.translation;
        let client_state = &mut client_info.net_obj_states;

        // sort by priority
        // TODO: cache? O(n log n) for each player
        for net_obj in
            net_objs
                .iter()
                .sort_by::<NetObjQuery>(|a: &NetObjQueryItem, b: &NetObjQueryItem| {
                    // TODO: no clone, take the &NetObjQueryItem directly in the priority function
                    let net_obj_a = NetObj::from(a.clone());
                    let net_obj_b = NetObj::from(b.clone());

                    let priority_a =
                        net_obj_priority(player_id, player_coords, &net_obj_a, client_state);
                    let priority_b =
                        net_obj_priority(player_id, player_coords, &net_obj_b, client_state);

                    // higher is better
                    Priority::partial_cmp(&priority_b, &priority_a).unwrap()
                })
        {
            let net_obj = NetObj::from(net_obj);
            let net_id = net_obj.net_id();
            let priority = net_obj_priority(player_id, player_coords, &net_obj, client_state);
            debug!("priority {priority:?}: {net_obj:?}");

            let elem = ServerBodyElement::NetObj(net_obj);
            if message.try_push(elem).is_err() {
                break;
            };
            client_state.set_sent_tick(net_id, *tick);
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
pub(super) struct ClientNetObjStateMap(pub(super) HashMap<NetId, ClientNetObjState>);

#[derive(Debug)]
pub(super) struct ClientNetObjState {
    last_sent: GameTick,
    last_ack: Option<GameTick>,
}

impl ClientNetObjState {
    pub(super) fn ticks_since_ack(&self) -> Option<u64> {
        self.last_ack.map(|l| self.last_sent - l)
    }
}

impl ClientNetObjStateMap {
    fn set_sent_tick(&mut self, net_id: NetId, tick: GameTick) {
        if let Some(state) = self.0.get_mut(&net_id) {
            state.last_sent = tick;
        } else {
            self.0.insert(
                net_id,
                ClientNetObjState {
                    last_sent: tick,
                    last_ack: None,
                },
            );
        }
    }

    fn update_recv_ack_tick(&mut self, net_id: NetId, ack_tick: GameTick) {
        // ignore acks to net ids that we never sent
        if let Some(state) = self.0.get_mut(&net_id) {
            // only advance forward, don't allow acking past the last time we sent it
            if state.last_ack.is_none_or(|last| ack_tick > last) && ack_tick <= state.last_sent {
                debug!("updating last recv ack for {net_id:?}");
                state.last_ack = Some(ack_tick);
            }
        }
    }
}
