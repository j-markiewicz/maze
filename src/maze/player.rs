use std::time::Duration;

use bevy::prelude::*;

use super::{
	algorithms::{
		Direction::{Bottom, Left, Right, Top},
		Tile,
	},
	maze, PlayerInput,
};
use crate::util::{Rand, TurboRand};

const TILE_SIZE: Vec2 = Vec2::new(24.0, 32.0);
const TILE_AMOUNT_IDLE: usize = 10;
const TILE_AMOUNT_WALKING: usize = 10;
const TILE_SCALE: f32 = 2.0;
const TILE_FRAME_TIME_SECONDS: f32 = 0.1;

const MOVEMENT_SPEED: f32 = 150.0;

const LIGHT_INITIAL_INTENSITY: f32 = 50_000_000_000.0;

#[derive(Debug, Component)]
pub struct Player {
	idle_atlas: Handle<TextureAtlasLayout>,
	idle_texture: Handle<Image>,
	walking_atlas: Handle<TextureAtlasLayout>,
	walking_texture: Handle<Image>,
}

#[derive(Debug, Component)]
pub struct Movement {
	is_walking: bool,
	is_right: bool,
}

#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn initialize(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
	let idle_handle = asset_server.load("maze/player-idle.png");
	let idle_atlas = TextureAtlasLayout::from_grid(TILE_SIZE, 1, TILE_AMOUNT_IDLE, None, None);
	let idle_atlas_handle = texture_atlases.add(idle_atlas);

	let walking_handle = asset_server.load("maze/player-walking.png");
	let walking_atlas =
		TextureAtlasLayout::from_grid(TILE_SIZE, 1, TILE_AMOUNT_WALKING, None, None);
	let walking_atlas_handle = texture_atlases.add(walking_atlas);

	commands
		.spawn((
			Player {
				idle_atlas: idle_atlas_handle.clone(),
				idle_texture: idle_handle.clone(),
				walking_atlas: walking_atlas_handle,
				walking_texture: walking_handle,
			},
			Movement {
				is_right: true,
				is_walking: false,
			},
			SpriteSheetBundle {
				atlas: TextureAtlas {
					layout: idle_atlas_handle,
					..default()
				},
				texture: idle_handle,
				sprite: Sprite::default(),
				transform: Transform {
					translation: Vec3 {
						z: 10.0,
						..default()
					},
					scale: Vec3::splat(TILE_SCALE),
					..default()
				},
				..default()
			},
			AnimationTimer(Timer::from_seconds(
				TILE_FRAME_TIME_SECONDS,
				TimerMode::Repeating,
			)),
		))
		.with_children(|builder| {
			builder.spawn((
				PointLightBundle {
					point_light: PointLight {
						color: Color::ORANGE,
						intensity: LIGHT_INITIAL_INTENSITY,
						range: 1000.0,
						shadows_enabled: true,
						..default()
					},
					transform: Transform {
						translation: Vec3 {
							x: 0.0,
							y: 0.0,
							z: 0.0,
						},
						..default()
					},
					..default()
				},
				FlickerTimer(Timer::new(Duration::ZERO, TimerMode::Repeating)),
			));
		});
}

#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn movement(
	time: Res<Time>,
	input: Res<PlayerInput>,
	mut query: Query<(&mut Transform, &mut Movement), With<Player>>,
) {
	let distance = MOVEMENT_SPEED * time.delta_seconds();

	for (mut trans, mut movement) in &mut query {
		if input.right > 0.0 {
			movement.is_right = true;
		} else if input.right < 0.0 {
			movement.is_right = false;
		}

		movement.is_walking = input.is_moving();

		trans.translation.y += distance * input.up;
		trans.translation.x += distance * input.right;
	}
}

#[derive(Component, Deref, DerefMut)]
pub struct FlickerTimer(Timer);

#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn light_flicker(
	time: Res<Time>,
	rng: Res<Rand>,
	mut query: Query<(&mut PointLight, &mut FlickerTimer)>,
) {
	for (mut light, mut timer) in &mut query {
		timer.tick(time.delta());

		if timer.just_finished() {
			light.intensity = LIGHT_INITIAL_INTENSITY * ((*rng).f32() + 1.0) / 2.0;
			timer.set_duration(Duration::from_secs_f64((*rng).f64() / 5.0));
			light.shadows_enabled = !light.shadows_enabled;
		}
	}
}

