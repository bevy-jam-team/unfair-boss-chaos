use std::f32::consts::PI;

use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::{na::UnitComplex, prelude::*};

use crate::{
	game::{GameGlobals, GameState, Health},
	physics::PhysicsGlobals,
	player::Player,
	shooting::ShootEvent,
	waypoints::{CreatePathEvent, NextWaypoint},
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub struct AIUpdateStage;

pub struct BossSpawnEvent;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
	fn build(&self, app: &mut App) {
		app.add_system_set(SystemSet::on_enter(GameState::Playing).with_system(spawn_boss))
			.add_event::<BossSpawnEvent>()
			.add_system_set(
				SystemSet::on_update(GameState::Playing)
					.with_system(enemy_movement)
					.with_system(enemy_state_control)
					.with_system(spawn_minions),
			)
			.insert_resource(EnemyParams::default())
			.insert_resource(MinionParams::default());
		//.register_inspectable::<Enemy>()
		//.add_plugin(InspectorPlugin::<EnemyParams>::new())
	}
}

/// Values we might want to tweak and that are used to define specific properties of the entities.
#[derive(Inspectable)]
pub struct EnemyParams {
	speed: f32,
	rot_offset: f32,
	spawn_pos: Vec2,
	follow_threshold: f32,
	attack_dist: f32,
	visibility_dist: f32,
	pub start_health: f32,
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
			speed: 80.0,
			rot_offset: -PI / 2.0,
			attack_dist: 200.0,
			start_health: 100.0,
			follow_threshold: 30.0,
			visibility_dist: 400.0,
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

#[derive(Inspectable)]
struct MinionParams {
	speed: f32,
	rot_offset: f32,
	spawn_pos: Vec2,
	follow_threshold: f32,
	attack_dist: f32,
	visibility_dist: f32,
	start_health: f32,
	body_scale: Vec2,
	weapon_pos: Vec2,
	weapon_scale: Vec2,
}

impl Default for MinionParams {
	fn default() -> Self {
		Self {
			speed: 160.0,
			rot_offset: -PI / 2.0,
			attack_dist: 140.0,
			start_health: 50.0,
			follow_threshold: 30.0,
			visibility_dist: 400.0,
			spawn_pos: Vec2::new(150.0, 0.0),
			body_scale: Vec2::new(50.0, 50.0),
			weapon_pos: Vec2::new(-75.0, 20.0),
			weapon_scale: Vec2::new(10.0, 30.0),
		}
	}
}

#[derive(Component, Inspectable)]
pub struct Enemy(EnemyState);

#[derive(Component)]
pub struct Boss;

#[derive(Component)]
pub struct Minion;

#[derive(Inspectable, Debug)]
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

fn spawn_boss(
	mut commands: Commands,
	params: Res<EnemyParams>,
	rapier_config: ResMut<RapierConfiguration>,
	physics_globals: Res<PhysicsGlobals>,
	mut ev_writer: EventWriter<BossSpawnEvent>,
) {
	let collider_flags = ColliderFlags {
		collision_groups: InteractionGroups::new(physics_globals.enemy_mask, u32::MAX),
		..Default::default()
	};

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
					flags: collider_flags.clone().into(),
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
					flags: collider_flags.clone().into(),
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
					flags: collider_flags.clone().into(),
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
					flags: collider_flags.clone().into(),
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
					flags: collider_flags.clone().into(),
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
					flags: collider_flags.clone().into(),
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
					flags: collider_flags.into(),
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
		.insert(Health(params.start_health))
		.id();

	ev_writer.send(BossSpawnEvent);
}

fn spawn_minions(
	mut commands: Commands,
	params: Res<MinionParams>,
	game_globals: Res<GameGlobals>,
	rapier_config: ResMut<RapierConfiguration>,
	physics_globals: Res<PhysicsGlobals>,
	q_minions: Query<&Minion>,
	mut ev_writer: EventWriter<BossSpawnEvent>,
	_time: Res<Time>,
) {
	if q_minions.iter().count() as u32 >= game_globals.minions {
		return;
	}

	let collider_flags = ColliderFlags {
		collision_groups: InteractionGroups::new(physics_globals.enemy_mask, u32::MAX),
		..Default::default()
	};

	info!("SPAWN_MINION");
	commands
		.spawn_bundle(RigidBodyBundle {
			position: (params.spawn_pos / rapier_config.scale).into(),
			..Default::default()
		})
		.insert(Transform::from_rotation(Quat::from_euler(
			EulerRot::XYZ,
			0.0,
			0.0,
			-PI / 2.0,
		)))
		.insert_bundle(SpriteBundle {
			sprite: Sprite {
				custom_size: Some(params.body_scale),
				color: Color::RED,
				..Default::default()
			},
			..Default::default()
		})
		.insert(ColliderPositionSync::Discrete)
		.insert_bundle(ColliderBundle {
			flags: collider_flags.clone().into(),
			position: Vec2::ZERO.into(),
			// Since the physics world is scaled, we divide pixel size by it to get the collider size
			shape: ColliderShapeComponent(ColliderShape::cuboid(
				params.body_scale.x * 0.5 / rapier_config.scale,
				params.body_scale.y * 0.5 / rapier_config.scale,
			)),
			..Default::default()
		})
		.insert(Enemy(EnemyState::IDLE))
		.insert(Minion)
		.insert(Health(params.start_health))
		.id();

	ev_writer.send(BossSpawnEvent);
}

fn enemy_movement(
	mut q_enemy: Query<
		(
			&Transform,
			&mut RigidBodyVelocityComponent,
			&mut RigidBodyPositionComponent,
			&NextWaypoint,
			&Enemy,
		),
		With<Enemy>,
	>,
	q_player_t: Query<&Transform, With<Player>>,
	params: Res<EnemyParams>,
	rapier_parameters: Res<RapierConfiguration>,
	physics_globals: Res<PhysicsGlobals>,
	query_pipeline: Res<QueryPipeline>,
	collider_query: QueryPipelineColliderComponentsQuery,
	_time: Res<Time>,
) {
	let collider_set = QueryPipelineColliderComponentsSet(&collider_query);
	for (transform, mut rb_vel, mut rb_pos, next_wp, Enemy(state)) in q_enemy.iter_mut() {
		let pos = transform.translation.xy();
		match state {
			EnemyState::CHASING(Some(entity)) => {
				let target_pos = next_wp.0 .0;
				let player_pos = q_player_t.get(*entity).unwrap().translation.xy();
				let dir = target_pos - pos;
				let dir_player = player_pos - pos;
				let move_delta = dir.normalize() * params.speed / rapier_parameters.scale;

				rb_vel.linvel = move_delta.into();

				let angle = if !raycast_between(
					pos,
					player_pos,
					&query_pipeline,
					&physics_globals,
					&collider_set,
				) && dir_player.length() < params.visibility_dist
				{
					dir_player.angle_between(Vec2::X)
				} else {
					move_delta.angle_between(Vec2::X)
				};

				rb_pos.0.position.rotation = UnitComplex::from_angle(params.rot_offset - angle);
			}
			EnemyState::ATTACK(Some(entity)) => {
				let player_pos = q_player_t.get(*entity).unwrap().translation.xy();
				let dir = player_pos - transform.translation.xy();
				let move_delta = dir.normalize() * params.speed / rapier_parameters.scale;

				rb_vel.linvel = Vec2::ZERO.into();
				rb_pos.0.position.rotation =
					UnitComplex::from_angle(params.rot_offset - move_delta.angle_between(Vec2::X));
			}
			_ => {
				rb_vel.linvel = Vec2::ZERO.into();
				info!("Not moving because in state: {:?}", state);
			}
		}
	}
}

fn raycast_between(
	pos: Vec2,
	target: Vec2,
	query_pipeline: &Res<QueryPipeline>,
	physics_globals: &Res<PhysicsGlobals>,
	collider_set: &QueryPipelineColliderComponentsSet,
) -> bool {
	let dir = target - pos;
	let ray = Ray::new(pos.into(), dir.into());
	if let Some(_) = query_pipeline.cast_ray(
		collider_set,
		&ray,
		1.0,
		true,
		InteractionGroups::new(
			u32::MAX,
			u32::MAX - physics_globals.player_mask - physics_globals.enemy_mask,
		),
		None,
	) {
		return true;
	} else {
		return false;
	}
}

fn enemy_state_control(
	mut q_enemy: Query<(Entity, &Transform, &mut Enemy)>,
	q_player: Query<(Entity, &Transform), With<Player>>,
	mut ev_shoot_writer: EventWriter<ShootEvent>,
	mut create_path_ew: EventWriter<CreatePathEvent>,
	query_pipeline: Res<QueryPipeline>,
	physics_globals: Res<PhysicsGlobals>,
	params: Res<EnemyParams>,
	collider_query: QueryPipelineColliderComponentsQuery,
	_time: Res<Time>,
) {
	let collider_set = QueryPipelineColliderComponentsSet(&collider_query);
	for (entity, transform, mut enemy) in q_enemy.iter_mut() {
		match enemy.0 {
			EnemyState::IDLE => {
				if let Ok((player, _)) = q_player.get_single() {
					enemy.0 = EnemyState::CHASING(Some(player));
				}
			}
			EnemyState::FLEEING => todo!(),
			EnemyState::CHASING(Some(target)) => {
				if let Ok((player, player_t)) = q_player.get(target) {
					let player_pos = player_t.translation.xy();
					let pos = transform.translation.xy();
					let dist = player_pos.distance(pos);

					create_path_ew.send(CreatePathEvent(pos, player_pos, entity));

					if dist < params.attack_dist {
						if !raycast_between(
							pos,
							player_pos,
							&query_pipeline,
							&physics_globals,
							&collider_set,
						) {
							enemy.0 = EnemyState::ATTACK(Some(player));
						}
					}
				}
			}
			EnemyState::ATTACK(Some(target)) => {
				if let Ok((player, player_t)) = q_player.get(target) {
					let pos = transform.translation.xy();
					let dir = player_t.translation.xy() - pos;
					ev_shoot_writer.send(ShootEvent(false, pos, dir));

					let dist = player_t.translation.distance(transform.translation);
					if dist > params.attack_dist {
						enemy.0 = EnemyState::CHASING(Some(player));
					}
				}
			}
			_ => {}
		}
	}
}
