use std::f32::consts::PI;

use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_inspector_egui::{Inspectable, InspectorPlugin, RegisterInspectable};
use bevy_rapier2d::{na::UnitComplex, prelude::*};

use crate::player::Player;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
	fn build(&self, app: &mut App) {
		app.register_inspectable::<Enemy>()
			.add_system_to_stage(CoreStage::Last, spawn_boss_reactive)
			.add_plugin(InspectorPlugin::<EnemyParams>::new())
			.add_system(enemy_state_control)
			.add_system(enemy_movement);
	}
}

/// Values we might want to tweak and that are used to define specific properties of the entities.
#[derive(Inspectable)]
struct EnemyParams {
	speed: f32,
	rot_offset: f32,
	spawn_pos: Vec2,
	attack_dist: f32,
	body_scale: Vec2,
	left_arm_pos: Vec2,
	left_arm_scale: Vec2,
	left_arm_rot: f32,
	right_arm_pos: Vec2,
	right_arm_scale: Vec2,
	right_arm_rot: f32,
	left_shield_pos: Vec2,
	left_shield_scale: Vec2,
	left_shield_rot: f32,
	right_shield_pos: Vec2,
	right_shield_scale: Vec2,
	right_shield_rot: f32,
	left_weapon_pos: Vec2,
	left_weapon_scale: Vec2,
	right_weapon_pos: Vec2,
	right_weapon_scale: Vec2,
}

impl Default for EnemyParams {
	fn default() -> Self {
		Self {
			speed: 20.0,
			rot_offset: -PI / 2.0,
			attack_dist: 4.0,
			spawn_pos: Vec2::new(150.0, 0.0),
			body_scale: Vec2::new(100.0, 100.0),
			// arms
			left_arm_pos: Vec2::new(-75.0, 0.0),
			left_arm_scale: Vec2::new(40.0, 40.0),
			left_arm_rot: 0.1,
			right_arm_pos: Vec2::new(75.0, 0.0),
			right_arm_scale: Vec2::new(40.0, 40.0),
			right_arm_rot: -0.1,
			// shields
			left_shield_pos: Vec2::new(-75.0, 60.0),
			left_shield_scale: Vec2::new(100.0, 5.0),
			left_shield_rot: PI / 4.0,
			right_shield_pos: Vec2::new(75.0, 60.0),
			right_shield_scale: Vec2::new(100.0, 5.0),
			right_shield_rot: -PI / 4.0,
			// weapons
			left_weapon_pos: Vec2::new(-75.0, 20.0),
			left_weapon_scale: Vec2::new(10.0, 30.0),
			right_weapon_pos: Vec2::new(75.0, 20.0),
			right_weapon_scale: Vec2::new(10.0, 30.0),
		}
	}
}

#[derive(Component, Inspectable)]
pub struct Enemy(EnemyState);

#[derive(Component)]
pub struct Boss;

#[derive(Component)]
pub struct Minion;

#[derive(Inspectable)]
pub enum EnemyState {
	IDLE,
	FLEEING,
	CHASING(Option<Entity>),
	ATTACK(Option<Entity>),
}

impl Default for EnemyState {
	fn default() -> Self {
		EnemyState::IDLE
	}
}

