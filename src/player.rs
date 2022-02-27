use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

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
	let scale_inv = 10.0;
	let sprite_size_x = 126.0 / scale_inv; // Make sure aspect ratio matches player.png!
	let sprite_size_y = 100.0 / scale_inv;

	// Since the physics world is scaled, we divide pixel size by it to get the collider size
	let collider_size_x = sprite_size_x / rapier_config.scale;
	let collider_size_y = sprite_size_y / rapier_config.scale;

	// spawn player sprite with physics attached and initial torque
	commands
		.spawn_bundle(SpriteBundle {
			texture: asset_server.load("physics_example/player.png"),
			sprite: Sprite {
				custom_size: Some(Vec2::new(sprite_size_x, sprite_size_y)),
				..Default::default()
			},
			..Default::default()
		})
		.insert_bundle(RigidBodyBundle {
			position: Vec2::new(0.0, 0.0).into(),
			forces: RigidBodyForces {
				torque: 1.0,
				..Default::default()
			}
			.into(),
			..Default::default()
		})
		.insert_bundle(ColliderBundle {
			position: [collider_size_x / 2.0, collider_size_y / 2.0].into(),
			..Default::default()
		})
		.insert(ColliderPositionSync::Discrete)
		.insert(Player(PLAYER_SPEED_VALUE));
}

/// System that simply updated the player's velocity if buttons to move the player are pressed
pub fn player_movement(
	keyboard_input: Res<Input<KeyCode>>,
	rapier_parameters: Res<RapierConfiguration>,
	mut player_info: Query<(&Player, &mut RigidBodyVelocityComponent)>,
) {
	for (player, mut rb_vels) in player_info.iter_mut() {
		let up = keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up);
		let down = keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down);
		let left = keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left);
		let right = keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right);

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
