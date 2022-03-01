use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct SetupPhysicsPlugin;

impl Plugin for SetupPhysicsPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(RapierConfiguration {
			gravity: Vec2::ZERO.into(),
			// trick to avoid floating rounding problems
			scale: 20.0,
			..Default::default()
		});
	}
}
