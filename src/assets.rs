use bevy::{
    asset::{Assets, Handle},
    ecs::resource::Resource,
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
