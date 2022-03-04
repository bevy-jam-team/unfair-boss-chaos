use std::time::Duration;

use bevy::{
	ecs::schedule::ShouldRun,
	prelude::*,
	tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::JsFuture;
use web_sys::Response;

use crate::{
	enemy::{Boss, EnemyParams},
	player::Player,
};

/// Plugin that handles when game restarts and tracks the player's score.
/// The game restarts when player dies, so player's health is tracked
/// Score is how long the player stays alive, given the current upgrade level of the boss (inc. enemy)
pub struct GamePlugin;

pub struct LeaderboardEvent;

impl Plugin for GamePlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(GameGlobals {
			level: 1,
			minions: 0,
			min_upgrade_health: 20.0,
			time_until_restart: Duration::from_secs(15),
			scores: vec![],
			..Default::default()
		})
		.add_event::<LeaderboardEvent>()
		.add_state(GameState::Playing)
		.add_system_set(
			SystemSet::on_update(GameState::Playing)
				.with_system(restart_game_when_player_dies)
				.with_system(update_score)
				.with_system(update_level_over_time),
		)
		.add_system_set(SystemSet::on_enter(GameState::Playing).with_system(reset_game_globals))
		.add_system_set(SystemSet::on_exit(GameState::Playing).with_system(teardown))
		.add_system_set(
			SystemSet::on_enter(GameState::GameOver)
				.with_system(upload_highscores)
				.with_system(display_highscores_when_loaded),
		)
		.add_system_set(SystemSet::on_update(GameState::GameOver).with_system(restart_game_timer))
		.add_system_set(SystemSet::on_exit(GameState::GameOver).with_system(teardown));
	}
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
	Playing,
	GameOver,
}

#[derive(Default)]
pub struct GameGlobals {
	pub level: u32,
	pub score: u32,
	pub time_started: Duration,
	pub minions: u32,
	pub min_upgrade_health: f32,
	pub scores: Vec<LeaderboardScore>,
	pub time_stopped: Duration,
	pub time_until_restart: Duration,
}

#[derive(Component)]
pub struct Health(pub f32);

fn reset_game_globals(mut globals: ResMut<GameGlobals>, time: Res<Time>) {
	globals.time_started = time.time_since_startup();
	globals.level = 1;
	globals.score = 0;
	globals.minions = 0;
}

pub fn run_when_enter_playing_state(
	state: Res<State<GameState>>,
	globals: Res<GameGlobals>,
	time: Res<Time>,
) -> ShouldRun {
	if *state.current() == GameState::Playing && time.time_since_startup() == globals.time_started {
		ShouldRun::YesAndCheckAgain
	} else {
		ShouldRun::NoAndCheckAgain
	}
}

fn update_level_over_time(
	mut q_health: Query<&mut Health, With<Boss>>,
	enemy_params: ResMut<EnemyParams>,
	mut state: ResMut<State<GameState>>,
	time: Res<Time>,
	mut globals: ResMut<GameGlobals>,
) {
	if let Ok(mut health) = q_health.get_single_mut() {
		if health.0 < globals.min_upgrade_health {
			health.0 = enemy_params.start_health;
			globals.level += 1;
			globals.minions += globals.level;
		}
	}
}

fn restart_game_when_player_dies(
	q_player: Query<&Health, With<Player>>,
	mut state: ResMut<State<GameState>>,
	time: Res<Time>,
	mut globals: ResMut<GameGlobals>,
) {
	for Health(health) in q_player.iter() {
		if *health <= 0.0 {
			let _ = state.overwrite_set(GameState::GameOver);
			globals.time_stopped = time.time_since_startup();
		}
	}
}

fn upload_highscores(globals: Res<GameGlobals>, thread_pool: Res<AsyncComputeTaskPool>) {
	// publish highscores to web api
	let score = globals.score;
	thread_pool.spawn(async move {
		let _ = Leaderboard::add_score(score, format!("player-{}", rand::random::<u32>()).as_str())
			.await;
		let res = Leaderboard::leaderboard().await.unwrap();
		res.scores
	});
}

