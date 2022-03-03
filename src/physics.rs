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
		})
		.insert_resource(PhysicsGlobals {
			player_mask: 0b00000001,
			enemy_mask: 0b00000010,
			scene_mask: 0b00000100,
			bullet_mask: 0b00001000,
		});
	}
}

pub struct PhysicsGlobals {
	pub player_mask: u32,
	pub enemy_mask: u32,
	pub scene_mask: u32,
	pub bullet_mask: u32,
}
