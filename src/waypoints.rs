use std::{
	f32::INFINITY,
	sync::{Arc, Mutex},
};

use bevy::{math::Vec3Swizzles, prelude::*, utils::HashMap};
use bevy_inspector_egui::{Inspectable, InspectorPlugin, RegisterInspectable};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use bevy_rapier2d::prelude::*;

use crate::game::{GameGlobals, GameState};

pub struct WaypointsPlugin;

impl Plugin for WaypointsPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(WaypointGlobals {
			weights_cell: Arc::new(Mutex::new(HashMap::default())),
		})
		.add_event::<CreatePathEvent>()
		.insert_resource(WaypointsParams::default())
		//.register_inspectable::<Waypoint>()
		//.add_plugin(InspectorPlugin::<WaypointsParams>::new())
		.add_system_set(SystemSet::on_enter(GameState::Playing).with_system(spawn_waypoints))
		.add_system_to_stage(CoreStage::PostUpdate, construct_edges)
		.add_system_set(
			SystemSet::on_update(GameState::Playing)
				.with_system(create_path_event_listener.before("set_next_waypoint"))
				.with_system(set_next_waypoint.label("set_next_waypoint")),
		)
		.add_plugin(DebugLinesPlugin::default())
		.add_system_to_stage(CoreStage::Last, debug_render);
	}
}

#[derive(Inspectable)]
struct WaypointsParams {
	gap: Vec2,
	debug_size: f32,
	scale: Vec2,
	offset: Vec2,
}

impl Default for WaypointsParams {
	fn default() -> Self {
		WaypointsParams {
			gap: Vec2::new(100.0, 125.0),
			scale: Vec2::new(1.0, 1.75),
			offset: Vec2::new(0.0, 50.0),
			debug_size: 20.0,
		}
	}
}

struct WaypointGlobals {
	weights_cell: Arc<Mutex<HashMap<Entity, f32>>>,
}

#[derive(Component, Debug, Clone, Inspectable)]
pub struct Waypoint(pub Vec2, Vec<WaypointEdge>);

#[derive(Component)]
pub struct NextWaypoint(pub Waypoint);

#[derive(Component, Debug, Clone, Copy, Inspectable, Default)]
struct WaypointEdge(Option<Entity>, f32);

#[derive(Component)]
pub struct WaypointPath(Vec<(Waypoint, Entity)>);

impl Waypoint {
	pub fn find_nearest<'a>(
		waypoints: impl Iterator<Item = (&'a Waypoint, Entity)>,
		pos: &Vec2,
	) -> Option<(&'a Self, Entity)> {
		let mut nearest: Option<(Entity, &Waypoint, f32)> = None;
		for (waypoint, entity) in waypoints {
			let dist = pos.distance(waypoint.0);
			if let Some((_, _, n_dist)) = nearest {
				if dist < n_dist {
					nearest = Some((entity, waypoint, dist));
				}
			} else {
				nearest = Some((entity, waypoint, dist));
			}
		}
		if let Some((entity, wp, _)) = nearest {
			Some((wp, entity))
		} else {
			None
		}
	}
	pub fn find_nearest_owned<'a>(
		waypoints: impl Iterator<Item = &'a (Waypoint, Entity)>,
		pos: &'a bevy::prelude::Vec2,
	) -> Option<(&Self, Entity)> {
		let mut nearest: Option<(Entity, &Waypoint, f32)> = None;
		for (waypoint, entity) in waypoints {
			let dist = pos.distance(waypoint.0);
			if let Some((_, _, n_dist)) = nearest {
				if dist < n_dist {
					nearest = Some((*entity, waypoint, dist));
				}
			} else {
				nearest = Some((*entity, waypoint, dist));
			}
		}
		if let Some((entity, wp, _)) = nearest {
			Some((wp, entity))
		} else {
			None
		}
	}
}

fn spawn_waypoints(
	mut commands: Commands,
	window: Res<WindowDescriptor>,
	params: Res<WaypointsParams>,
) {
	let x_max = (window.width / params.gap.x / 2.0) as i32;
	let y_max = (window.height / params.gap.y / 2.0) as i32;
	for y_i in -y_max..y_max {
		for x_i in -x_max..x_max {
			let pos =
				Vec2::new(x_i as f32 * params.gap.x, y_i as f32 * params.gap.y) + params.offset;
			commands
				.spawn()
				.insert(Waypoint(pos * params.scale, vec![]));
		}
	}
}