fn display_highscores_when_loaded(
	mut commands: Commands,
	mut globals: ResMut<GameGlobals>,
	mut transform_tasks: Query<(Entity, &mut Task<Vec<LeaderboardScore>>)>,
) {
	for (entity, mut task) in transform_tasks.iter_mut() {
		if let Some(scores) = future::block_on(future::poll_once(&mut *task)) {
			globals.scores = scores;
			// Task is complete, so remove task component from entity
			commands.entity(entity).remove::<Task<Transform>>();
		}
	}
}

fn restart_game_timer(
	time: Res<Time>,
	globals: Res<GameGlobals>,
	mut state: ResMut<State<GameState>>,
) {
	if time.time_since_startup() > (globals.time_stopped + globals.time_until_restart) {
		let _ = state.overwrite_set(GameState::Playing);
	}
}

/// updates score when player is there
fn update_score(time: Res<Time>, mut globals: ResMut<GameGlobals>) {
	globals.score =
		((time.time_since_startup() - globals.time_started).as_secs() as u32) * globals.level;
}

/// remove all entities that are not a camera
fn teardown(mut commands: Commands, entities: Query<Entity, Without<Camera>>) {
	for entity in entities.iter() {
		commands.entity(entity).despawn_recursive();
	}
}

#[derive(Serialize, Deserialize, Debug)]
struct LeaderboardResponse {
	scores: Vec<LeaderboardScore>,
}

#[derive(Serialize, Deserialize, Debug)]
struct LeaderboardJSON {
	response: LeaderboardResponse,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LeaderboardScore {
	pub score: String,
	pub sort: String,
	pub guest: String,
}

use wasm_bindgen::{prelude::*, JsCast};

const API_ROOT: &'static str = "https://api.gamejolt.com/api/game/v1_2";
const PRIVATE_KEY: &'static str = "868930350536b2437a2cd5fb503ca7fc";
const GAME_ID: &'static str = "697047";
const TABLE_ID: &'static str = "705726";

struct Leaderboard;

impl Leaderboard {
	pub async fn leaderboard() -> Result<LeaderboardResponse, JsValue> {
		let res = Self::fetch_api("/scores", Some(format!("table_id={}", TABLE_ID))).await?;
		let json = JsFuture::from(res.json()?).await?;
		let leaderboard: LeaderboardJSON = json.into_serde().unwrap();
		Ok(leaderboard.response)
	}

	pub async fn add_score(score: u32, user: &str) -> Result<(), JsValue> {
		let _ = web_sys::window()
			.unwrap()
			.alert_with_message(format!("You died... well it's unfair because the boss never dies!!!.\n\nYou scored {} Points under the name {}\n\nCheck out the leaderboard at https://gamejolt.com/games/bevy-jam-1-submission/697047/scores/705726/best", score, user).as_str());
		Self::fetch_api(
			"/scores/add",
			Some(format!(
				"score={} Points&sort={}&guest={}&table_id={}",
				score, score, user, TABLE_ID
			)),
		)
		.await?;
		Ok(())
	}

	async fn fetch_api(path: &str, params: Option<String>) -> Result<Response, JsValue> {
		let window = web_sys::window().unwrap();
		let params = if let Some(params) = params {
			format!("?game_id={}&{}", GAME_ID, params)
		} else {
			format!("?game_id={}", GAME_ID)
		};
		let url = format!("{}{}{}", API_ROOT, path, params);
		let signature = format!("{:x}", &md5::compute(format!("{}{}", url, PRIVATE_KEY)));
		let url = format!("{}&signature={}", url, signature);

		JsFuture::from(window.fetch_with_str(&url))
			.await?
			.dyn_into::<Response>()
	}
}
