use std::{
	array,
	f32::consts::PI,
	fmt::{Debug, Formatter, Result as FmtResult},
	iter,
	ops::Neg,
};

use bevy::{
	prelude::*,
	render::render_resource::{
		Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
	},
	window::PrimaryWindow,
};
use image::{imageops, load_from_memory, RgbaImage};

use self::Direction::{Bottom, Left, Right, Top};
use super::algorithms::{gen_maze, MazeParams};
use crate::{
	algorithms::{gen_rooms, solve_maze, Tree},
	path::{self, Path},
	util::{Rand, TurboRand},
};

pub const MAZE_SIZE: UVec2 = UVec2::splat(128);
pub const MIN_MAZE_SIZE: u16 = 3;
pub const MAX_MAZE_SIZE: u16 = 100;

pub const TILE_SIZE: Vec2 = Vec2::new(32.0, 32.0);
pub const TILE_SCALE: f32 = 5.0;
pub const WALL_THICKNESS: f32 = 4.0;

pub const SUBTILE_SIZE: Vec2 = Vec2::new(16.0, 16.0);
pub const SUBTILE_SCALE: f32 = 2.0 / 5.0;

#[derive(Resource)]
pub struct Maze {
	pub tiles: Box<[Tile]>,
	textures: Box<[Handle<StandardMaterial>; 256]>,
	wall_mesh: Handle<Mesh>,
	floor_mesh: Handle<Mesh>,
	wall_material: Handle<StandardMaterial>,
}

impl Maze {
	/// Create a new `Maze`
	///
	/// # Panic
	/// Panics if the maze is not `MAZE_SIZE` tiles large
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		maze: impl Into<Box<[Tile]>>,
		params: MazeParams,
		textures: Box<[Handle<StandardMaterial>; 256]>,
		wall_mesh: Handle<Mesh>,
		floor_mesh: Handle<Mesh>,
		wall_material: Handle<StandardMaterial>,
		roof_mesh: Handle<Mesh>,
		roof_material: Handle<StandardMaterial>,
		commands: &mut Commands,
	) -> Self {
		let tiles = maze.into();

		assert_eq!(
			MAZE_SIZE.x * MAZE_SIZE.y,
			u32::try_from(tiles.len()).expect("maze is too large"),
			"the maze's size is incorrect"
		);

		let roof_size = UVec2::new(params.width() + 1, params.height() + 1);

		#[allow(clippy::cast_precision_loss)]
		commands.spawn((Roof, PbrBundle {
			mesh: roof_mesh,
			material: roof_material,
			transform: Transform {
				translation: Vec3 {
					x: if params.width % 2 == 0 {
						TILE_SIZE.x / 2.0 * TILE_SCALE
					} else {
						0.0
					},
					y: if params.height % 2 == 0 {
						TILE_SIZE.y / 2.0 * TILE_SCALE
					} else {
						0.0
					},
					z: 10.0,
				},
				scale: Vec3::new(roof_size.x as f32, roof_size.y as f32, 1.0),
				..default()
			},
			..default()
		}));

		Self {
			tiles,
			textures,
			wall_mesh,
			floor_mesh,
			wall_material,
		}
	}

	/// Get the tile at the given position
	///
	/// # Panic
	/// Panics if `x` is not less than the maze's width or `y` is not less than
	/// the maze's height
	pub fn get(&self, TilePos { x, y }: TilePos) -> Tile {
		assert!(x < MAZE_SIZE.x, "x must be less than the maze's width");
		assert!(y < MAZE_SIZE.y, "y must be less than the maze's height");

		self.tiles[usize::try_from(y * MAZE_SIZE.x + x).unwrap()]
	}

	/// Spawn the tile at `(x, y)` at the given location
	#[allow(clippy::too_many_arguments)]
	pub fn spawn_tile(&self, x: u32, y: u32, loc: Vec2, commands: &mut Commands) {
		let tile = self.get(TilePos { x, y });

		commands
			.spawn((tile, TilePos { x, y }, PbrBundle {
				mesh: self.floor_mesh.clone(),
				material: self.textures[tile.0 as usize].clone(),
				transform: Transform {
					translation: Vec3 {
						x: loc.x,
						y: loc.y,
						..default()
					},
					scale: Vec3::splat(TILE_SCALE),
					..default()
				},
				..default()
			}))
			.with_children(|builder| {
				if !(tile.is_grass()) {
					self.spawn_tile_walls(builder, tile);
				}
			});
	}

	fn spawn_tile_walls(&self, builder: &mut ChildBuilder, tile: Tile) {
		if tile.is_closed(Top) {
			builder.spawn(PbrBundle {
				mesh: self.wall_mesh.clone(),
				material: self.wall_material.clone(),
				transform: Transform {
					translation: Vec3 {
						x: 0.0,
						y: TILE_SIZE.y / 2.0,
						z: 0.0,
					},
					..default()
				},
				..default()
			});
		}

		if tile.is_closed(Bottom) {
			builder.spawn(PbrBundle {
				mesh: self.wall_mesh.clone(),
				material: self.wall_material.clone(),
				transform: Transform {
					translation: Vec3 {
						x: 0.0,
						y: -TILE_SIZE.y / 2.0,
						z: 0.0,
					},
					..default()
				},
				..default()
			});
		}

		if tile.is_closed(Right) {
			builder.spawn(PbrBundle {
				mesh: self.wall_mesh.clone(),
				material: self.wall_material.clone(),
				transform: Transform {
					translation: Vec3 {
						x: TILE_SIZE.x / 2.0,
						y: 0.0,
						z: 0.0,
					},
					rotation: Quat::from_rotation_z(PI / 2.0),
					..default()
				},
				..default()
			});
		}

		if tile.is_closed(Left) {
			builder.spawn(PbrBundle {
				mesh: self.wall_mesh.clone(),
				material: self.wall_material.clone(),
				transform: Transform {
					translation: Vec3 {
						x: -TILE_SIZE.x / 2.0,
						y: 0.0,
						z: 0.0,
					},
					rotation: Quat::from_rotation_z(PI / 2.0),
					..default()
				},
				..default()
			});
		}
	}
}

