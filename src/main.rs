use bevy::prelude::*;

fn main() {
    App::new()
        .add_startup_system(hello_world)
        .run();
}

fn hello_world() {
    info!("Hello World");
}
