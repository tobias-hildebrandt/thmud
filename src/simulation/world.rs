use std::collections::BTreeSet;

use bevy::{
    app::{FixedUpdate, Plugin, Startup},
    ecs::{
        query::With,
        resource::Resource,
        schedule::IntoScheduleConfigs,
        system::{Commands, Query, Res, ResMut},
    },
    math::Vec3,
    transform::components::Transform,
};
use bevy_rapier2d::prelude::Velocity;
use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::networking::ecs::{NetId, NetPhysicsBundle, Networked};

use super::thingy::{Thingy, ThingyMarker, ThingyNet};

#[derive(Debug, Resource)]
pub(crate) struct WorldSeed(i64);

fn initialize_world_seed(mut commands: Commands) {
    let world_seed = std::env::var("WORLD_SEED")
        .ok()
        .and_then(|seed| seed.parse().ok())
        .unwrap_or(0);
    commands.insert_resource(WorldSeed(world_seed));
}

const CHUNK_SIZE: i32 = 256;
const MAX_THINGIES_PER_CHUNK: usize = 5;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub(crate) struct Chunk {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl Chunk {
    pub(crate) fn from_transform(transform: &Transform) -> Self {
        Self {
            x: transform.translation.x as i32 / CHUNK_SIZE,
            y: transform.translation.y as i32 / CHUNK_SIZE,
        }
    }

    pub(crate) fn iter_nearby(&self) -> impl Iterator<Item = Self> {
        const RADIUS: i32 = 5;
        let nearby = -RADIUS..=RADIUS;
        nearby.clone().flat_map(move |delta_x| {
            nearby.clone().map(move |delta_y| Chunk {
                x: self.x + delta_x,
                y: self.y + delta_y,
            })
        })
    }
}

impl From<ChunkId> for Chunk {
    fn from(value: ChunkId) -> Self {
        let bytes = value.0.to_le_bytes();
        Self {
            x: i32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            y: i32::from_le_bytes(bytes[4..].try_into().unwrap()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub(crate) struct ChunkId(u64);

impl From<Chunk> for ChunkId {
    fn from(value: Chunk) -> Self {
        let mut bytes = [0u8; 8];
        bytes[0..4].copy_from_slice(&value.x.to_le_bytes());
        bytes[4..].copy_from_slice(&value.y.to_le_bytes());
        ChunkId(u64::from_le_bytes(bytes))
    }
}

#[derive(Debug, Resource, Default)]
pub(crate) struct SpawnedChunks {
    ids: BTreeSet<ChunkId>,
}

pub(crate) fn initialize_chunk_spawn_tracker(mut commands: Commands) {
    commands.insert_resource(SpawnedChunks::default());
}

pub(crate) fn chunk_spawning(
    mut commands: Commands,
    world_seed: Res<WorldSeed>,
    mut spawned_chunks: ResMut<SpawnedChunks>,
) {
    let current_chunk = Chunk { x: 0, y: 0 };

    for chunk in current_chunk.iter_nearby() {
        let chunk_id = ChunkId::from(chunk);
        if !spawned_chunks.ids.contains(&chunk_id) {
            // spawn the chunk!

            let chunk_seed = world_seed
                .0
                .wrapping_mul(i64::from_ne_bytes(chunk_id.0.to_ne_bytes()));

            // RNG seed based off of world seed and chunk ID
            let mut rng = StdRng::seed_from_u64(u64::from_ne_bytes(chunk_seed.to_ne_bytes()));

            // calculate number of thingies to spawn for this chunk
            let num_thingies = rng.random_range(0..MAX_THINGIES_PER_CHUNK);

            for _ in 0..num_thingies {
                // chance to spawn thingy in chunk
                let should_spawn_thingy = rng.random_bool(0.3);

                if should_spawn_thingy {
                    let chunk_offset_x = rng.random_range(0..CHUNK_SIZE);
                    let chunk_offset_y = rng.random_range(0..CHUNK_SIZE);

                    let thingy_bundle = Thingy::bundle(ThingyNet {
                        // TODO: track net-ids to avoid collisions?? 128 bit random should be fine tho
                        net_id: NetId(rng.random()),
                        physics: NetPhysicsBundle {
                            transform: Networked(Transform {
                                translation: Vec3 {
                                    x: (chunk.x * CHUNK_SIZE + chunk_offset_x) as f32,
                                    y: (chunk.y * CHUNK_SIZE + chunk_offset_y) as f32,
                                    ..Default::default()
                                },
                                ..Default::default()
                            }),
                            velocity: Default::default(),
                        },
                    });

                    commands.spawn(thingy_bundle);
                }
            }

            // chunk was "spawned" even if we didn't need to spawn any thingies
            spawned_chunks.ids.insert(chunk_id);
        }
    }
}

// TODO: remove, this is for debugging
fn randomly_add_vel_to_thingies(query: Query<&mut Velocity, With<ThingyMarker>>) {
    const RAND_VEL: f32 = 200.0;
    for mut vel in query {
        if rand::random_bool(0.95) {
            continue;
        }
        vel.linvel.x += rand::random_range(-RAND_VEL..RAND_VEL);
        vel.linvel.y += rand::random_range(-RAND_VEL..RAND_VEL);
    }
}

pub struct GameWorldGenPlugin;

impl Plugin for GameWorldGenPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(
            Startup,
            (initialize_world_seed, initialize_chunk_spawn_tracker),
        )
        .add_systems(
            FixedUpdate,
            (
                chunk_spawning,
                randomly_add_vel_to_thingies.after(chunk_spawning),
            ),
        );
    }
}
