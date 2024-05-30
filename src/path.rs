use std::{f32, time::Duration};

use bevy::prelude::*;

use super::player::Player;
use crate::{
	maze::{nearest_tile, tile_position, Paths, TilePos, MAZE_SIZE, TILE_SCALE, TILE_SIZE},
	util::{Rand, TurboRand},
};

const MOVEMENT_SPEED: f32 = 30.0;
const ROTATION_SPEED: f32 = 0.5;
const FADING_DURATION: f32 = 5.0;
const SPAWNING_TIME: f32 = 2.5;
const LIGHT_INITIAL_INTENSITY: f32 = 500_000_000.0;

#[derive(Debug, Component)]
pub struct Path;

#[derive(Debug, Clone, Copy, Component)]
pub struct MovementDirection(Vec2);

#[derive(Debug, Resource)]
pub struct PathSpawnTimer(Timer);

pub fn initialize(mut commands: Commands) {
	commands.insert_resource(PathSpawnTimer(Timer::from_seconds(
		SPAWNING_TIME,
		TimerMode::Repeating,
	)));
}

#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn spawn_initial(commands: &mut Commands, rng: &Rand, paths: &Paths) {
	let mut current = paths.0.search(&TilePos {
		x: MAZE_SIZE.x / 2,
		y: MAZE_SIZE.y / 2,
	});

	while let Some(pos) = current {
		let idx = paths.0.get(pos).unwrap().index();
		let Vec2 { mut x, mut y } = tile_position(idx);
		current = paths.0.parent(pos);

		x += rng.f32_normalized() * TILE_SIZE.x * TILE_SCALE / 4.0;
		y += rng.f32_normalized() * TILE_SIZE.y * TILE_SCALE / 4.0;

		commands.spawn((
			Path,
			MovementDirection(Vec2::NAN),
			PointLightBundle {
				point_light: PointLight {
					color: Color::hsl(
						rng.f32() * 360.0,
						rng.f32_normalized().mul_add(0.25, 0.75),
						rng.f32_normalized().mul_add(0.25, 0.5),
					),
					intensity: LIGHT_INITIAL_INTENSITY,
					range: TILE_SIZE.x * TILE_SCALE,
					shadows_enabled: false,
					..default()
				},
				transform: Transform {
					translation: Vec3 { z: 5.0, x, y },
					..default()
				},
				..default()
			},
			PathFlickerTimer(Timer::new(Duration::ZERO, TimerMode::Repeating)),
		));
	}
}

#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn spawn_more(
	mut commands: Commands,
	rng: Res<Rand>,
	player: Query<&GlobalTransform, With<Player>>,
	mut timer: ResMut<PathSpawnTimer>,
	time: Res<Time>,
) {
	timer.0.tick(time.delta());

	if timer.0.just_finished() {
		let Vec3 { x, y, .. } = player.single().translation();

		commands.spawn((
			Path,
			MovementDirection(Vec2::NAN),
			PointLightBundle {
				point_light: PointLight {
					color: Color::hsl(
						rng.f32() * 360.0,
						rng.f32_normalized().mul_add(0.25, 0.75),
						rng.f32_normalized().mul_add(0.25, 0.5),
					),
					intensity: LIGHT_INITIAL_INTENSITY,
					range: TILE_SIZE.x * TILE_SCALE,
					shadows_enabled: false,
					..default()
				},
				transform: Transform {
					translation: Vec3 { z: 5.0, x, y },
					..default()
				},
				..default()
			},
			PathFlickerTimer(Timer::new(Duration::ZERO, TimerMode::Repeating)),
		));
	}
}

#[derive(Component, Deref, DerefMut)]
pub struct PathFlickerTimer(Timer);

#[derive(Component, Deref, DerefMut)]
pub struct FadingOut(Timer);

