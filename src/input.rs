use bevy::prelude::*;

use crate::{game::GameState, scene::MainCamera};

pub struct InputPlugin;

impl Plugin for InputPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(MousePosition(Vec2::new(0.0, 0.0)))
			.add_system_set(
				SystemSet::on_update(GameState::Playing)
					.with_system(update_mouse_position.label("input")),
			);
	}
}

/// A resource that holds the current mouse position relative to the game world. Implemented as a resource, we might want to
/// use it in other parts of the game
pub struct MousePosition(pub Vec2);

/// System that updates the MousePosition resource, so that it is available for the entire app to use
fn update_mouse_position(
	mut mouse_pos: ResMut<MousePosition>,
	windows_info: Res<Windows>,
	q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
	let (camera, camera_transform) = q_camera.single();
	let wnd = windows_info.get(camera.window).unwrap();

	if let Some(screen_pos) = wnd.cursor_position() {
		let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);
		let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
		let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix.inverse();
		let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
		let world_pos: Vec2 = world_pos.truncate();
		mouse_pos.0 = world_pos;
	}
}
