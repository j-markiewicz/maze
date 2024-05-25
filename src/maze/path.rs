use std::time::Duration;

use bevy::prelude::*;

use super::player::Player;
use crate::util::{Rand, TurboRand};

const MOVEMENT_SPEED: f32 = 30.0;
const LIGHT_INITIAL_INTENSITY: f32 = 50_000_000.0;

#[derive(Debug, Component)]
pub struct Path;

#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn initialize(mut commands: Commands, rng: Res<Rand>) {
	commands
		.spawn((Path, SpatialBundle {
			transform: Transform {
				translation: Vec3 {
					z: 5.0,
					..default()
				},
				..default()
			},
			..default()
		}))
		.with_children(|builder| {
			builder.spawn((
				PointLightBundle {
					point_light: PointLight {
						color: Color::hsl(
							rng.f32_normalized().mul_add(5.0, 347.7),
							rng.f32_normalized().mul_add(0.1, 0.83),
							rng.f32_normalized().mul_add(0.1, 0.47),
						),
						intensity: LIGHT_INITIAL_INTENSITY,
						range: 1000.0,
						shadows_enabled: true,
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

#[derive(Component, Deref, DerefMut)]
pub struct PathFlickerTimer(Timer);

#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn light_flicker(
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
			light.intensity *= 10000.0 / trans.translation().distance_squared(player);
			timer.set_duration(Duration::from_secs_f64((*rng).f64() / 5.0));
		}
	}
}
