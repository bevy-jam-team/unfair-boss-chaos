use bevy::{core::FixedTimestep, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::player::Player;

pub struct SetupPhysicsPlugin;

impl Plugin for SetupPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_physics_params)
            .add_stage_after(
                // runs every 1.5 seconds to debug physics stats
                CoreStage::Update,
                FixedUpdateStage,
                SystemStage::parallel()
                    .with_run_criteria(FixedTimestep::step(1.5))
                    .with_system(print_physics_stats),
            );
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub struct FixedUpdateStage;

fn setup_physics_params(mut rapier_config: ResMut<RapierConfiguration>) {
    // configure 0 gravity
    rapier_config.gravity = Vec2::ZERO.into();

    // trick to avoid floating rounding problems
    rapier_config.scale = 20.0;
}

fn print_physics_stats(
    positions: Query<&RigidBodyPositionComponent>,
    mut player_info: Query<(&Player, &mut Transform)>,
) {
    for rb_pos in positions.iter() {
        info!(
            "Ball physics position: {:?},  Ball transform position: {:?}",
            rb_pos.position.translation.vector,
            player_info.single_mut().1.translation
        );
    }
}
