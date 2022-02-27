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
fn spawn_camera_and_scene(mut commands: Commands) {
	// camera
	commands
		.spawn_bundle(OrthographicCameraBundle::new_2d())
		.insert(CameraTag);

	// test dummy rigidbody
	commands
		.spawn_bundle(SpriteBundle {
			transform: Transform {
				translation: Vec3::new(0.0, 100.0, 0.0),
				..Default::default()
			},
			sprite: Sprite {
				color: Color::rgb(0.0, 0.0, 0.0),
				custom_size: Some(Vec2::new(50.0, 10.0)),
				..Default::default()
			},
			..Default::default()
		})
		.insert_bundle(RigidBodyBundle::default())
		.insert_bundle(ColliderBundle {
			position: [2.5 / 2.0, 0.5 / 2.0].into(),
			..Default::default()
		});

	// TODO: spawn more level assets
}