impl Debug for Maze {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("Maze").finish_non_exhaustive()
	}
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Tile(pub u8);

impl Tile {
	/// Fully closed stone tile
	pub const CLOSED: Self = Self(0b1111_1111);
	/// Fully open stone tile
	pub const OPEN: Self = Self(0);

	pub fn grass(rng: &Rand) -> Self {
		Self(rng.u8(0..0xf) << 4 | 0b1111)
	}

	/// Open the given `side` of this Tile
	pub fn open(&mut self, side: Direction) -> &mut Self {
		match side {
			Direction::Top => self.0 &= 0b1111_0111,
			Direction::Right => self.0 &= 0b1111_1011,
			Direction::Bottom => self.0 &= 0b1111_1101,
			Direction::Left => self.0 &= 0b1111_1110,
		}

		self
	}

	/// Whether the given `side` of this Tile is open
	pub const fn is_open(self, side: Direction) -> bool {
		!self.is_grass()
			&& match side {
				Direction::Top => self.0 & 0b1000 == 0,
				Direction::Right => self.0 & 0b0100 == 0,
				Direction::Bottom => self.0 & 0b0010 == 0,
				Direction::Left => self.0 & 0b0001 == 0,
			}
	}

	/// Whether the given `side` of this Tile is closed
	pub const fn is_closed(self, side: Direction) -> bool {
		!self.is_open(side)
	}

	/// Whether this tile is grass
	pub const fn is_grass(self) -> bool {
		self.0 & 0b1111 == 0b1111 && self.0 != Self::CLOSED.0
	}
}

impl Default for Tile {
	fn default() -> Self {
		Self::CLOSED
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
	Top,
	Right,
	Bottom,
	Left,
}

impl Neg for Direction {
	type Output = Self;

