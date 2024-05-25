use std::{iter, ops::Neg};

#[cfg(feature = "debug")]
use bevy::log::debug;
use bevy::{ecs::component::Component, math::UVec2};
use turborand::TurboRand;
use Direction::{Bottom, Left, Right, Top};

use super::maze::{MAZE_MARGIN, MAZE_ROOMS, MAZE_SIZE};
use crate::util::Rand;

/// Get the neighbors of a tile, along with the direction towards which they are
/// from the input tile position
pub fn neighbors(UVec2 { x, y }: UVec2) -> impl Iterator<Item = (UVec2, Direction)> {
	[
		((x, u32::min(y + 1, MAZE_SIZE.y - 1 - MAZE_MARGIN)), Top),
		((u32::min(x + 1, MAZE_SIZE.x - 1 - MAZE_MARGIN), y), Right),
		((x, y.saturating_sub(1 + MAZE_MARGIN) + MAZE_MARGIN), Bottom),
		((x.saturating_sub(1 + MAZE_MARGIN) + MAZE_MARGIN, y), Left),
	]
	.into_iter()
	.map(|(v, d)| (UVec2::from(v), d))
}

/// Get the next tile in the maze for the usual recursive backtracking
/// algorithm
pub fn next_maze(pos: UVec2, visited: &[UVec2], rng: &Rand) -> Option<(UVec2, Direction)> {
	rng.sample_iter(neighbors(pos).filter(|(p, _)| !visited.contains(p)))
}

#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn gen_maze(rng: &Rand) -> Vec<Tile> {
	let us = |u32: u32| -> usize { u32.try_into().unwrap() };
	let idx = |UVec2 { x, y }| usize::try_from(y * MAZE_SIZE.x + x).unwrap();

	let mut maze = iter::from_fn(|| Some(Tile::grass(rng)))
		.take(us(MAZE_SIZE.x) * us(MAZE_SIZE.y))
		.collect::<Vec<_>>();

	let mut pos = MAZE_SIZE / 2;
	let mut visited = Vec::with_capacity(us(MAZE_SIZE.x) * us(MAZE_SIZE.y));
	visited.push(pos);
	let mut route = vec![pos];

	for x in MAZE_MARGIN..MAZE_SIZE.x - MAZE_MARGIN {
		for y in MAZE_MARGIN..MAZE_SIZE.x - MAZE_MARGIN {
			maze[idx(UVec2::new(x, y))] = Tile::CLOSED;
		}
	}

	loop {
		let Some((next, dir)) = next_maze(pos, &visited, rng) else {
			pos = if let Some(p) = route.pop() {
				p
			} else {
				break;
			};
			continue;
		};

		maze[idx(pos)].open(dir);
		maze[idx(next)].open(-dir);

		visited.push(next);
		route.push(next);

		pos = next;

		#[cfg(feature = "debug")]
		#[allow(clippy::cast_precision_loss)]
		if visited.len() % 512 == 0 {
			debug!(
				"gen_maze - {:.2}%",
				100.0 * visited.len() as f32 / (MAZE_SIZE.x as f32 * MAZE_SIZE.y as f32)
			);
		}
	}

	for pos in rng
		.sample_multiple_iter(
			(MAZE_MARGIN + 1..MAZE_SIZE.x - 1 - MAZE_MARGIN).flat_map(|x| {
				(MAZE_MARGIN + 1..MAZE_SIZE.y - 1 - MAZE_MARGIN).map(move |y| UVec2 { x, y })
			}),
			MAZE_ROOMS,
		)
		.into_iter()
		.chain([MAZE_SIZE / 2])
	{
		maze[idx(pos)]
			.open(Direction::Top)
			.open(Direction::Right)
			.open(Direction::Bottom)
			.open(Direction::Left);

		for (pos, dir) in neighbors(pos) {
			maze[idx(pos)].open(-dir);
		}
	}

	let exit = UVec2::new(
		rng.u32(MAZE_MARGIN + 1..MAZE_SIZE.x - 1 - MAZE_MARGIN),
		MAZE_SIZE.y - MAZE_MARGIN,
	);
	maze[idx(exit)].open(Bottom);
	maze[idx(exit - UVec2::Y)].open(Top);

	for x in MAZE_MARGIN - 1..=MAZE_SIZE.x - MAZE_MARGIN {
		let pos = UVec2::new(x, MAZE_SIZE.y - MAZE_MARGIN);
		maze[idx(pos)].open(Top);
		maze[idx(pos)].open(Left);
		maze[idx(pos)].open(Right);

		let pos = UVec2::new(x, MAZE_MARGIN - 1);
		maze[idx(pos)].open(Bottom);
		maze[idx(pos)].open(Left);
		maze[idx(pos)].open(Right);
	}

	for y in MAZE_MARGIN - 1..=MAZE_SIZE.y - MAZE_MARGIN {
		let pos = UVec2::new(MAZE_SIZE.x - MAZE_MARGIN, y);
		maze[idx(pos)].open(Top);
		maze[idx(pos)].open(Bottom);
		maze[idx(pos)].open(Right);

		let pos = UVec2::new(MAZE_MARGIN - 1, y);
		maze[idx(pos)].open(Top);
		maze[idx(pos)].open(Bottom);
		maze[idx(pos)].open(Left);
	}

	for i in 0..maze.len() {
		maze[i].0 = tile_bits(i, &maze);
	}

	for x in MAZE_MARGIN - 1..=MAZE_SIZE.x - MAZE_MARGIN {
		let pos = UVec2::new(x, MAZE_SIZE.y - MAZE_MARGIN);
		maze[idx(pos)].0 &= 0b0011_1111;

		let pos = UVec2::new(x, MAZE_MARGIN - 1);
		maze[idx(pos)].0 &= 0b1100_1111;
	}

	for y in MAZE_MARGIN - 1..=MAZE_SIZE.y - MAZE_MARGIN {
		let pos = UVec2::new(MAZE_SIZE.x - MAZE_MARGIN, y);
		maze[idx(pos)].0 &= 0b1010_1111;

		let pos = UVec2::new(MAZE_MARGIN - 1, y);
		maze[idx(pos)].0 &= 0b0101_1111;
	}

	maze
}

fn tile_bits(i: usize, maze: &[Tile]) -> u8 {
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

#[derive(Debug, Clone, Copy, Component)]
pub struct Tile(pub u8);

impl Tile {
	/// Fully closed stone tile
	pub const CLOSED: Self = Self(0b1111);
	/// Fully open stone tile
	pub const OPEN: Self = Self(0);

	pub fn grass(rng: &Rand) -> Self {
		Self(rng.u8(1..=0xf) << 4 | 0b1111)
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
		self.0 & 0b1111 == 0b1111 && self.0 >> 4 != 0
	}
}

impl Default for Tile {
	fn default() -> Self {
		Self(0b1111)
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
