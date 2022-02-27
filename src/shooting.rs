use bevy::prelude::*;

use crate::{input::MousePosition, player::Player};
use bevy_inspector_egui::Inspectable;

pub struct ShootingPlugin;

impl Plugin for ShootingPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<ShootEvent>().add_system_set(
			SystemSet::new()
				.after("input")
				.with_system(check_for_shoot_event)
				.label("check_for_shoot_event")
				.with_system(shoot)
				.label("shoot")
				.with_system(move_bullets)
				.label("move_bullets"),
		);
	}
}

/// Values we might want to tweak and that are used to define specific properties of the entities.
const BULLET_SPEED_VALUE: f32 = 300.0;

/// used to check and trigger the shooting mechanic
struct ShootEvent;

// COMPONENTS

/// Just used tags component to be able to identify specific entities to retrieve
#[derive(Inspectable, Component)]
struct BulletTag;

// Components used to hold informations and data realtive to the entity they are attached to

#[derive(Inspectable, Component)]
struct Speed {
	value: f32,
}

#[derive(Inspectable, Component)]
struct Direction {
	value: Vec2,
}

// CUSTOM BUNDLES

/// Just custom bundles, to spawn a specific entity without the need to insert every time the specific
/// components related to that entity
#[derive(Bundle)]
struct BulletBundle {
	tag: BulletTag,
	speed: Speed,
	direction: Direction,
	#[bundle]
	sprite: SpriteBundle,
}

// SYSTEMS

// The names of the systems are as expressive as possible in order to allow an easy understanding of
// what they are doing

/// System that moves the bullets according to their direction and speed (direction is calculated when the bullet is spawned)
fn move_bullets(
	mut bullets_query: Query<(&mut Transform, &Direction, &Speed), With<BulletTag>>,
	time: Res<Time>,
) {
	for (mut bullet_transform, bullet_direction, bullet_speed) in bullets_query.iter_mut() {
		bullet_transform.translation.x +=
			bullet_direction.value.x * bullet_speed.value * time.delta_seconds();
		bullet_transform.translation.y +=
			bullet_direction.value.y * bullet_speed.value * time.delta_seconds();
	}
}

/// System that checks if the mouse button has been pressed. If so, queues a new event to shoot a bullet
fn check_for_shoot_event(
	mut ev_shoot_writer: EventWriter<ShootEvent>,
	mouse_input: Res<Input<MouseButton>>,
) {
	if mouse_input.just_pressed(MouseButton::Left) {
		ev_shoot_writer.send(ShootEvent);
	}
}

/// System that spawns a bullet if a ShootEvent was triggered. It just spawns a bullet in the current player position and calculates the direction
/// the bullet must follow
fn shoot(
	mut commands: Commands,
	mut ev_shoot_reader: EventReader<ShootEvent>,
	mouse_info: Res<MousePosition>,
	player_info: Query<&Transform, With<Player>>,
) {
	let player_transform = player_info.single();

	for _ in ev_shoot_reader.iter() {
		commands.spawn_bundle(BulletBundle {
			tag: BulletTag,
			speed: Speed {
				value: BULLET_SPEED_VALUE,
			},
			direction: Direction {
				value: Vec2::new(
					mouse_info.x - player_transform.translation.x,
					mouse_info.y - player_transform.translation.y,
				)
				.normalize(),
			},
			sprite: SpriteBundle {
				sprite: Sprite {
					color: Color::rgb(0.75, 0.75, 0.75),
					..Default::default()
				},
				transform: Transform {
					translation: Vec3::new(
						player_transform.translation.x,
						player_transform.translation.y,
						0.0,
					),
					scale: Vec3::new(15.0, 15.0, 0.0),
					..Default::default()
				},
				..Default::default()
			},
		});
	}
}
