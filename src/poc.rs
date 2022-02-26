use bevy::prelude::*;

// -----------------
// PLUGIN CORE
// -----------------

pub struct PoC;

impl Plugin for PoC {
    fn build(&self, app: &mut App) {
        app.add_startup_system(poc_setup)
            .add_system_set(
                SystemSet::new()
                    .with_system(move_player).label("move_player")
                    .with_system(update_mouse_position).label("update_mouse_position")
                    .with_system(print_mouse_coords).label("print_mouse_coords")
            );
    }
}

// CONSTANTS 

const SPEED_VALUE: f32 = 150.0;

// RESOURCES

struct MousePosition {
    x_value: f32,
    y_value: f32,
}

// COMPONENTS

#[derive(Component)]
struct PlayerTag;

#[derive(Component)]
struct CameraTag;

#[derive(Component)]
struct Speed {
    value: f32,
}

// CUSTOM BUNDLES

#[derive(Bundle)]
struct PlayerBundle {
    tag: PlayerTag,
    speed: Speed,
    #[bundle]
    sprite: SpriteBundle,
}

// SYSTEMS

fn poc_setup(
    mut commands: Commands
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d()).insert(CameraTag);

    commands.spawn_bundle(PlayerBundle {
        tag: PlayerTag,
        speed: Speed { value: SPEED_VALUE },
        sprite: SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.25, 0.25, 0.75),
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, -150.0, 0.0),
                scale: Vec3::new(50.0, 50.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        }
    });

    commands.insert_resource(MousePosition { x_value: 0.0, y_value: 0.0 });
}

fn move_player(
    mut player_query: Query<(&mut Transform, &Speed), With<PlayerTag>>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (mut player_transform, speed) = player_query.single_mut();

    if keyboard_input.pressed(KeyCode::A) {
        player_transform.translation.x -= speed.value * time.delta_seconds(); 
    }
    if keyboard_input.pressed(KeyCode::D) {
        player_transform.translation.x += speed.value * time.delta_seconds(); 
    }
    if keyboard_input.pressed(KeyCode::S) {
        player_transform.translation.y -= speed.value * time.delta_seconds(); 
    }
    if keyboard_input.pressed(KeyCode::W) {
        player_transform.translation.y += speed.value * time.delta_seconds(); 
    }
}

fn update_mouse_position(
    mut mouse_position_info: ResMut<MousePosition>,
    windows_info: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<CameraTag>>
) {
    let (camera, camera_transform) = q_camera.single();
    let wnd = windows_info.get(camera.window).unwrap();
    
    if let Some(screen_pos) = wnd.cursor_position() { 
        let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix.inverse();
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
        let world_pos: Vec2 = world_pos.truncate();
        mouse_position_info.x_value = world_pos.x;
        mouse_position_info.y_value = world_pos.y;
    }
}

// BAD DEBUG CODE

fn print_mouse_coords(
    mouse_position_info: Res<MousePosition>
) {
    info!("Mouse x pos: {}", mouse_position_info.x_value);
    info!("Mouse y pos: {}", mouse_position_info.y_value);
}