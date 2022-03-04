use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
	game::{GameState, Health},
	physics::PhysicsGlobals,
};

pub struct PlayerSpawnEvent;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(PlayerParams {
			start_health: 100.0,
		})
		.add_event::<PlayerSpawnEvent>()
		.add_system_set(SystemSet::on_enter(GameState::Playing).with_system(spawn_player))
		.add_system_set(SystemSet::on_update(GameState::Playing).with_system(player_movement));
	}
}

const PLAYER_SPEED_VALUE: f32 = 300.0; // Pixels / sec

/// The float value is the player movement speed in 'pixels/second'.
#[derive(Component)]
pub struct Player(pub f32);

struct PlayerParams {
	start_health: f32,
}

fn spawn_player(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	rapier_config: Res<RapierConfiguration>,
	physics_globals: Res<PhysicsGlobals>,
	params: Res<PlayerParams>,
	mut ev_writer: EventWriter<PlayerSpawnEvent>,
) {
	info!("SPAWN_PLAYER");

	// spawn player sprite with physics attached
	commands
		.spawn_bundle(SpriteBundle {
			texture: asset_server.load("physics_example/player.png"),
			sprite: Sprite {
				custom_size: Some(Vec2::new(12.6, 10.0)),
				..Default::default()
			},
			..Default::default()
		})
		.insert_bundle(RigidBodyBundle {
			position: Vec2::new(-10.0, 0.0).into(),

			..Default::default()
		})
		.insert(ColliderPositionSync::Discrete)
		.insert_bundle(ColliderBundle {
			position: Vec2::ZERO.into(),
			// Since the physics world is scaled, we divide pixel size by it to get the collider size
			shape: ColliderShapeComponent(ColliderShape::ball(10.0 / rapier_config.scale)),
			flags: ColliderFlags {
				collision_groups: InteractionGroups::new(physics_globals.player_mask, u32::MAX),
				..Default::default()
			}
			.into(),
			..Default::default()
		})
		.insert(Player(PLAYER_SPEED_VALUE))
		.insert(Health(params.start_health));

	ev_writer.send(PlayerSpawnEvent);
}

/// System that simply updated the player's velocity if buttons to move the player are pressed
pub fn player_movement(
	keyboard_input: Res<Input<KeyCode>>,
	rapier_parameters: Res<RapierConfiguration>,
	mut player_info: Query<(&Player, &mut RigidBodyVelocityComponent)>,
) {
	for (player, mut rb_vels) in player_info.iter_mut() {
		let up = keyboard_input.any_pressed([KeyCode::W, KeyCode::Up]);
		let down = keyboard_input.any_pressed([KeyCode::S, KeyCode::Down]);
		let left = keyboard_input.any_pressed([KeyCode::A, KeyCode::Left]);
		let right = keyboard_input.any_pressed([KeyCode::D, KeyCode::Right]);

		let x_axis = -(left as i8) + right as i8;
		let y_axis = -(down as i8) + up as i8;

		let mut move_delta = Vec2::new(x_axis as f32, y_axis as f32);
		if move_delta != Vec2::ZERO {
			// multiply with scale to transform pixels/sec to physical units/sec
			move_delta /= move_delta.length() * rapier_parameters.scale;
		}

		// update velocity
		rb_vels.linvel = (move_delta * player.0).into();
	}
}
