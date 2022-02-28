use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::input::MousePosition;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app.add_startup_system(spawn_player)
			.add_system(player_movement);
	}
}

const PLAYER_SPEED_VALUE: f32 = 300.0; // Pixels / sec

/// The float value is the player movement speed in 'pixels/second'.
#[derive(Component)]
pub struct Player(pub f32);

pub fn spawn_player(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	rapier_config: ResMut<RapierConfiguration>,
) {
	info!("SPAWN_PLAYER");

	// spawn player sprite with physics attached and initial torque
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
			position: Vec2::new(0.0, 0.0).into(),
			forces: RigidBodyForces {
				..Default::default()
			}
			.into(),
			..Default::default()
		})
		.insert(ColliderPositionSync::Discrete)
		.insert_bundle(ColliderBundle {
			position: Vec2::ZERO.into(),
			// Since the physics world is scaled, we divide pixel size by it to get the collider size
			shape: ColliderShapeComponent(ColliderShape::ball(10.0 / rapier_config.scale)),
			..Default::default()
		})
		.insert(Player(PLAYER_SPEED_VALUE));
}

/// System that simply updated the player's velocity if buttons to move the player are pressed
pub fn player_movement(
	keyboard_input: Res<Input<KeyCode>>,
	rapier_parameters: Res<RapierConfiguration>,
	mouse_pos: Res<MousePosition>,
	mut player_info: Query<(
		&Player,
		&mut RigidBodyVelocityComponent,
		&mut RigidBodyPositionComponent,
	)>,
) {
	for (player, mut rb_vels, mut rb_pos) in player_info.iter_mut() {
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