#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
#[allow(clippy::too_many_lines)]
pub fn collision(
	mut player: Query<&mut Transform, With<Player>>,
	tiles: Query<(&Transform, &Tile), Without<Player>>,
) {
	let mut player = player.single_mut();

	let half_size = maze::TILE_SIZE / 2.0;
	let inner_half = half_size - maze::WALL_THICKNESS;
	let scaled_inner = inner_half * maze::TILE_SCALE;

	let player_edges = [
		player.translation.y + TILE_SIZE.y * TILE_SCALE / 2.0,
		player.translation.x + TILE_SIZE.x * TILE_SCALE / 2.0,
		player.translation.y - TILE_SIZE.y * TILE_SCALE / 2.0,
		player.translation.x - TILE_SIZE.x * TILE_SCALE / 2.0,
	];

	let mut nearby_tiles = tiles
		.iter()
		.filter(|(t, ..)| {
			let diff = (t.translation - player.translation).abs();
			diff.x < 1.5 * maze::TILE_SIZE.x * maze::TILE_SCALE
				&& diff.y < 1.5 * maze::TILE_SIZE.y * maze::TILE_SCALE
		})
		.map(|(trans, tile)| {
			if tile.is_grass() {
				(trans, Tile::OPEN)
			} else {
				(trans, *tile)
			}
		})
		.collect::<Vec<_>>();

	if nearby_tiles.len() < 9 {
		return;
	}

	#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
	nearby_tiles.sort_by_key(|(t, _)| (-t.translation.y as i32, t.translation.x as i32));
	let nearby_tiles = nearby_tiles;

	let current_tile = nearby_tiles[4];

	if current_tile.1.is_grass() {
		return;
	}

	let mut tile_edges = [
		current_tile.0.translation.y + scaled_inner.y,
		current_tile.0.translation.x + scaled_inner.x,
		current_tile.0.translation.y - scaled_inner.y,
		current_tile.0.translation.x - scaled_inner.x,
	];

	let mut is_above = player_edges[0] > tile_edges[0];
	let mut is_right = player_edges[1] > tile_edges[1];
	let mut is_below = player_edges[2] < tile_edges[2];
	let mut is_left = player_edges[3] < tile_edges[3];

	if current_tile.1.is_closed(Top) && is_above {
		player.translation.y -= player_edges[0] - tile_edges[0];
		tile_edges[0] = current_tile.0.translation.y + scaled_inner.y;
		is_above = false;
	}

	if current_tile.1.is_closed(Right) && is_right {
		player.translation.x -= player_edges[1] - tile_edges[1];
		tile_edges[1] = current_tile.0.translation.x + scaled_inner.x;
		is_right = false;
	}

	if current_tile.1.is_closed(Bottom) && is_below {
		player.translation.y -= player_edges[2] - tile_edges[2];
		tile_edges[2] = current_tile.0.translation.y - scaled_inner.y;
		is_below = false;
	}

	if current_tile.1.is_closed(Left) && is_left {
		player.translation.x -= player_edges[3] - tile_edges[3];
		tile_edges[3] = current_tile.0.translation.x - scaled_inner.x;
		is_left = false;
	}

	let player_tile_diff = (player.translation - current_tile.0.translation).abs();
	let coll_is_horizontal = player_tile_diff.y > player_tile_diff.x;

	if (nearby_tiles[3].1.is_closed(Top) || nearby_tiles[1].1.is_closed(Left))
		&& is_above
		&& is_left
	{
		if coll_is_horizontal {
			player.translation.x -= player_edges[3] - tile_edges[3];
		} else {
			player.translation.y -= player_edges[0] - tile_edges[0];
		}
	}

	if (nearby_tiles[3].1.is_closed(Bottom) || nearby_tiles[7].1.is_closed(Left))
		&& is_below
		&& is_left
	{
		if coll_is_horizontal {
			player.translation.x -= player_edges[3] - tile_edges[3];
		} else {
			player.translation.y -= player_edges[2] - tile_edges[2];
		}
	}

	if (nearby_tiles[5].1.is_closed(Top) || nearby_tiles[1].1.is_closed(Right))
		&& is_above
		&& is_right
	{
		if coll_is_horizontal {
			player.translation.x -= player_edges[1] - tile_edges[1];
		} else {
			player.translation.y -= player_edges[0] - tile_edges[0];
		}
	}

	if (nearby_tiles[5].1.is_closed(Bottom) || nearby_tiles[7].1.is_closed(Right))
		&& is_below
		&& is_right
	{
		if coll_is_horizontal {
			player.translation.x -= player_edges[1] - tile_edges[1];
		} else {
			player.translation.y -= player_edges[2] - tile_edges[2];
		}
	}
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(Timer);

#[allow(clippy::type_complexity)]
#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn animation(
	time: Res<Time>,
	mut query: Query<(
		&Movement,
		&Player,
		&mut AnimationTimer,
		&mut Sprite,
		&mut TextureAtlas,
		&mut Handle<Image>,
	)>,
) {
	for (movement, player, mut timer, mut sprite, mut atlas, mut texture) in &mut query {
		timer.tick(time.delta());
		if timer.just_finished() {
			atlas.index += 1;
		}

		if movement.is_walking {
			atlas.layout = player.walking_atlas.clone();
			*texture = player.walking_texture.clone();
			atlas.index %= TILE_AMOUNT_WALKING;
		} else {
			atlas.layout = player.idle_atlas.clone();
			*texture = player.idle_texture.clone();
			atlas.index %= TILE_AMOUNT_IDLE;
		}

		sprite.flip_x = !movement.is_right;
	}
}
