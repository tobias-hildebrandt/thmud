use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
};

use bevy::{
    app::{FixedPostUpdate, FixedPreUpdate, Plugin},
    ecs::{
        entity::Entity,
        query::With,
        resource::Resource,
        schedule::IntoScheduleConfigs,
        system::{Commands, Query, Res, ResMut},
    },
};

use crate::{
    networking::messages::{ClientBodyElement, NetHeader, ServerBodyElement},
    simulation::{
        input::PlayerInput,
        player::{Player, PlayerMarker, PlayerNet, PlayerNetQuery},
        thingy::{ThingyMarker, ThingyNet, ThingyNetQuery},
    },
};

use super::{
    messages::{ClientMessage, MessageBuffer, NetMessage},
    sockets::{NetServerSocket, real::RealNetServerSocket},
};

pub struct GameServerPlugin;

impl Plugin for GameServerPlugin {
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

        app.add_systems(FixedPostUpdate, (server_send,));

        app.insert_resource(ServerBuffer::new());
    }
}

#[derive(Debug, Resource, Default)]
pub struct Clients(HashMap<SocketAddr, Entity>);

type ServerBuffer = MessageBuffer<(ClientMessage<'static>, SocketAddr)>;

fn server_recv_messages(mut server: ResMut<NetServerSocket>, mut buffer: ResMut<ServerBuffer>) {
    while let Ok(Some((msg, peer))) = server.recv() {
        // println!("server recv msg from peer {peer}");
        buffer.messages.push((msg, peer));
    }
}

// TODO: btreemap of NetId -> Entity?
// TODO: improve parallelism, each message type into separate resources queues,
//       then run a system for each type
fn server_handle_messages(
    mut buffer: ResMut<ServerBuffer>,
    mut clients: ResMut<Clients>,
    mut commands: Commands,
    mut input_query: Query<&mut PlayerInput>,
) {
    let mut despawned = HashSet::new();
    for (msg, peer) in buffer.messages.drain(..) {
        // TODO: process header
        for element in msg.body_elements() {
            match element {
                ClientBodyElement::Dummy(_cow) => {}
                ClientBodyElement::Register => {
                    clients.0.entry(peer).or_insert_with(|| {
                        println!("registering peer {peer}, spawning");

                        let player_net = PlayerNet::new_random();

                        commands.spawn(Player::bundle(player_net)).id()
                    });
                }
                ClientBodyElement::Unregister => {
                    println!("unregistering peer {peer}, despawning");

                    // remove from clients
                    let entity = clients.0.remove(&peer);

                    // despawn entity
                    if let Some(entity) = entity {
                        commands.entity(entity).despawn();
                    }

                    despawned.insert(peer);
                }
                ClientBodyElement::Input(new_input) => {
                    // ignore despawned peers
                    if despawned.contains(&peer) {
                        continue;
                    }

                    let Some(player) = clients.0.get(&peer) else {
                        println!("peer has no player entity");
                        continue;
                    };

                    let Ok(mut player_input) = input_query.get_mut(*player) else {
                        println!("peer's player entity has no PlayerInput?");
                        continue;
                    };

                    *player_input = new_input;
                }
            }
        }
    }
}

fn server_send(
    clients: Res<Clients>,
    mut socket: ResMut<NetServerSocket>,
    thingies: Query<ThingyNetQuery, With<ThingyMarker>>,
    players: Query<PlayerNetQuery, With<PlayerMarker>>,
) {
    // TODO: individual priorities
    for (peer, entity) in clients.0.iter() {
        let mut message = NetMessage::new(NetHeader {});

        let peers_player = players.get(*entity);
        let mut player_id = None;
        let mut player_transform = None;
        if let Ok(peers_player) = peers_player {
            player_id = Some(peers_player.player_id());
            player_transform = Some(peers_player.transform());
            message.push_front(ServerBodyElement::YourPlayerId(peers_player.player_id()));
            if let Err(e) =
                message.try_push_back(ServerBodyElement::Player(PlayerNet::from(peers_player)))
            {
                println!(
                    "can't include peer's player element in server message: {:?}",
                    e
                );
            }
        } else {
            println!("client player entity doesn't exist?");
        };

        // TODO: sort by priority
        for player_query_item in players.iter() {
            let player_net = PlayerNet::from(player_query_item);

            // skip peers player based on ID, since we already put it first
            if player_id.is_some_and(|player_id| player_id == player_net.player_id.0) {
                continue;
            }

            let elem = ServerBodyElement::Player(player_net);
            if message.try_push_back(elem).is_err() {
                break;
            }
        }

        // TODO: sort by priority

        for thingy_query_item in thingies.iter().sort_by::<ThingyNetQuery>(|a, b| {
            // TODO: move into another function, add stale-ness, etc.
            let distance_a = a
                .transform()
                .translation
                .distance(player_transform.unwrap_or_default().translation);
            let distance_b = b
                .transform()
                .translation
                .distance(player_transform.unwrap_or_default().translation);

            f32::total_cmp(&distance_a, &distance_b)
        }) {
            let thingy_net = ThingyNet::from(thingy_query_item);
            let elem = ServerBodyElement::Thingy(thingy_net);
            if message.try_push_back(elem).is_err() {
                break;
            }
        }

        socket.send_to(&message, *peer).unwrap();
    }
}