	fn neg(self) -> Self::Output {
		match self {
			Self::Top => Self::Bottom,
			Self::Right => Self::Left,
			Self::Bottom => Self::Top,
			Self::Left => Self::Right,
		}
	}
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Roof;

#[derive(Debug, Clone, Resource)]
pub struct Paths(pub Tree<TilePos>);

#[derive(Debug, Clone, Copy, Event)]
pub struct RegenerateMaze;

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn regenerate(
	mut commands: Commands,
	tiles: Query<Entity, (With<Tile>, Without<Path>)>,
	indicators: Query<Entity, (With<Path>, Without<Tile>)>,
	mut maze: ResMut<Maze>,
	params: Res<MazeParams>,
	rng: Res<Rand>,
	mut events: EventReader<RegenerateMaze>,
	roof: Query<(Entity, &Handle<Mesh>, &Handle<StandardMaterial>), With<Roof>>,
	mut paths: ResMut<Paths>,
) {
	if !events.is_empty() {
		events.clear();

		let mut new_tiles = prepare_maze(&rng, *params);
		let start = gen_maze(&mut new_tiles, &rng, *params);
		gen_rooms(&mut new_tiles, &rng, *params);
		adjust_maze_textures(&mut new_tiles, *params);

		maze.tiles = new_tiles.into();
		info!("maze exit at {start:?}");
		paths.0 = solve_maze(&maze, start, *params);

		let (roof, roof_mesh, roof_material) = roof.single();

		let roof_size = UVec2::new(params.width() + 1, params.height() + 1);

		#[allow(clippy::cast_precision_loss)]
		commands.spawn((Roof, PbrBundle {
			mesh: roof_mesh.clone(),
			material: roof_material.clone(),
			transform: Transform {
				translation: Vec3 {
					x: if params.width % 2 == 0 {
						TILE_SIZE.x / 2.0 * TILE_SCALE
					} else {
						0.0
					},
					y: if params.height % 2 == 0 {
						TILE_SIZE.y / 2.0 * TILE_SCALE
					} else {
						0.0
					},
					z: 10.0,
				},
				scale: Vec3::new(roof_size.x as f32, roof_size.y as f32, 1.0),
				..default()
			},
			..default()
		}));

		commands.entity(roof).despawn_recursive();

		for tile in &tiles {
			commands.entity(tile).despawn_recursive();
		}

		for indicator in &indicators {
			commands.entity(indicator).despawn_recursive();
		}

		path::spawn_initial(&mut commands, &rng, &paths);
	}
}

#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
fn gen_tile_textures(
	wall: &[&[u8]],
	floor: &[&[u8]],
	grass: &[&[u8]],
	images: &mut Assets<Image>,
	rng: &Rand,
) -> [Handle<Image>; 256] {
	let mut res = array::from_fn::<_, 256, _>(|_| None);

	let wall = wall
		.iter()
		.map(|data| load_from_memory(data).expect("invalid image").into_rgba8())
		.collect::<Vec<_>>();

	let grass = grass
		.iter()
		.map(|data| load_from_memory(data).expect("invalid image").into_rgba8())
		.collect::<Vec<_>>();

	let floor = floor
		.iter()
		.map(|data| load_from_memory(data).expect("invalid image").into_rgba8())
		.collect::<Vec<_>>();

	for bits in 0u8..=255u8 {
		let tile = Tile(if bits & 0b1111 == 0b1111 {
			bits
		} else {
			bits & 0b1111
		});

		let is_edge = |sx, sy| match (sx, sy) {
			(1..=3, 0) => tile.is_closed(Top),
			(4, 1..=3) => tile.is_closed(Right),
			(1..=3, 4) => tile.is_closed(Bottom),
			(0, 1..=3) => tile.is_closed(Left),
			(0, 0) => tile.is_closed(Top) || tile.is_closed(Left) || (bits & 0b1000_0000 != 0),
			(4, 0) => tile.is_closed(Top) || tile.is_closed(Right) || (bits & 0b0100_0000 != 0),
			(0, 4) => tile.is_closed(Bottom) || tile.is_closed(Left) || (bits & 0b0010_0000 != 0),
			(4, 4) => tile.is_closed(Bottom) || tile.is_closed(Right) || (bits & 0b0001_0000 != 0),
			_ => false,
		};

		let is_fully_closed = tile.is_closed(Top)
			&& tile.is_closed(Right)
			&& tile.is_closed(Bottom)
			&& tile.is_closed(Left);

		let mut image = RgbaImage::from_raw(5 * 16, 5 * 16, vec![0; 4 * 5 * 16 * 5 * 16]).unwrap();

		for sy in 0..5 {
			for sx in 0..5 {
				let subimage = if is_fully_closed && bits != 0xff {
					rng.sample(&grass).expect("there are no grass images")
				} else if is_edge(sx, sy) || bits == 0xff {
					rng.sample(&wall).expect("there are no wall images")
				} else {
					rng.sample(&floor).expect("there are no floor images")
				};

				imageops::overlay(&mut image, subimage, sx * 16, sy * 16);
			}
		}

		let handle = images.add(Image {
			data: image.into_vec(),
			texture_descriptor: TextureDescriptor {
				label: None,
				size: Extent3d {
					width: 5 * 16,
					height: 5 * 16,
					..default()
				},
				dimension: TextureDimension::D2,
				format: TextureFormat::Rgba8UnormSrgb,
				mip_level_count: 1,
				sample_count: 1,
				usage: TextureUsages::TEXTURE_BINDING
					| TextureUsages::COPY_DST
					| TextureUsages::RENDER_ATTACHMENT,
				view_formats: &[],
			},
			texture_view_descriptor: None,
			..default()
		});
		res[bits as usize] = Some(handle);
	}

	res.map(|o| o.expect("image creation failed"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component, Hash)]
pub struct TilePos {
	pub x: u32,
	pub y: u32,
}

impl TilePos {
	pub const fn index(self) -> u32 {
		self.y * MAZE_SIZE.x + self.x
	}
}

impl From<UVec2> for TilePos {
	fn from(UVec2 { x, y }: UVec2) -> Self {
		Self { x, y }
	}
}

impl From<TilePos> for UVec2 {
	fn from(TilePos { x, y }: TilePos) -> Self {
		Self { x, y }
	}
}

#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn initialize(
	mut commands: Commands,
	rng: Res<Rand>,
	params: Res<MazeParams>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	mut images: ResMut<Assets<Image>>,
) {
	let wall = [&include_bytes!("../assets/maze/cave-wall.png")[..]];
	let floor = [
		&include_bytes!("../assets/maze/cave-floor-1.png")[..],
		&include_bytes!("../assets/maze/cave-floor-2.png")[..],
	];
	let grass = [
		&include_bytes!("../assets/maze/grass-1.png")[..],
		&include_bytes!("../assets/maze/grass-2.png")[..],
		&include_bytes!("../assets/maze/grass-3.png")[..],
	];

	let floor_mesh = meshes.add(Rectangle::from_size(TILE_SIZE));
	let wall_mesh = meshes.add(Cuboid::new(
		SUBTILE_SIZE.x.mul_add(SUBTILE_SCALE, TILE_SIZE.x),
		SUBTILE_SIZE.y * SUBTILE_SCALE,
		25.0,
	));

	let wall_material = materials.add(StandardMaterial {
		base_color: Color::rgba(1.0, 1.0, 1.0, 1.0),
		emissive: Color::rgba(0.0, 0.0, 0.0, 0.0),
		reflectance: 1.0,
		unlit: true,
		fog_enabled: false,
		..default()
	});

	let roof_mesh = meshes.add(Rectangle::from_size(TILE_SIZE * TILE_SCALE));

	let roof_material = materials.add(StandardMaterial {
		base_color: Color::BLACK,
		reflectance: 0.0,
		unlit: true,
		fog_enabled: false,
		..default()
	});

	let mut maze = prepare_maze(&rng, *params);
	let exit = gen_maze(&mut maze, &rng, *params);
	gen_rooms(&mut maze, &rng, *params);
	adjust_maze_textures(&mut maze, *params);

	let textures = gen_tile_textures(&wall, &floor, &grass, &mut images, &rng).map(|h| {
		materials.add(StandardMaterial {
			base_color: Color::GRAY,
			base_color_texture: Some(h.clone()),
			reflectance: rng.f32().mul_add(0.1, 0.1),
			perceptual_roughness: rng.f32().mul_add(0.15, 0.85),
			emissive: Color::hsl(210.0, 0.3, 0.3).as_rgba() * 18.0,
			emissive_texture: Some(h),
			unlit: false,
			..default()
		})
	});

	let maze = Maze::new(
		maze,
		*params,
		Box::new(textures),
		wall_mesh,
		floor_mesh,
		wall_material,
		roof_mesh,
		roof_material,
		&mut commands,
	);

	commands.insert_resource(Paths(solve_maze(&maze, exit, *params)));
	commands.insert_resource(maze);
}

#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
fn prepare_maze(rng: &Rand, params: MazeParams) -> Vec<Tile> {
	let us = |u32: u32| -> usize { u32.try_into().unwrap() };
	let idx = |UVec2 { x, y }| usize::try_from(y * MAZE_SIZE.x + x).unwrap();

	let mut maze = iter::from_fn(|| Some(Tile::grass(rng)))
		.take(us(MAZE_SIZE.x) * us(MAZE_SIZE.y))
		.collect::<Vec<_>>();

	for x in params.margin_x()..params.margin_x() + params.width() {
		for y in params.margin_y()..params.margin_y() + params.height() {
			maze[idx(UVec2::new(x, y))] = Tile::CLOSED;
		}
	}

	maze
}

pub fn adjust_maze_textures(maze: &mut [Tile], params: MazeParams) {
	let idx = |UVec2 { x, y }| usize::try_from(y * MAZE_SIZE.x + x).unwrap();

	for x in params.margin_x() - 1..=params.margin_x() + params.width() {
		let pos = UVec2::new(x, params.margin_y() + params.height());
		maze[idx(pos)].open(Top);
		maze[idx(pos)].open(Left);
		maze[idx(pos)].open(Right);

		let pos = UVec2::new(x, params.margin_y() - 1);
		maze[idx(pos)].open(Bottom);
		maze[idx(pos)].open(Left);
		maze[idx(pos)].open(Right);
	}

	for y in params.margin_y() - 1..=params.margin_y() + params.height() {
		let pos = UVec2::new(params.margin_x() + params.width(), y);
		maze[idx(pos)].open(Top);
		maze[idx(pos)].open(Bottom);
		maze[idx(pos)].open(Right);

		let pos = UVec2::new(params.margin_x() - 1, y);
		maze[idx(pos)].open(Top);
		maze[idx(pos)].open(Bottom);
		maze[idx(pos)].open(Left);
	}

	for i in 0..maze.len() {
		maze[i].0 = tile_bits(i, maze);
	}

	for x in params.margin_x() - 1..=params.margin_x() + params.width() {
		let pos = UVec2::new(x, params.margin_y() + params.height());
		maze[idx(pos)].0 &= 0b0011_1111;

		let pos = UVec2::new(x, params.margin_y() - 1);
		maze[idx(pos)].0 &= 0b1100_1111;
	}

	for y in params.margin_y() - 1..=params.margin_y() + params.height() {
		let pos = UVec2::new(params.margin_x() + params.width(), y);
		maze[idx(pos)].0 &= 0b1010_1111;

		let pos = UVec2::new(params.margin_x() - 1, y);
		maze[idx(pos)].0 &= 0b0101_1111;
	}
}

pub fn tile_bits(i: usize, maze: &[Tile]) -> u8 {
	let tile = maze[i];
	if tile.is_grass() {
		return tile.0;
	}

	let maze_size = (
		usize::try_from(MAZE_SIZE.x).unwrap(),
		usize::try_from(MAZE_SIZE.y).unwrap(),
	);

	let tile_is_edge = !(maze_size.0..=(maze_size.1 - 1) * maze_size.0).contains(&i)
		|| i % maze_size.0 == 0
		|| i % maze_size.0 == maze_size.0 - 1;

	let mut res = tile.0 & 0b1111;

	if !tile_is_edge {
		if maze[i.saturating_sub(1)].is_closed(Top)
			|| maze[i.saturating_add(maze_size.0)].is_closed(Left)
		{
			res |= 0b1000_0000;
		}

		if maze[i.saturating_add(1)].is_closed(Top)
			|| maze[i.saturating_add(maze_size.0)].is_closed(Right)
		{
			res |= 0b0100_0000;
		}

		if maze[i.saturating_sub(1)].is_closed(Bottom)
			|| maze[i.saturating_sub(maze_size.0)].is_closed(Left)
		{
			res |= 0b0010_0000;
		}

		if maze[i.saturating_add(1)].is_closed(Bottom)
			|| maze[i.saturating_sub(maze_size.0)].is_closed(Right)
		{
			res |= 0b0001_0000;
		}
	}

	res
}

#[allow(clippy::cast_precision_loss)]
pub fn tile_position(i: u32) -> Vec2 {
	Vec2 {
		x: (i32::try_from(i % MAZE_SIZE.x).unwrap() - i32::try_from(MAZE_SIZE.x / 2).unwrap())
			as f32 * TILE_SCALE
			* TILE_SIZE.x,
		y: (i32::try_from(i / MAZE_SIZE.x).unwrap() - i32::try_from(MAZE_SIZE.y / 2).unwrap())
			as f32 * TILE_SCALE
			* TILE_SIZE.y,
	}
}

#[allow(
	clippy::cast_precision_loss,
	clippy::cast_possible_truncation,
	clippy::cast_sign_loss,
	clippy::cast_possible_wrap
)]
pub fn nearest_tile(pos: Vec2) -> TilePos {
	TilePos {
		x: (((pos.x.floor() + MAZE_SIZE.x as f32 / 2.0) / TILE_SCALE / TILE_SIZE.x).floor() as i32
			+ MAZE_SIZE.x as i32 / 2) as u32,
		y: (((pos.y.floor() + MAZE_SIZE.y as f32 / 2.0) / TILE_SCALE / TILE_SIZE.y).floor() as i32
			+ MAZE_SIZE.y as i32 / 2) as u32,
	}
}

