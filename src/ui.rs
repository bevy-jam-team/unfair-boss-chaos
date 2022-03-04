use std::{cell::RefCell, rc::Rc};

use bevy::prelude::*;

use crate::{
	enemy::{Boss, BossSpawnEvent},
	game::{GameGlobals, GameState, Health, LeaderboardEvent},
	player::{Player, PlayerSpawnEvent},
	scene::MainCamera,
};

pub struct UIPlugin;

impl Plugin for UIPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(UIParams {
			health_pos: Vec2::new(0.1, 0.1),
			enemy_health_pos: Vec2::new(0.9, 0.1),
		})
		.insert_resource(UIGlobals::default())
		.add_startup_system(spawn_ui_camera)
		.add_system(spawn_health_bars)
		.add_system_set(SystemSet::on_update(GameState::GameOver).with_system(spawn_leaderboard))
		.add_system_set(SystemSet::on_update(GameState::Playing).with_system(update_health_bars))
		.add_system_set(SystemSet::on_exit(GameState::Playing).with_system(reset_state));
	}
}

struct UIParams {
	health_pos: Vec2,
	enemy_health_pos: Vec2,
}

#[derive(Default)]
struct UIGlobals {
	/// The first prop is health bar entity
	/// Second prop is health entity
	health_bars: Vec<(Entity, Entity)>,
}

fn reset_state(mut globals: ResMut<UIGlobals>) {
	globals.health_bars = vec![];
}

fn spawn_ui_camera(mut commands: Commands) {
	info!("SPAWN_UI_CAMERA");
	commands.spawn_bundle(UiCameraBundle::default());
}

fn spawn_health_bars(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	q_player: Query<(Entity, &Health), With<Player>>,
	q_boss: Query<(Entity, &Health), With<Boss>>,
	globals: ResMut<UIGlobals>,
	_ev_reader_player: EventReader<PlayerSpawnEvent>,
	_ev_reader_boss: EventReader<BossSpawnEvent>,
) {
	if globals.health_bars.len() >= 2 {
		return;
	}

	let globals = Rc::new(RefCell::new(globals));

	// get either healths from player or boss only when bar doesn't exists
	let healths = vec![(q_player.get_single(), true), (q_boss.get_single(), false)];
	let healths = healths.iter().filter_map(|(res, is_player)| {
		if let Ok((health_entity, _)) = res {
			let globals = Rc::clone(&globals);
			for (_, he) in &globals.borrow().health_bars[..] {
				if *he == *health_entity {
					return None;
				}
			}
			return Some((res.as_ref().unwrap(), is_player));
		}
		None
	});

	for ((health_entity, Health(health)), is_player) in healths {
		let (pos, text) = if *is_player {
			(
				Rect {
					bottom: Val::Px(10.0),
					left: Val::Px(10.0),
					right: Val::Undefined,
					top: Val::Undefined,
				},
				"Player",
			)
		} else {
			(
				Rect {
					bottom: Val::Px(10.0),
					left: Val::Undefined,
					right: Val::Px(10.0),
					top: Val::Undefined,
				},
				"Boss",
			)
		};
		commands
			.spawn_bundle(NodeBundle {
				style: Style {
					size: Size::new(Val::Px(400.0), Val::Px(80.0)),
					margin: Rect::all(Val::Auto),
					justify_content: JustifyContent::Center,
					align_items: AlignItems::FlexStart,
					position_type: PositionType::Relative,
					position: pos,
					..Default::default()
				},
				color: Color::GRAY.into(),
				..Default::default()
			})
			.with_children(|parent| {
				let health_bar_entity = parent
					.spawn_bundle(NodeBundle {
						style: Style {
							size: Size::new(Val::Px(400.0), Val::Px(80.0)),
							margin: Rect::all(Val::Undefined),
							justify_content: JustifyContent::Center,
							align_items: AlignItems::Center,
							position_type: PositionType::Absolute,
							position: Rect {
								top: Val::Px(0.0),
								left: Val::Px(0.0),
								bottom: Val::Px(0.9),
								right: Val::Percent(100.0 - health),
							},
							..Default::default()
						},
						color: Color::RED.into(),
						..Default::default()
					})
					.with_children(|parent| {
						parent.spawn_bundle(TextBundle {
							text: Text::with_section(
								text,
								TextStyle {
									font: asset_server.load("fonts/PressStart2P-Regular.ttf"),
									font_size: 30.0,
									color: Color::rgb(0.9, 0.9, 0.9),
								},
								Default::default(),
							),
							..Default::default()
						});
					})
					.id();

				globals
					.borrow_mut()
					.health_bars
					.push((health_bar_entity, *health_entity));
			});
	}
}

fn update_health_bars(
	globals: Res<UIGlobals>,
	q_health: Query<&Health>,
	mut q_bar_style: Query<(Entity, &mut Style)>,
) {
	for (e_bar, e_health) in &globals.health_bars {
		if let Ok(Health(health)) = q_health.get(*e_health) {
			if let Ok((_entity, mut node)) = q_bar_style.get_mut(*e_bar) {
				node.size.width = Val::Percent(*health);
			}
		}
	}
}

fn spawn_leaderboard(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	game_globals: Res<GameGlobals>,
	mut ev_reader: EventReader<LeaderboardEvent>,
) {
	if ev_reader.iter().count() == 0 {
		return;
	}

	commands
		.spawn_bundle(NodeBundle {
			style: Style {
				size: Size::new(Val::Px(400.0), Val::Px(1000.0)),
				margin: Rect::all(Val::Auto),
				justify_content: JustifyContent::Center,
				align_items: AlignItems::FlexEnd,
				position_type: PositionType::Relative,

				position: Rect::all(Val::Auto),
				..Default::default()
			},
			color: Color::GRAY.into(),
			..Default::default()
		})
		.with_children(|parent| {
			for score in &game_globals.scores {
				let text = format!("{} for: {}", score.score, score.guest);
				parent.spawn_bundle(TextBundle {
					text: Text::with_section(
						text,
						TextStyle {
							font: asset_server.load("fonts/PressStart2P-Regular.ttf"),
							font_size: 20.0,
							color: Color::rgb(0.9, 0.9, 0.9),
						},
						Default::default(),
					),
					..Default::default()
				});
			}
		});
}