/// This system spawns the boss or respawns the boss if EnemyParams have changed
fn spawn_boss_reactive(
	mut commands: Commands,
	params: Res<EnemyParams>,
	rapier_config: ResMut<RapierConfiguration>,
	mut query: Query<Entity, With<Boss>>,
) {
	if !params.is_changed() {
		return;
	}

	if let Ok(entity) = query.get_single_mut() {
		commands.entity(entity).despawn_recursive();
	}

	info!("SPAWN_BOSS");
	commands
		.spawn_bundle(RigidBodyBundle {
			position: (params.spawn_pos / rapier_config.scale).into(),
			..Default::default()
		})
		.insert(RigidBodyPositionSync::Discrete)
		.insert(Transform::from_rotation(Quat::from_euler(
			EulerRot::XYZ,
			0.0,
			0.0,
			-PI / 2.0,
		)))
		.with_children(|parent| {
			// body
			parent
				.spawn_bundle(SpriteBundle {
					sprite: Sprite {
						custom_size: Some(params.body_scale),
						color: Color::RED,
						..Default::default()
					},
					..Default::default()
				})
				.insert(ColliderPositionSync::Discrete)
				.insert_bundle(ColliderBundle {
					position: Vec2::ZERO.into(),
					// Since the physics world is scaled, we divide pixel size by it to get the collider size
					shape: ColliderShapeComponent(ColliderShape::cuboid(
						params.body_scale.x * 0.5 / rapier_config.scale,
						params.body_scale.y * 0.5 / rapier_config.scale,
					)),
					..Default::default()
				});

			// left arm
			parent
				.spawn_bundle(SpriteBundle {
					sprite: Sprite {
						custom_size: Some(params.left_arm_scale),
						color: Color::RED,
						..Default::default()
					},
					..Default::default()
				})
				.insert(ColliderPositionSync::Discrete)
				.insert_bundle(ColliderBundle {
					position: (
						params.left_arm_pos / rapier_config.scale,
						params.left_arm_rot,
					)
						.into(),
					// Since the physics world is scaled, we divide pixel size by it to get the collider size
					shape: ColliderShapeComponent(ColliderShape::cuboid(
						params.left_arm_scale.x * 0.5 / rapier_config.scale,
						params.left_arm_scale.y * 0.5 / rapier_config.scale,
					)),
					..Default::default()
				});

			// right arm
			parent
				.spawn_bundle(SpriteBundle {
					sprite: Sprite {
						custom_size: Some(params.right_arm_scale),
						color: Color::RED,
						..Default::default()
					},
					..Default::default()
				})
				.insert(ColliderPositionSync::Discrete)
				.insert_bundle(ColliderBundle {
					position: (
						params.right_arm_pos / rapier_config.scale,
						params.right_arm_rot,
					)
						.into(),
					// Since the physics world is scaled, we divide pixel size by it to get the collider size
					shape: ColliderShapeComponent(ColliderShape::cuboid(
						params.right_arm_scale.x * 0.5 / rapier_config.scale,
						params.right_arm_scale.y * 0.5 / rapier_config.scale,
					)),
					..Default::default()
				});

			// left shield
			parent
				.spawn_bundle(SpriteBundle {
					sprite: Sprite {
						custom_size: Some(params.left_shield_scale),
						color: Color::ALICE_BLUE,
						..Default::default()
					},
					..Default::default()
				})
				.insert(ColliderPositionSync::Discrete)
				.insert_bundle(ColliderBundle {
					position: (
						params.left_shield_pos / rapier_config.scale,
						params.left_shield_rot,
					)
						.into(),
					// Since the physics world is scaled, we divide pixel size by it to get the collider size
					shape: ColliderShapeComponent(ColliderShape::cuboid(
						params.left_shield_scale.x * 0.5 / rapier_config.scale,
						params.left_shield_scale.y * 0.5 / rapier_config.scale,
					)),
					..Default::default()
				});

			// right shield
			parent
				.spawn_bundle(SpriteBundle {
					sprite: Sprite {
						custom_size: Some(params.right_shield_scale),
						color: Color::ALICE_BLUE,
						..Default::default()
					},
					..Default::default()
				})
				.insert(ColliderPositionSync::Discrete)
				.insert_bundle(ColliderBundle {
					position: (
						params.right_shield_pos / rapier_config.scale,
						params.right_shield_rot,
					)
						.into(),
					// Since the physics world is scaled, we divide pixel size by it to get the collider size
					shape: ColliderShapeComponent(ColliderShape::cuboid(
						params.right_shield_scale.x * 0.5 / rapier_config.scale,
						params.right_shield_scale.y * 0.5 / rapier_config.scale,
					)),
					..Default::default()
				});

			// left weapon
			parent
				.spawn_bundle(SpriteBundle {
					sprite: Sprite {
						custom_size: Some(params.left_weapon_scale),
						color: Color::BLUE,
						..Default::default()
					},
					..Default::default()
				})
				.insert(ColliderPositionSync::Discrete)
				.insert_bundle(ColliderBundle {
					position: (params.left_weapon_pos / rapier_config.scale).into(),
					// Since the physics world is scaled, we divide pixel size by it to get the collider size
					shape: ColliderShapeComponent(ColliderShape::cuboid(
						params.left_weapon_scale.x * 0.5 / rapier_config.scale,
						params.left_weapon_scale.y * 0.5 / rapier_config.scale,
					)),
					..Default::default()
				});

			// right weapon
			parent
				.spawn_bundle(SpriteBundle {
					sprite: Sprite {
						custom_size: Some(params.right_weapon_scale),
						color: Color::BLUE,
						..Default::default()
					},
					..Default::default()
				})
				.insert(ColliderPositionSync::Discrete)
				.insert_bundle(ColliderBundle {
					position: (params.right_weapon_pos / rapier_config.scale).into(),
					// Since the physics world is scaled, we divide pixel size by it to get the collider size
					shape: ColliderShapeComponent(ColliderShape::cuboid(
						params.right_weapon_scale.x * 0.5 / rapier_config.scale,
						params.right_weapon_scale.y * 0.5 / rapier_config.scale,
					)),
					..Default::default()
				});
		})
		.insert(Enemy(EnemyState::IDLE))
		.insert(Boss)
		.id();
}