#[allow(
	clippy::cast_possible_truncation,
	clippy::type_complexity,
	clippy::too_many_arguments
)]
#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn spawn_visible_tiles(
	mut commands: Commands,
	maze: Res<Maze>,
	tiles: Query<&TilePos, With<Tile>>,
	window: Query<&Window, (With<PrimaryWindow>, Without<Tile>, Without<Camera2d>)>,
	camera: Query<&Transform, (With<Camera2d>, Changed<Transform>, Without<Tile>)>,
) {
	let Ok(window) = window.get_single() else {
		return;
	};

	let Ok(camera) = camera.get_single() else {
		return;
	};

	let existing_tiles = tiles.iter().copied().collect::<Vec<_>>();

	let new_tiles = (0..maze.tiles.len())
		.filter(|&i| {
			let Vec2 { x, y } = tile_position(i as u32);
			let width = TILE_SIZE.x.mul_add(TILE_SCALE * 2.0, window.width());
			let height = TILE_SIZE.y.mul_add(TILE_SCALE * 2.0, window.height());
			let x_extent =
				(camera.translation.x - width / 2.0)..(camera.translation.x + width / 2.0);
			let y_extent =
				(camera.translation.y - height / 2.0)..(camera.translation.y + height / 2.0);
			x_extent.contains(&x) && y_extent.contains(&y)
		})
		.filter_map(|i| {
			let pos = TilePos {
				x: i as u32 % MAZE_SIZE.x,
				y: i as u32 / MAZE_SIZE.x,
			};

			(!existing_tiles.contains(&pos)).then_some((pos.x, pos.y, i))
		});

	for (x, y, i) in new_tiles {
		maze.spawn_tile(x, y, tile_position(i as _), &mut commands);
	}
}

