use std::ops::Neg;

#[cfg(feature = "debug")]
use bevy::log::debug;
use bevy::{
	ecs::{component::Component, system::Resource},
	math::UVec2,
	utils::{HashMap, HashSet},
};
use turborand::TurboRand;
use Direction::{Bottom, Left, Right, Top};

use super::maze::{Maze, TilePos, MAZE_SIZE};
use crate::util::Rand;

#[derive(Debug, Copy, Clone, Resource)]
pub struct MazeParams {
	pub width: u16,
	pub height: u16,
	pub rooms: u16,
	pub bias: DirectionalBias,
}

impl MazeParams {
	pub fn margin_x(self) -> u32 {
		(MAZE_SIZE.x - u32::from(self.width)) / 2 + 1
	}

	pub fn margin_y(self) -> u32 {
		(MAZE_SIZE.y - u32::from(self.height)) / 2 + 1
	}

	pub fn width(self) -> u32 {
		u32::from(self.width)
	}

	pub fn height(self) -> u32 {
		u32::from(self.height)
	}
}

impl Default for MazeParams {
	fn default() -> Self {
		Self {
			width: 7,
			height: 5,
			rooms: 2,
			bias: DirectionalBias::None,
		}
	}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DirectionalBias {
	None,
	Horizontal,
	VeryHorizontal,
	Vertical,
	VeryVertical,
}

/// Get the neighbors of a tile, along with the direction towards which they are
/// from the input tile position
pub fn neighbors(
	UVec2 { x, y }: UVec2,
	params: MazeParams,
) -> impl Iterator<Item = (UVec2, Direction)> + Clone {
	let mx = params.margin_x();
	let my = params.margin_y();
	let h = params.height();
	let w = params.width();

	[
		((x, u32::min(y + 1, my + h - 1)), Top),
		((u32::min(x + 1, mx + w - 1), y), Right),
		((x, u32::max(y - 1, my)), Bottom),
		((u32::max(x - 1, mx), y), Left),
	]
	.into_iter()
	.map(|(v, d)| (UVec2::from(v), d))
}

/// Get the next tile in the maze for the usual recursive backtracking
/// algorithm
pub fn next_maze(
	pos: UVec2,
	visited: &[UVec2],
	rng: &Rand,
	params: MazeParams,
) -> Option<(UVec2, Direction)> {
	let neighbours = neighbors(pos, params).filter(|(p, _)| !visited.contains(p));
	let hor_neighbours = neighbours.clone().filter(|&(_, d)| d == Left || d == Right);
	let ver_neighbours = neighbours.clone().filter(|&(_, d)| d == Top || d == Bottom);

	match params.bias {
		DirectionalBias::None => rng.sample_iter(neighbours),
		DirectionalBias::Horizontal => rng.sample_iter(hor_neighbours.chain(neighbours)),
		DirectionalBias::Vertical => rng.sample_iter(ver_neighbours.chain(neighbours)),
		DirectionalBias::VeryHorizontal => rng.sample_iter(
			hor_neighbours.clone().chain(
				hor_neighbours.clone().chain(
					hor_neighbours
						.clone()
						.chain(hor_neighbours)
						.chain(neighbours),
				),
			),
		),
		DirectionalBias::VeryVertical => rng.sample_iter(
			ver_neighbours.clone().chain(
				ver_neighbours.clone().chain(
					ver_neighbours
						.clone()
						.chain(ver_neighbours)
						.chain(neighbours),
				),
			),
		),
	}
}

#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn gen_maze(maze: &mut [Tile], rng: &Rand, params: MazeParams) -> TilePos {
	let us = |u32: u32| -> usize { u32.try_into().unwrap() };
	let idx = |UVec2 { x, y }| usize::try_from(y * MAZE_SIZE.x + x).unwrap();

	let mut pos = MAZE_SIZE / 2;
	let mut visited = Vec::with_capacity(us(params.width()) * us(params.height()));
	visited.push(pos);
	let mut route = vec![pos];

	loop {
		let Some((next, dir)) = next_maze(pos, &visited, rng, params) else {
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

	let exit = UVec2::new(
		rng.u32(params.margin_x()..params.margin_x() + params.width()),
		params.margin_y() + params.height() - 1,
	);
	maze[idx(exit + UVec2::Y)].open(Bottom);
	maze[idx(exit)].open(Top);

	exit.into()
}

#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn gen_rooms(maze: &mut [Tile], rng: &Rand, params: MazeParams) {
	let idx = |UVec2 { x, y }| usize::try_from(y * MAZE_SIZE.x + x).unwrap();

	if params.rooms > 0 {
		for pos in rng
			.sample_multiple_iter(
				(params.margin_x() + 1..params.margin_x() + params.width() - 1).flat_map(|x| {
					(params.margin_y() + 1..params.margin_y() + params.height() - 1)
						.map(move |y| UVec2 { x, y })
				}),
				(params.rooms - 1).into(),
			)
			.into_iter()
			.chain([MAZE_SIZE / 2])
		{
			maze[idx(pos)]
				.open(Direction::Top)
				.open(Direction::Right)
				.open(Direction::Bottom)
				.open(Direction::Left);

			for (pos, dir) in neighbors(pos, params) {
				maze[idx(pos)].open(-dir);
			}
		}
	}
}

#[derive(Debug, Clone)]
pub struct Tree<T> {
	nodes: Vec<(T, usize)>,
}

impl<T> Tree<T> {
	pub fn new(root: T) -> Self {
		Self {
			nodes: vec![(root, 0)],
		}
	}

