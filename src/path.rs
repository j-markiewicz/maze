use std::time::Duration;

use bevy::prelude::*;

use super::player::Player;
use crate::{
	maze::{tile_position, Paths, TilePos, MAZE_SIZE, TILE_SCALE, TILE_SIZE},
	util::{Rand, TurboRand},
};

const MOVEMENT_SPEED: f32 = 30.0;
const LIGHT_INITIAL_INTENSITY: f32 = 500_000_000.0;

#[derive(Debug, Component)]
pub struct Path;

#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn spawn(commands: &mut Commands, rng: &Rand, paths: &Paths) {
	let mut current = paths.0.search(&TilePos {
		x: MAZE_SIZE.x / 2,
		y: MAZE_SIZE.y / 2,
	});

	while let Some(pos) = current {
		let idx = paths.0.get(pos).unwrap().index();
		let Vec2 { x, y } = tile_position(idx);
		current = paths.0.parent(pos);

		commands
			.spawn((Path, SpatialBundle {
				transform: Transform {
					translation: Vec3 { z: 5.0, x, y },
					..default()
				},
				..default()
			}))
			.with_children(|builder| {
				builder.spawn((
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
							translation: Vec3 {
								x: 0.0,
								y: 0.0,
								z: 0.5,
							},
							..default()
						},
						..default()
					},
					PathFlickerTimer(Timer::new(Duration::ZERO, TimerMode::Repeating)),
				));
			});
	}
}

#[derive(Component, Deref, DerefMut)]
pub struct PathFlickerTimer(Timer);

#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn flicker(
	time: Res<Time>,
	rng: Res<Rand>,
	player: Query<&GlobalTransform, (With<Player>, Without<Path>)>,
	mut query: Query<(&mut PointLight, &mut PathFlickerTimer, &GlobalTransform)>,
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

// #[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
// pub fn movement(
// 	time: Res<Time>,
// 	paths: Res<PathTree>,
// 	mut query: Query<&mut Transform, With<Path>>,
// ) {
// 	let distance = MOVEMENT_SPEED * time.delta_seconds();

// 	for mut trans in &mut query {
// 		trans.translation.y += distance * input.up;
// 		trans.translation.x += distance * input.right;
// 	}
// }