fn construct_edges(
	mut commands: Commands,
	mut query: Query<(Entity, &mut Waypoint)>,
	query_pipeline: Res<QueryPipeline>,
	collider_query: QueryPipelineColliderComponentsQuery,
	rapier_params: Res<RapierConfiguration>,
	state: Res<State<GameState>>,
	game_globals: Res<GameGlobals>,
	time: Res<Time>,
) {
	if *state.current() != GameState::Playing {
		return;
	}

	if (time.time_since_startup() - game_globals.time_started).as_secs() < 3 {
		return;
	}

	{
		if query.iter().filter(|(_, wp)| wp.1.len() == 0).count() == 0 {
			return;
		}
	}

	let mut iter = query.iter_combinations_mut();
	let collider_set = QueryPipelineColliderComponentsSet(&collider_query);

	while let Some([(e1, mut wp1), (e2, mut wp2)]) = iter.fetch_next() {
		let pos = wp1.0 / rapier_params.scale;
		let dir = (wp2.0 - wp1.0) / rapier_params.scale;
		let ray = Ray::new(pos.into(), dir.into());

		if let None = query_pipeline.cast_ray(
			&collider_set,
			&ray,
			1.0,
			true,
			InteractionGroups::all(),
			None,
		) {
			let dist = wp2.0.distance(wp1.0);
			wp1.1.push(WaypointEdge(Some(e2), dist));
			wp2.1.push(WaypointEdge(Some(e1), dist));
		}
	}

	// check and remove orphaned nodes
	let orphans = query.iter().filter(|(_, wp)| wp.1.len() == 0);

	for (entity, wp) in orphans {
		info!("Orphaned node removed at {}", wp.0);
		commands.entity(entity).despawn();
	}
}

pub struct CreatePathEvent(pub Vec2, pub Vec2, pub Entity);

/// This system is responsible for generating paths between waypoints. It reacts to CreatePathEvent events
/// fired by entities (mainly enemies) by attaching a WaypointPath component
/// to the Entity (ideally an enemy) that send the event.
fn create_path_event_listener(
	mut commands: Commands,
	mut event_reader: EventReader<CreatePathEvent>,
	q_waypoints: Query<(&Waypoint, Entity)>,
	mut globals: ResMut<WaypointGlobals>,
) {
	for CreatePathEvent(src, dst, sender_entity) in event_reader.iter() {
		let wp_src = Waypoint::find_nearest(q_waypoints.iter(), src);
		let wp_dst = Waypoint::find_nearest(q_waypoints.iter(), dst);

		if wp_src.is_none() || wp_dst.is_none() {
			info!(
				"Failed to create path between {:?} and {:?}",
				wp_src, wp_dst
			);
			return;
		}

		if wp_src.unwrap().0 .1.len() == 0 {
			return;
		}

		let mut visited = vec![];
		let total_wp = q_waypoints.iter().len();

		// set starting node to 0 and other nodes to infinity
		// loop over nodes
		// if node not visited: 2 options:
		// 1. totat dist < weight of node: update nodes weight to total dist
		// 2. leave weight
		// move to another unvisited node until all are visited

		globals.weights_cell = Arc::new(Mutex::new(HashMap::default()));

		let (mut our_wp, mut our_entity) = wp_src.unwrap();
		// mark starting node as visited and set weight to 0
		visited.push(our_entity);
		{
			let mut weights = globals.weights_cell.lock().unwrap();
			weights.insert(our_entity, 0.0);
		}

		// search until all nodes visited
		while visited.len() < total_wp {
			let our_edges = &our_wp.1[..];

			// loop over all unvisited nodes connectde to our starting node
			for WaypointEdge(entity, dist) in our_edges {
				let entity = entity.unwrap();
				if visited.contains(&entity) {
					continue;
				} else {
					let mut weights = globals.weights_cell.lock().unwrap();
					let our_weight = weights.get(&our_entity).unwrap();
					let total_dist = dist + *our_weight;
					let weight = weights.entry(entity).or_insert(INFINITY);

					// set to total distance when it's smaller than the node's weight
					if total_dist < *weight {
						*weight = total_dist;
					}
				}
			}

			// mark our node as visited before moving on
			visited.push(our_entity);

			// move on to next node with smallest weight that isn't visted
			let weights = globals.weights_cell.lock().unwrap();
			if let Some((next_wp, next_entity)) = q_waypoints
				.iter()
				.filter(|(_, e)| !(&visited[..]).contains(e))
				.min_by(|(_, e1), (_, e2)| {
					weights
						.get(e1)
						.unwrap_or(&INFINITY)
						.partial_cmp(weights.get(e2).unwrap_or(&INFINITY))
						.unwrap()
				}) {
				our_wp = next_wp;
				our_entity = next_entity;
			}
		}

		// start from end waypoint and make our way down

		let (mut our_wp, mut our_entity) = wp_dst.unwrap();
		let (_, src_entity) = wp_src.unwrap();

		let mut path = vec![(our_wp.clone(), our_entity)];

		while our_entity != src_entity {
			let n_wp = our_wp
				.1
				.iter()
				.min_by(|WaypointEdge(e1, _), WaypointEdge(e2, _)| {
					let weights = globals.weights_cell.lock().unwrap();
					weights
						.get(&e1.unwrap())
						.partial_cmp(&weights.get(&e2.unwrap()))
						.unwrap()
				})
				.unwrap();

			if let WaypointEdge(Some(n_entity), _) = n_wp {
				if let Ok((n_wp, _)) = q_waypoints.get(*n_entity) {
					path.push((n_wp.clone().into(), *n_entity));
					our_entity = *n_entity;
					our_wp = n_wp;
				}
			}
		}

		if path.len() > 0 {
			commands.entity(*sender_entity).insert(WaypointPath(path));
		}
	}
}