#[allow(clippy::type_complexity)]
#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn despawn_invisible_tiles(
	mut commands: Commands,
	tiles: Query<(Entity, &Transform), With<Tile>>,
	window: Query<&Window, (With<PrimaryWindow>, Without<Tile>, Without<Camera2d>)>,
	camera: Query<&Transform, (With<Camera2d>, Changed<Transform>, Without<Tile>)>,
) {
	let Ok(window) = window.get_single() else {
		return;
	};

	let Ok(camera) = camera.get_single() else {
		return;
	};

	let mut old_tiles = tiles.iter().filter(|&(_, t)| {
		let Vec3 { x, y, .. } = t.translation;
		let width = TILE_SIZE.x.mul_add(TILE_SCALE * 3.0, window.width());
		let height = TILE_SIZE.y.mul_add(TILE_SCALE * 3.0, window.height());
		let x_extent = (camera.translation.x - width / 2.0)..(camera.translation.x + width / 2.0);
		let y_extent = (camera.translation.y - height / 2.0)..(camera.translation.y + height / 2.0);
		!x_extent.contains(&x) || !y_extent.contains(&y)
	});

	if let Some((e, _)) = old_tiles.next() {
		// This is very slow, so only do one per frame
		commands.entity(e).despawn_recursive();
	}
}
