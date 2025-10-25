use bevy::{
    app::{Plugin, Startup},
    asset::{Assets, Handle},
    color::Color,
    ecs::{
        resource::Resource,
        system::{Commands, ResMut},
    },
    math::primitives::{Circle, Rectangle},
    render::mesh::Mesh,
    sprite::ColorMaterial,
};
use fixed::FixedI32;
use fixed::types::extra::U12;
use std::collections::HashMap;

#[derive(Resource)]
pub(crate) struct AssetHandles {
    pub(crate) player_mesh: Handle<Mesh>,
    pub(crate) player_mat: Handle<ColorMaterial>,
    pub(crate) thingy_circle_meshes: HashMap<FixedI32<U12>, Handle<Mesh>>,
    pub(crate) thingy_square_meshes: HashMap<FixedI32<U12>, Handle<Mesh>>,
    pub(crate) thingy_mat: Handle<ColorMaterial>,
}

impl AssetHandles {
    pub(crate) fn thingy_circle_mesh(
        &mut self,
        radius: f32,
        meshes: &mut Assets<Mesh>,
    ) -> Handle<Mesh> {
        self.thingy_circle_meshes
            .entry(FixedI32::from_num(radius))
            .or_insert(meshes.add(Circle::new(radius)))
            .clone()
    }

    pub(crate) fn thingy_square_mesh(
        &mut self,
        side: f32,
        meshes: &mut Assets<Mesh>,
    ) -> Handle<Mesh> {
        self.thingy_square_meshes
            .entry(FixedI32::from_num(side))
            .or_insert(meshes.add(Rectangle::new(side, side)))
            .clone()
    }
}

pub(crate) fn initialize_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let assets = AssetHandles {
        player_mesh: meshes.add(Circle::new(50.0)),
        player_mat: materials.add(Color::hsl(180., 0.95, 0.7)),
        thingy_circle_meshes: HashMap::new(),
        thingy_square_meshes: HashMap::new(),
        thingy_mat: materials.add(Color::hsl(110., 0.45, 0.4)),
    };

    commands.insert_resource(assets);
}

pub struct GameAssetPlugin;

impl Plugin for GameAssetPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, initialize_assets);
    }
}