#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn flicker(
	time: Res<Time>,
	rng: Res<Rand>,
	player: Query<&GlobalTransform, (With<Player>, Without<Path>)>,
	mut query: Query<
		(&mut PointLight, &mut PathFlickerTimer, &GlobalTransform),
		Without<FadingOut>,
	>,
) {
	let player = player.single().translation();

	for (mut light, mut timer, trans) in &mut query {
		timer.tick(time.delta());

		if timer.just_finished() {
			light.intensity = LIGHT_INITIAL_INTENSITY * ((*rng).f32() + 1.0) / 2.0;
			light.intensity *= f32::min(
				1.0,
				5000.0 / (trans.translation().distance_squared(player) + f32::EPSILON),
			);
			timer.set_duration(Duration::from_secs_f64((*rng).f64() / 5.0));
		}
	}
}

#[allow(clippy::type_complexity)]
#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn movement(
	time: Res<Time>,
	paths: Res<Paths>,
	mut query: Query<
		(Entity, &mut Transform, &mut MovementDirection),
		(With<Path>, Without<FadingOut>),
	>,
	mut commands: Commands,
) {
	let distance = MOVEMENT_SPEED * time.delta_seconds();
	let rotation = ROTATION_SPEED * time.delta_seconds();

	for (entity, mut trans, mut dir) in &mut query {
		let current_tile = nearest_tile(trans.translation.truncate());
		let Some(next_tile) = paths
			.0
			.search(&current_tile)
			.and_then(|t| paths.0.parent(t))
			.and_then(|t| paths.0.get(t))
			.copied()
		else {
			commands
				.entity(entity)
				.insert(FadingOut(Timer::from_seconds(
					FADING_DURATION,
					TimerMode::Once,
				)));
			continue;
		};

		let direction = tile_position(next_tile.index()) - trans.translation.truncate();
		let direction = direction.normalize();

		if dir.0.is_nan() {
			dir.0 = direction;
		}

		let angle = dir.0.angle_between(direction).clamp(-rotation, rotation);

		let direction = dir.0.rotate(Vec2::from_angle(angle));
		dir.0 = direction;

		trans.translation.x += distance * direction.x;
		trans.translation.y += distance * direction.y;
	}
}

#[allow(clippy::type_complexity)]
#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn fadeout(
	time: Res<Time>,
	rng: Res<Rand>,
	mut commands: Commands,
	mut query: Query<
		(
			Entity,
			&mut Transform,
			&mut PointLight,
			&mut PathFlickerTimer,
			&mut FadingOut,
			&mut MovementDirection,
		),
		With<Path>,
	>,
	player: Query<&GlobalTransform, (With<Player>, Without<Path>)>,
) {
	let distance = MOVEMENT_SPEED * time.delta_seconds();
	let rotation = ROTATION_SPEED * time.delta_seconds();
	let player = player.single().translation();

	for (entity, mut trans, mut light, mut timer, mut fade, mut dir) in &mut query {
		timer.tick(time.delta());
		fade.0.tick(time.delta());

		if timer.just_finished() {
			light.intensity = LIGHT_INITIAL_INTENSITY * ((*rng).f32() + 1.0) / 2.0;
			light.intensity *= f32::min(
				1.0,
				5000.0 / (trans.translation.distance_squared(player) + f32::EPSILON),
			);
			light.intensity *= fade.0.fraction_remaining();
		}

		if fade.0.just_finished() {
			commands.entity(entity).despawn_recursive();
		}

		let current_tile = nearest_tile(trans.translation.truncate());
		let outside = TilePos {
			y: current_tile.y + 1,
			..current_tile
		};

		let direction = tile_position(outside.index()) - trans.translation.truncate();
		let direction = direction.normalize();

		if dir.0.is_nan() {
			dir.0 = direction;
		}

		let angle = dir.0.angle_between(direction).clamp(-rotation, rotation);

		let direction = dir.0.rotate(Vec2::from_angle(angle));
		dir.0 = direction;

		trans.translation.x += distance * direction.x;
		trans.translation.y += distance * direction.y;
	}
}
