use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(hello_world)
        .run();
}

fn hello_world() {
    info!("Hello World");
}
