use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

mod input;
mod physics;
mod player;
mod scene;
mod shooting;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_inspector_egui::WorldInspectorPlugin::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierRenderPlugin)
        .add_plugin(input::InputPlugin)
        .add_plugin(physics::SetupPhysicsPlugin)
        .add_plugin(scene::SetupScenePlugin)
        .add_plugin(shooting::ShootingPlugin)
        .add_plugin(player::PlayerPlugin)
        .run();
}
