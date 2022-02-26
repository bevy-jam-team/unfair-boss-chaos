use bevy::prelude::*;

mod poc;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(poc::PoC)
        .run();
}