	pub fn get(&self, idx: usize) -> Option<&T> {
		self.nodes.get(idx).map(|(v, _)| v)
	}

	pub fn append(&mut self, val: T, parent: usize) {
		self.nodes.push((val, parent));
	}

	pub fn parent(&self, idx: usize) -> Option<usize> {
		if idx != 0 {
			Some(self.nodes.get(idx)?.1)
		} else {
			None
		}
	}

	pub fn search(&self, val: &T) -> Option<usize>
	where
		T: PartialEq,
	{
		self.nodes.iter().position(|(v, _)| v == val)
	}
}

fn reachable_neighbours(
	tile: Tile,
	pos: TilePos,
	params: MazeParams,
) -> impl Iterator<Item = TilePos> {
	neighbors(pos.into(), params)
		.filter(move |(_, d)| tile.is_open(*d))
		.map(|(n, _)| n.into())
		.filter(move |&p| p != pos)
}

pub fn solve_maze(maze: &Maze, start: TilePos, params: MazeParams) -> Tree<TilePos> {
	let mut tree = Tree::new(start);

	// 1. Mark all nodes as unvisited. Create a set of all the unvisited nodes
	//    called the unvisited set.
	let mut unvisited = (params.margin_x()..params.margin_x() + params.width())
		.flat_map(|x| {
			(params.margin_y()..params.margin_y() + params.width()).map(move |y| TilePos { x, y })
		})
		.collect::<HashSet<_>>();

	// 2. Assign to every node a distance from start value: for the starting node,
	//    it is zero, and for all other nodes, it is infinity, since initially no
	//    path is known to these nodes. During execution of the algorithm, the
	//    distance of a node N is the length of the shortest path discovered so far
	//    between the starting node and N. Set the current node to be the starting
	//    node.
	let mut distances = unvisited
		.iter()
		.map(|&p| (p, u32::MAX))
		.collect::<HashMap<_, _>>();
	*distances.get_mut(&start).unwrap() = 0;
	let mut current = start;

	loop {
		// 3. For the current node, consider all of its unvisited neighbors and update
		//    their distances through the current node: Compare the newly calculated
		//    distance to the one currently assigned to the neighbor and assign it the
		//    smaller one. For example, if the current node A is marked with a distance
		//    of 6, and the edge connecting it with its neighbor B has length 2, then
		//    the distance to B through A is 6 + 2 = 8. If B was previously marked with
		//    a distance greater than 8, then update it to 8 (the path to B through A is
		//    shorter). Otherwise, keep its current distance (the path to B through A is
		//    not the shortest).
		#[allow(clippy::needless_collect)]
		for unvisited_neighbour in reachable_neighbours(maze.get(current), current, params)
			.filter(|&p| unvisited.contains(&p))
			.collect::<Vec<_>>()
		{
			let current_distance = *distances.get(&current).unwrap();
			let neighbour_distance = distances.get_mut(&unvisited_neighbour).unwrap();
			*neighbour_distance = (*neighbour_distance).min(current_distance + 1);
		}

		// 4. When we are done considering all of the unvisited neighbors of the current
		//    node, mark the current node as visited and remove it from the unvisited
		//    set. A visited node will never be checked again. At this point, the
		//    recorded distance of the current node is final and minimal, because this
		//    node was selected to be the next to visit due to having the smallest
		//    distance from the starting node; any paths discovered thereafter would not
		//    be shorter.
		unvisited.remove(&current);

		let min_neighbour = reachable_neighbours(maze.get(current), current, params)
			.min_by_key(|n| *distances.get(n).unwrap())
			.filter(|n| *distances.get(n).unwrap() != u32::MAX)
			.unwrap_or(start);
		tree.append(current, tree.search(&min_neighbour).unwrap_or_default());

		// 5. From the unvisited nodes, select the one with the smallest known distance
		//    as the new "current node" and go back to step 3. If an unvisited node has
		//    an "infinity" distance, it means that it is unreachable (so far) and
		//    should not be selected. If there are no more reachable unvisited nodes,
		//    the algorithm has finished. If the new "current node" is the target node,
		//    then we have found the shortest path to it. We can exit here, or continue
		//    to find the shortest paths to all reachable nodes.
		current = if let Some(new) = unvisited
			.iter()
			.filter(|&n| *distances.get(n).unwrap() != u32::MAX)
			.min_by_key(|&n| *distances.get(n).unwrap())
		{
			*new
		} else {
			break;
		}
	}

	// 6. Once the loop exits, the shortest path can be extracted from the set of
	//    visited nodes by starting from the target node and picking its neighbor
	//    with the shortest distance, going back to start on an optimal path. If the
	//    target node distance is infinity, no path exists.

	tree
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
