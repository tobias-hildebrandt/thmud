use std::f32::consts::PI;

use bevy::{
    app::{Plugin, Startup},
    asset::Assets,
    color::{Alpha, Color},
    ecs::system::{Commands, ResMut},
    math::{Dir3, primitives::Rectangle},
    render::mesh::{Mesh, Mesh2d},
    sprite::{ColorMaterial, MeshMaterial2d},
    transform::components::Transform,
};
use tracing::debug;

/// Plugin that spawns a visual grid onto the world.
pub(crate) struct GameDrawGridPlugin;

impl Plugin for GameDrawGridPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, spawn_grid);
    }
}

/// Spawn several grid lines.
///
/// One-time system.
// TODO: spawn them around the local player, despawn them when the player moves too far away.
// TODO: add numbers every so often
fn spawn_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    const SPACING: f32 = 100.0;
    const LINE_WIDTH: f32 = 1.0;
    const LINE_HEIGHT: f32 = 10000.0;
    const HALF_NUM_GRID_LINES: i32 = 25;

    let line = meshes.add(Rectangle::new(LINE_WIDTH, LINE_HEIGHT));
    let white = materials.add(Color::WHITE.with_alpha(0.2));
    let red = materials.add(Color::linear_rgba(1.0, 0.0, 0.0, 0.2));

    for i in -HALF_NUM_GRID_LINES..HALF_NUM_GRID_LINES {
        let coord = i as f32 * SPACING;

        debug!("grid coord: {coord}");

        let vertical = Transform::from_xyz(coord, 0.0, -1.0);
        let mut horizontal = Transform::from_xyz(0.0, coord, -1.0);

        let color = if i == 0 { red.clone() } else { white.clone() };

        horizontal.rotate_axis(Dir3::Z, PI / 2.);

        commands.spawn((
            Mesh2d(line.clone()),
            MeshMaterial2d(color.clone()),
            horizontal,
        ));
        commands.spawn((
            Mesh2d(line.clone()),
            MeshMaterial2d(color.clone()),
            vertical,
        ));
    }
}