fn set_next_waypoint(
	mut commands: Commands,
	q_path: Query<(Entity, &Transform, &WaypointPath)>,
	mut q_next_wp: Query<&mut NextWaypoint>,
	_time: Res<Time>,
) {
	for (entity, transform, path) in q_path.iter() {
		let pos = transform.translation.xy();
		let (nearest_wp, nearest_id) = Waypoint::find_nearest_owned(path.0.iter(), &pos).unwrap();

		// if we arrive at end, stop
		let (last_wp, _) = path.0.iter().next().unwrap();
		if nearest_wp.0 == last_wp.0 {
			continue;
		}

		for (i, &(_, path_id)) in path.0.iter().enumerate() {
			if path_id == nearest_id {
				// find the waypoint one index ahead from the nearest
				let (next_wp, _) = path.0.iter().take(i).last().unwrap();

				// set the found index
				if let Ok(mut wp) = q_next_wp.get_mut(entity) {
					wp.0 = next_wp.clone();
				} else {
					commands
						.entity(entity)
						.insert(NextWaypoint(next_wp.clone()));
				}
			}
		}
	}
}

fn debug_render(
	mut commands: Commands,
	q_waypoints: Query<(Entity, &Waypoint)>,
	mut q_has_sprite: Query<&mut Sprite>,
	mut lines: ResMut<DebugLines>,
	globals: Res<WaypointGlobals>,
	params: Res<WaypointsParams>,
	state: Res<State<GameState>>,
) {
	if *state.current() != GameState::Playing {
		return;
	}

	let max_weight = Arc::clone(&globals.weights_cell)
		.lock()
		.unwrap()
		.clone()
		.into_values()
		.reduce(f32::max)
		.unwrap_or(INFINITY);

	for (entity, Waypoint(pos, edges)) in q_waypoints.iter() {
		let mut color = Color::PINK;
		if let Some(weight) = globals.weights_cell.lock().unwrap().get(&entity) {
			color = Color::rgb(0.0, 0.0, weight / max_weight);
		}

		if let Ok(mut sprite) = q_has_sprite.get_mut(entity) {
			sprite.color = color;
			sprite.custom_size = Some(Vec2::ONE * color.b() * params.debug_size);
		} else {
			commands.entity(entity).insert_bundle(SpriteBundle {
				sprite: Sprite {
					color,
					custom_size: Some(Vec2::new(5.0, 5.0)),
					..Default::default()
				},
				transform: Transform::from_xyz(pos.x, pos.y, 0.0),
				..Default::default()
			});
		}

		let max_path = q_waypoints
			.iter()
			.map(|(_, Waypoint(_, edges))| edges.len())
			.sum();
		if lines.positions.len() >= max_path {
			continue;
		}

		for edge in edges.iter() {
			if let Ok((_, Waypoint(edge_pos, _))) = q_waypoints.get(edge.0.unwrap()) {
				lines.line_colored(
					Vec3::new(pos.x, pos.y, 0.0),
					Vec3::new(edge_pos.x, edge_pos.y, 0.0),
					INFINITY,
					Color::PINK,
				);
			}
		}
	}
}
