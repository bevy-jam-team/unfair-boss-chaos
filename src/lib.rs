use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// The float value is the player movement speed in 'pixels/second'.
#[derive(Component)]
pub struct Player(f32);

pub fn setup_physics_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    // configure 0 gravity
    rapier_config.gravity = Vec2::ZERO.into();

    let scale_inv = 10.0;
    let sprite_size_x = 126.0 / scale_inv; // Make sure aspect ratio matches player.png!
    let sprite_size_y = 100.0 / scale_inv;

    // trick to avoid floating rounding problems
    rapier_config.scale = 20.0;
    let collider_size_x = sprite_size_x / rapier_config.scale;
    let collider_size_y = sprite_size_y / rapier_config.scale;

    // setup camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // spawn player sprite with physics attached and initial torque
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("physics_example/player.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::new(sprite_size_x, sprite_size_y)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert_bundle(RigidBodyBundle {
            position: Vec2::new(0.0, 0.0).into(),
            forces: RigidBodyForces {
                torque: 1.0,
                ..Default::default()
            }
            .into(),
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::ball(0.5).into(),
            material: ColliderMaterial {
                restitution: 0.7,
                ..Default::default()
            }
            .into(),
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(Player(300.0));

    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 100.0, 0.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::rgb(0.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(sprite_size_x, sprite_size_y)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert_bundle(RigidBodyBundle::default())
        .insert_bundle(ColliderBundle {
            position: [collider_size_x / 2.0, collider_size_y / 2.0].into(),
            ..Default::default()
        });
}

pub fn print_physics_stats(
    positions: Query<&RigidBodyPositionComponent>,
    mut player_info: Query<(&Player, &mut Transform)>,
) {
    for rb_pos in positions.iter() {
        info!(
            "Ball physics position: {:?},  Ball transform position: {:?}",
            rb_pos.position.translation.vector,
            player_info.single_mut().1.translation
        );
    }
}

pub fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    rapier_parameters: Res<RapierConfiguration>,
    mut player_info: Query<(&Player, &mut RigidBodyVelocityComponent)>,
) {
    for (player, mut rb_vels) in player_info.iter_mut() {
        let up = keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up);
        let down = keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down);
        let left = keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left);
        let right = keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right);

        let x_axis = -(left as i8) + right as i8;
        let y_axis = -(down as i8) + up as i8;

        let mut move_delta = Vec2::new(x_axis as f32, y_axis as f32);
        if move_delta != Vec2::ZERO {
            // multiply with scale to transform pixels/sec to physical units/sec
            move_delta /= move_delta.length() * rapier_parameters.scale;
        }

        // update velocity
        rb_vels.linvel = (move_delta * player.0).into();
    }
}
