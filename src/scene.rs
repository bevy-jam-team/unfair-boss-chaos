use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct SetupScenePlugin;

impl Plugin for SetupScenePlugin {
	fn build(&self, app: &mut App) {
		app.add_startup_system(spawn_camera_and_scene);
	}
}

#[derive(Component)]
pub struct CameraTag;

/// Startup system. Spawns all the things that are necessary to render the scene
fn spawn_camera_and_scene(mut commands: Commands, rapier_parameters: Res<RapierConfiguration>) {
	info!("SPAWN_CAMERA_AND_SCENE");

	// camera
	commands
		.spawn_bundle(OrthographicCameraBundle::new_2d())
		.insert(CameraTag);

	// test dummy rigidbody
	commands
		.spawn_bundle(SpriteBundle {
			sprite: Sprite {
				color: Color::rgb(0.0, 0.0, 0.0),
				custom_size: Some(Vec2::new(50.0, 10.0)),
				..Default::default()
			},
			..Default::default()
		})
		.insert_bundle(RigidBodyBundle {
			position: RigidBodyPosition {
				position: Isometry::translation(0.0, 150.0 / rapier_parameters.scale),
				..Default::default()
			}
			.into(),
			forces: RigidBodyForces {
				torque: 2.0,
				..Default::default()
			}
			.into(),
			damping: RigidBodyDamping {
				linear_damping: 1.0,
				angular_damping: 1.0,
			}
			.into(),
			..Default::default()
		})
		.insert_bundle(ColliderBundle {
			position: Vec2::ZERO.into(),
			shape: ColliderShapeComponent(ColliderShape::cuboid(
				50.0 / rapier_parameters.scale,
				10.0 / rapier_parameters.scale,
			)),
			..Default::default()
		})
		.insert(ColliderPositionSync::Discrete);

	// TODO: spawn more level assets
}
