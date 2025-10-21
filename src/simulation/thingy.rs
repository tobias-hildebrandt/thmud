use std::collections::BTreeSet;

use bevy::{
    app::{FixedUpdate, Plugin, Startup},
    asset::{Assets, Handle},
    ecs::{
        bundle::Bundle,
        component::Component,
        query::With,
        resource::Resource,
        system::{Commands, Query, Res, ResMut},
    },
    render::mesh::{Mesh, Mesh2d},
    sprite::{ColorMaterial, MeshMaterial2d},
    transform::components::Transform,
};
use bevy_rapier2d::prelude::{Collider, ColliderMassProperties, RigidBody};
use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::{
    assets::AssetHandles,
    simulation::player::{Player, PlayerMarker},
};

#[derive(Debug, Component)]
pub(crate) struct ThingyMarker;

#[derive(Debug, Bundle)]
pub(crate) struct Thingy {
    pub(crate) marker: ThingyMarker,
    pub(crate) mesh: Mesh2d,
    pub(crate) mesh_material: MeshMaterial2d<ColorMaterial>,
    pub(crate) transform: Transform,
    pub(crate) rigid_body: RigidBody,
    pub(crate) collider: Collider,
    pub(crate) mass_properties: ColliderMassProperties,
}

impl Thingy {
    pub(crate) fn create_bundle(
        x: f32,
        y: f32,
        fixed: bool,
        is_circle: bool,
        color: Handle<ColorMaterial>,
        mesh: Handle<Mesh>,
    ) -> Self {
        Self {
            marker: ThingyMarker,
            transform: Transform::from_xyz(x, y, 0.0),
            mesh: Mesh2d(mesh),
            mesh_material: MeshMaterial2d(color),
            rigid_body: if fixed {
                RigidBody::Fixed
            } else {
                RigidBody::Dynamic
            },
            collider: match is_circle {
                true => Collider::ball(50.),
                false => Collider::cuboid(25.0, 25.0),
            },
            mass_properties: ColliderMassProperties::Density(Player::DENSITY / 2.),
        }
    }
}

#[derive(Debug, Resource)]
pub(crate) struct WorldSeed(i64);

fn initialize_world_seed(mut commands: Commands) {
    let world_seed = std::env::var("WORLD_SEED")
        .map_err(|_e| ())
        .and_then(|seed| seed.parse().map_err(|_e| ()))
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
    player_coords: Query<&Transform, With<PlayerMarker>>,
    mut asset_handles: ResMut<AssetHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let player_coords = player_coords.single().unwrap();
    let current_chunk = Chunk::from_transform(player_coords);

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
                    let is_fixed = rng.random_bool(0.3);
                    let is_circle = rng.random_bool(0.5);
                    let radius = rng.random_range(0..32) as f32;

                    let chunk_offset_x = rng.random_range(0..CHUNK_SIZE);
                    let chunk_offset_y = rng.random_range(0..CHUNK_SIZE);

                    let thingy_bundle = Thingy::create_bundle(
                        (chunk.x * CHUNK_SIZE + chunk_offset_x) as f32,
                        (chunk.y * CHUNK_SIZE + chunk_offset_y) as f32,
                        is_fixed,
                        is_circle,
                        asset_handles.thingy_mat.clone(),
                        if is_circle {
                            asset_handles.thingy_circle_mesh(radius, &mut meshes)
                        } else {
                            asset_handles.thingy_square_mesh(radius, &mut meshes)
                        },
                    );

                    commands.spawn(thingy_bundle);
                }
            }

            // chunk was "spawned" even if we didn't need to spawn any thingies
            spawned_chunks.ids.insert(chunk_id);
        }
    }
}

pub struct GameThingyPlugin;

impl Plugin for GameThingyPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(
            Startup,
            (initialize_world_seed, initialize_chunk_spawn_tracker),
        )
        .add_systems(FixedUpdate, chunk_spawning);
    }
}
