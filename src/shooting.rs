use std::f32::consts::PI;

use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::{input::MousePosition, player::Player};
use bevy_inspector_egui::{Inspectable, InspectorPlugin};

pub struct ShootingPlugin;

impl Plugin for ShootingPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<ShootEvent>()
			.add_system_set(
				SystemSet::new()
					.after("input")
					.with_system(check_for_shoot_event)
					.label("check_for_shoot_event")
					.with_system(shoot)
					.label("shoot"),
			)
			.add_plugin(InspectorPlugin::<BulletParams>::new());
	}
}

/// Values we might want to tweak and that are used to define specific properties of the entities.
#[derive(Inspectable)]
struct BulletParams {
	bullet_force_scale: f32,
	bullet_offset: f32,
}

impl Default for BulletParams {
	fn default() -> Self {
		Self {
			bullet_force_scale: 1000.0,
			bullet_offset: 0.5,
		}
	}
}

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

#[derive(Inspectable, Component, Clone)]
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
	#[bundle]
	rigidbody: RigidBodyBundle,
	#[bundle]
	collider: ColliderBundle,
}

// SYSTEMS

// The names of the systems are as expressive as possible in order to allow an easy understanding of
// what they are doing

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
	mouse_pos: Res<MousePosition>,
	player_info: Query<(&Transform, &RigidBodyPositionComponent), With<Player>>,
	rapier_parameters: Res<RapierConfiguration>,
	asset_server: Res<AssetServer>,
	params: Res<BulletParams>,
) {
	let (player_transform, player_rb_pos) = player_info.single();

	for _ in ev_shoot_reader.iter() {
		let direction = Direction {
			value: (mouse_pos.0 - player_transform.translation.xy()).normalize(),
		};
		commands
			.spawn_bundle(BulletBundle {
				tag: BulletTag,
				speed: Speed {
					value: params.bullet_force_scale,
				},
				direction: direction.clone(),
				sprite: SpriteBundle {
					texture: asset_server.load("physics_example/bullet.png"),
					sprite: Sprite {
						custom_size: Some(Vec2::new(1.0, 1.0)),
						..Default::default()
					},
					transform: Transform {
						translation: player_transform.translation,
						rotation: player_transform.rotation,
						scale: Vec3::new(15.0, 15.0, 0.0),
						..Default::default()
					},
					..Default::default()
				},
				rigidbody: RigidBodyBundle {
					position: RigidBodyPosition {
						position: player_rb_pos.position.translation
							* Isometry::from(direction.value * params.bullet_offset)
							* Isometry::rotation((direction.value.y / direction.value.x).atan()),
						..Default::default()
					}
					.into(),
					forces: RigidBodyForces {
						force: (direction.value * params.bullet_force_scale).into(),
						..Default::default()
					}
					.into(),
					..Default::default()
				},
				collider: ColliderBundle {
					flags: ColliderFlags {
						// only ignore group 0, which should be player. NOTE: only works for player spawned bullets
						collision_groups: InteractionGroups::new(u32::MAX, !0b0001),
						..Default::default()
					}
					.into(),
					..Default::default()
				},
			})
			.insert(ColliderPositionSync::Discrete);
	}
}
