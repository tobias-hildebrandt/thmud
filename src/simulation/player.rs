use bevy::{
    app::{FixedUpdate, Plugin, Startup},
    ecs::{
        bundle::Bundle,
        component::Component,
        query::AnyOf,
        schedule::IntoScheduleConfigs,
        system::{Commands, Query},
    },
    text::{Text2d, TextColor},
    transform::components::Transform,
};
use bevy_rapier2d::prelude::{
    Ccd, Collider, ColliderMassProperties, ExternalForce, LockedAxes, RigidBody, Velocity,
};

/// Marker struct for players.
#[derive(Debug, Component)]
pub struct PlayerMarker;

// TODO: move graphics out
#[derive(Debug, Bundle)]
pub struct Player {
    // game
    pub(crate) player: PlayerMarker,

    // physics
    pub(crate) rigid_body: RigidBody,
    pub(crate) collider: Collider,
    pub(crate) velocity: Velocity,
    pub(crate) transform: Transform,
    pub(crate) mass_properties: ColliderMassProperties,
    pub(crate) locked_axes: LockedAxes,
    pub(crate) external_force: ExternalForce,
    pub(crate) collision_detection: Ccd,

    // internal input forces
    pub(crate) input_force: MovementInputForce,
    pub(crate) boost_force: BoostForce,
}

#[derive(Debug, Default, Component)]
pub struct MovementInputForce(pub ExternalForce);

#[derive(Debug, Default, Component)]
pub struct BoostForce(pub ExternalForce);

#[derive(Debug, Bundle)]
pub struct PlayerText {
    pub(crate) text: Text2d,
    pub(crate) color: TextColor,
    pub(crate) transform: Transform,
}

impl Player {
    pub(crate) const DENSITY: f32 = 20.;

    pub(crate) fn create_bundle() -> impl Bundle {
        // player entity
        let player = Player {
            player: PlayerMarker,
            rigid_body: RigidBody::Dynamic,
            collider: Collider::ball(50.0),
            velocity: Velocity::zero(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            mass_properties: ColliderMassProperties::Density(Self::DENSITY),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            external_force: Default::default(),
            collision_detection: Default::default(),

            input_force: Default::default(),
            boost_force: Default::default(),
        };

        // // child entity for text
        // let text = PlayerText {
        //     text: Text2d("Player".to_string()),
        //     color: TextColor::BLACK,
        //     transform: Transform::from_xyz(0.0, 20.0, 0.0),
        // };

        (player /* Children::spawn_one(text) */,)
    }
}

fn apply_player_forces(
    query: Query<(
        &mut ExternalForce,
        AnyOf<(&MovementInputForce, &BoostForce)>,
    )>,
) {
    for (mut total, (input, boost)) in query {
        *total = input.map(|w| w.0).unwrap_or_default() + boost.map(|w| w.0).unwrap_or_default();
    }
}

fn spawn_player(mut commands: Commands) {
    // spawn player bundle
    commands.spawn(Player::create_bundle());
}

pub struct GamePlayerPlugin;

impl Plugin for GamePlayerPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(
            Startup,
            spawn_player.after(crate::assets::initialize_assets),
        )
        .add_systems(FixedUpdate, apply_player_forces);
    }
}
