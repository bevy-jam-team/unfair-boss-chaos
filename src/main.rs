use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

mod enemy;
mod game;
mod input;
mod physics;
mod player;
mod scene;
mod shooting;
mod ui;
mod waypoints;

fn main() {
	// When building for WASM, print panics to the browser console
	#[cfg(target_arch = "wasm32")]
	console_error_panic_hook::set_once();
	App::new()
		.add_plugins(DefaultPlugins)
		//.add_plugin(bevy_inspector_egui::WorldInspectorPlugin::default())
		.add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
		.add_plugin(game::GamePlugin)
		.add_plugin(input::InputPlugin)
		.add_plugin(ui::UIPlugin)
		.add_plugin(physics::SetupPhysicsPlugin)
		.add_plugin(scene::SetupScenePlugin)
		.add_plugin(shooting::ShootingPlugin)
		.add_plugin(player::PlayerPlugin)
		.add_plugin(enemy::EnemyPlugin)
		.add_plugin(waypoints::WaypointsPlugin)
		.run();
}
