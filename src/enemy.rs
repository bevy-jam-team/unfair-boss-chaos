use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};
use bevy_rapier2d::prelude::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
	fn build(&self, app: &mut App) {
		app.add_system(spawn_boss_reactive)
			.add_plugin(InspectorPlugin::<EnemyParams>::new());
	}
}

/// Values we might want to tweak and that are used to define specific properties of the entities.
#[derive(Inspectable)]
struct EnemyParams {
	spawn_pos: Vec2,
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

/// The float value is the player movement speed in 'pixels/second'.
#[derive(Component)]
pub struct Boss(pub f32);

/// This system spawns the boss or respawns the boss if EnemyParams have changed
fn spawn_boss_reactive(
	mut commands: Commands,
	params: Res<EnemyParams>,
	rapier_config: ResMut<RapierConfiguration>,
	mut query: Query<&mut RigidBodyPositionComponent, With<Boss>>,
) {
	info!("SPAWN_BOSS");

	if let Ok(mut rb_pos) = query.get_single_mut() {
		rb_pos.position.translation = (params.spawn_pos / rapier_config.scale).into();
	} else {
		commands
			.spawn_bundle(RigidBodyBundle {
				position: (params.spawn_pos / rapier_config.scale).into(),
				..Default::default()
			})
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
			.insert(Boss(300.0))
			.id();
	};
}
