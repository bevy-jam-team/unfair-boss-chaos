use bevy::{core::FixedTimestep, prelude::*};
use bevy_jam_1_submission::*;
use bevy_rapier2d::prelude::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct FixedUpdateStage;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierRenderPlugin)
        .add_startup_system(setup_physics_scene)
        .add_startup_system(hello_world)
        .add_stage_after(
            // runs every 1.5 seconds to debug physics stats
            CoreStage::Update,
            FixedUpdateStage,
            SystemStage::parallel()
                .with_run_criteria(FixedTimestep::step(1.5))
                .with_system(print_physics_stats),
        )
        .add_system(player_movement)
        .run();
}

fn hello_world() {
    info!("Hello World");
}
