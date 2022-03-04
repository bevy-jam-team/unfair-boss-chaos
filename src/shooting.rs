use std::time::Duration;

use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::{
	game::{GameState, Health},
	input::MousePosition,
	physics::PhysicsGlobals,
	player::Player,
};
use bevy_inspector_egui::Inspectable;

pub struct ShootingPlugin;

impl Plugin for ShootingPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<ShootEvent>() // TODO: handle on bullet hit event
			.add_system_set_to_stage(
				CoreStage::Update,
				SystemSet::on_update(GameState::Playing)
					.after("input")
					.with_system(check_for_shoot_event) // TODO: check for shoot event long press
					.label("check_for_shoot_event")
					.with_system(shoot)
					.label("shoot")
					.with_system(check_bullet_hit),
			)
			.add_system_to_stage(CoreStage::Last, check_despawns)
			.insert_resource(BulletParams::default());
		//.add_plugin(InspectorPlugin::<BulletParams>::new());
	}
}

/// Values we might want to tweak and that are used to define specific properties of the entities.
#[derive(Inspectable)]
struct BulletParams {
	bullet_force_scale: f32,
	bullet_offset: f32,
	damage: f32,
	bullet_lifetime_ms: u32,
}

impl Default for BulletParams {
	fn default() -> Self {
		Self {
			bullet_force_scale: 100.0,
			bullet_offset: 0.5,
			damage: 5.0,
			bullet_lifetime_ms: 1000,
		}
	}
}

#[derive(Component)]
struct DespawnTimer(Duration, Duration);

/// used to check and trigger the shooting mechanic
/// inner value represents boolean if bullet sent from player
/// second inner value is position from bullet fire
/// third inner value is direction
pub struct ShootEvent(pub bool, pub Vec2, pub Vec2);

// COMPONENTS

/// Bullet with inner value as damage
#[derive(Inspectable, Component)]
struct Bullet(pub f32);

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
	mouse_pos: Res<MousePosition>,
	mouse_input: Res<Input<MouseButton>>,
	q_player_t: Query<&Transform, With<Player>>,
) {
	if mouse_input.just_pressed(MouseButton::Left) {
		if let Ok(player_t) = q_player_t.get_single() {
			let player_pos = player_t.translation.xy();
			let dir = mouse_pos.0 - player_pos;
			ev_shoot_writer.send(ShootEvent(true, player_pos, dir));
		}
	}
}

/// System that spawns a bullet if a ShootEvent was triggered. It just spawns a bullet in the current player position and calculates the direction
/// the bullet must follow
fn shoot(
	mut commands: Commands,
	mut ev_shoot_reader: EventReader<ShootEvent>,
	asset_server: Res<AssetServer>,
	rapier_config: Res<RapierConfiguration>,
	physics_globals: Res<PhysicsGlobals>,
	params: Res<BulletParams>,
) {
	for ShootEvent(from_player, from_pos, dir) in ev_shoot_reader.iter() {
		let direction = Direction {
			value: dir.normalize(),
		};
		let ignore_mask = if *from_player {
			physics_globals.player_mask
		} else {
			physics_globals.enemy_mask
		};
		commands
			.spawn_bundle(BulletBundle {
				speed: Speed {
					value: params.bullet_force_scale,
				},
				direction: direction.clone(),
				sprite: SpriteBundle {
					texture: asset_server.load("physics_example/bullet.png"),
					sprite: Sprite {
						custom_size: Some(Vec2::new(10.0, 10.0)),
						..Default::default()
					},
					..Default::default()
				},
				rigidbody: RigidBodyBundle {
					position: RigidBodyPosition {
						position: Isometry::translation(
							from_pos.x / rapier_config.scale,
							from_pos.y / rapier_config.scale,
						) * Isometry::from(direction.value * params.bullet_offset)
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
						// accept all bullets for now
						collision_groups: InteractionGroups::new(
							physics_globals.bullet_mask,
							u32::MAX - ignore_mask,
						),
						active_events: ActiveEvents::CONTACT_EVENTS,
						..Default::default()
					}
					.into(),
					shape: ColliderShape::cuboid(
						5.0 / rapier_config.scale,
						1.0 / rapier_config.scale,
					)
					.into(),
					..Default::default()
				},
			})
			.insert(ColliderPositionSync::Discrete)
			.insert(Bullet(params.damage));
	}
}

/// A system that listens to contact events triggered only by bullets
fn check_bullet_hit(
	mut contact_events: EventReader<ContactEvent>,
	q_bullet: Query<(Entity, &Bullet)>,
	mut commands: Commands,
	mut q_health: Query<&mut Health>,
	params: Res<BulletParams>,
	time: Res<Time>,
) {
	for contact_event in contact_events.iter() {
		if let ContactEvent::Started(h1, h2) = contact_event {
			if let Ok((e, Bullet(dmg))) = q_bullet.get(h2.entity()).or(q_bullet.get(h1.entity())) {
				if let Ok(mut health) = q_health.get_mut(h1.entity()) {
					health.0 -= dmg;
					info!("DAMAGE -> HEALTH {}", health.0);
				} else if let Ok(mut health) = q_health.get_mut(h2.entity()) {
					health.0 -= dmg;
					info!("DAMAGE -> HEALTH {}", health.0);
				}

				commands.entity(e).insert(DespawnTimer(
					Duration::new(0, params.bullet_lifetime_ms * 1000000),
					time.time_since_startup(),
				));
			}
		}
	}
}

fn check_despawns(
	mut commands: Commands,
	q_despawns: Query<(Entity, &DespawnTimer)>,
	time: Res<Time>,
) {
	for (e, DespawnTimer(lifetime, start_time)) in q_despawns.iter() {
		if time.time_since_startup() - *start_time > *lifetime {
			commands.entity(e).despawn_recursive();
		}
	}
}
