use bevy::{
    app::{Plugin, Startup},
    diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
};
use bevy_rapier2d::render::RapierDebugRenderPlugin;
use iyes_perf_ui::{
    PerfUiPlugin,
    prelude::{PerfUiEntryEntityCount, PerfUiEntryFPS, PerfUiEntryFPSAverage},
};

// TODO: split into debug overlay (UI) and debug via logs, etc
pub(crate) struct GameDebugUiPlugin;

impl Plugin for GameDebugUiPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(RapierDebugRenderPlugin::default())
            .add_plugins((
                PerfUiPlugin,
                FrameTimeDiagnosticsPlugin::default(),
                EntityCountDiagnosticsPlugin,
            ))
            .add_systems(Startup, |mut commands: bevy::ecs::system::Commands| {
                commands.spawn((
                    PerfUiEntryFPS::default(),
                    PerfUiEntryFPSAverage::default(),
                    PerfUiEntryEntityCount::default(),
                ));
            });
    }
}