fn enemy_movement(
	mut q_enemy: Query<
		(
			&Transform,
			&mut RigidBodyVelocityComponent,
			&mut RigidBodyPositionComponent,
			&Enemy,
		),
		With<Enemy>,
	>,
	q_target: Query<&Transform>,
	params: Res<EnemyParams>,
	rapier_parameters: Res<RapierConfiguration>,
	_time: Res<Time>,
) {
	for (transform, mut rb_vel, mut rb_pos, Enemy(state)) in q_enemy.iter_mut() {
		if let EnemyState::CHASING(Some(entity)) = state {
			if let Ok(pos) = q_target.get(*entity) {
				let move_delta = (pos.translation.xy() - transform.translation.xy()).normalize()
					* params.speed / rapier_parameters.scale;
				rb_vel.linvel = move_delta.into();
				rb_pos.0.position.rotation =
					UnitComplex::from_angle(params.rot_offset - move_delta.angle_between(Vec2::X));
			}
		}
	}
}

fn enemy_state_control(
	mut commands: Commands,
	q_enemy: Query<(Entity, &Transform, &Enemy), With<Enemy>>,
	q_player: Query<(Entity, &Transform), With<Player>>,
	params: Res<EnemyParams>,
	_time: Res<Time>,
) {
	for (entity, transform, Enemy(state)) in q_enemy.iter() {
		info!("Found {:?}", entity);
		match state {
			EnemyState::IDLE => {
				if let Ok((player, _)) = q_player.get_single() {
					commands
						.entity(entity)
						.remove::<Enemy>()
						.insert(Enemy(EnemyState::CHASING(Some(player))));
				}
			}
			EnemyState::FLEEING => todo!(),
			EnemyState::CHASING(Some(target)) => {
				if let Ok((player, player_t)) = q_player.get(*target) {
					let dist = player_t.translation.distance(transform.translation);
					if dist < params.attack_dist {
						commands
							.entity(entity)
							.remove::<Enemy>()
							.insert(Enemy(EnemyState::ATTACK(Some(player))));
					}
				}
			}
			EnemyState::ATTACK(Some(target)) => {
				if let Ok((player, player_t)) = q_player.get(*target) {
					let dist = player_t.translation.distance(transform.translation);
					if dist > params.attack_dist {
						commands
							.entity(entity)
							.remove::<Enemy>()
							.insert(Enemy(EnemyState::CHASING(Some(player))));
					}
				}
			}
			_ => {}
		}
	}
}
