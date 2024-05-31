//! Algorithms and data structures used for generating and solving the maze.

#[cfg(feature = "debug")]
use bevy::log::debug;
use bevy::{
	ecs::system::Resource,
	math::UVec2,
	utils::{HashMap, HashSet},
};
use turborand::TurboRand;

use super::maze::{Maze, TilePos, MAZE_SIZE};
use crate::{
	maze::{
		Direction::{self, Bottom, Left, Right, Top},
		Tile,
	},
	util::Rand,
};

/// Maze generation parameters
#[derive(Debug, Copy, Clone, Resource)]
pub struct MazeParams {
	/// The width of the maze in tiles
	pub width: u16,
	/// The height of the maze in tiles
	pub height: u16,
	/// The number of fully-open rooms in the maze
	pub rooms: u16,
	/// The directional bias of passages in the maze
	pub bias: DirectionalBias,
}

impl MazeParams {
	/// Get the margin (distance between the edge of the world and the maze) in
	/// the x axis
	pub fn margin_x(self) -> u32 {
		(MAZE_SIZE.x - u32::from(self.width)) / 2 + 1
	}

	/// Get the margin (distance between the edge of the world and the maze) in
	/// the y axis
	pub fn margin_y(self) -> u32 {
		(MAZE_SIZE.y - u32::from(self.height)) / 2 + 1
	}

	/// Get the width of the maze as a `u32`
	pub fn width(self) -> u32 {
		u32::from(self.width)
	}

	/// Get the height of the maze as a `u32`
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

/// The directional bias of passages in the maze
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DirectionalBias {
	/// No bias, all directions are equally likely
	None,
	/// Horizontal bias, horizontal directions are twice as likely
	Horizontal,
	/// Strong horizontal bias, horizontal directions are five times as likely
	VeryHorizontal,
	/// Vertical bias, vertical directions are twice as likely
	Vertical,
	/// Strong vertical bias, vertical directions are five times as likely
	VeryVertical,
}

/// Get the neighbors of a tile, along with the direction towards which they are
/// from the input tile position. The returned values may include the input
/// value if movement in a direction is not possible.
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

/// Randomly get the next tile in the maze for the usual recursive backtracking
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

/// Generate the maze
#[cfg_attr(feature = "debug", tracing::instrument(skip(maze, rng)))]
pub fn gen_maze(maze: &mut [Tile], rng: &Rand, params: MazeParams) -> TilePos {
	let us = |u32: u32| -> usize { u32.try_into().unwrap() };
	let idx = |UVec2 { x, y }| usize::try_from(y * MAZE_SIZE.x + x).unwrap();

	// Keep track of visited positions, starting with the middle
	let mut pos = MAZE_SIZE / 2;
	let mut visited = Vec::with_capacity(us(params.width()) * us(params.height()));
	visited.push(pos);
	let mut route = vec![pos];

	loop {
		// Go in a random direction
		let Some((next, dir)) = next_maze(pos, &visited, rng, params) else {
			// All neighbours have been visited, backtrack
			pos = if let Some(p) = route.pop() {
				// Try again from the previous position
				p
			} else {
				// The current position is the starting position and backtracking is impossible,
				// the algorithms is done
				break;
			};
			continue;
		};

		// Open the wall between the current and next tiles
		maze[idx(pos)].open(dir);
		maze[idx(next)].open(-dir);

		visited.push(next);
		route.push(next);

		// Go to the next position
		pos = next;

		// In debug mode, print progress
		#[cfg(feature = "debug")]
		#[allow(clippy::cast_precision_loss)]
		if visited.len() % 512 == 0 {
			debug!(
				"gen_maze - {:.2}%",
				100.0 * visited.len() as f32 / (params.width() as f32 * params.height() as f32)
			);
		}
	}

	// Pick a random maze exit on the top
	let exit = UVec2::new(
		rng.u32(params.margin_x()..params.margin_x() + params.width()),
		params.margin_y() + params.height() - 1,
	);

	// Open the exit
	maze[idx(exit + UVec2::Y)].open(Bottom);
	maze[idx(exit)].open(Top);

	exit.into()
}

/// Generate the maze's rooms
#[cfg_attr(feature = "debug", tracing::instrument(skip(maze, rng)))]
pub fn gen_rooms(maze: &mut [Tile], rng: &Rand, params: MazeParams) {
	let idx = |UVec2 { x, y }| usize::try_from(y * MAZE_SIZE.x + x).unwrap();

	// Don't do anything if no rooms should be generated
	if params.rooms == 0 {
		return;
	}

	// Generate `params.rooms - 1` randomly positioned rooms and one room at the
	// starting position
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
		// Open all walls
		maze[idx(pos)].open(Top).open(Right).open(Bottom).open(Left);

		// Open neighbours' adjacent walls
		for (pos, dir) in neighbors(pos, params) {
			maze[idx(pos)].open(-dir);
		}
	}
}

/// A binary search-able [`Tree`]
#[derive(Debug, Clone)]
pub struct SortedTree<T> {
	inner: Tree<T>,
}

impl<T> SortedTree<T> {
	/// Create a new sorted tree from the given tree
	#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
	pub fn new(tree: Tree<T>) -> Self
	where
		T: Ord,
	{
		let mut nodes = tree
			.nodes
			.into_iter()
			.enumerate()
			.map(|(i, (v, p))| (v, p, i))
			.collect::<Vec<_>>();
		nodes.sort_unstable_by(|(a, ..), (b, ..)| a.cmp(b));

		let mut relocations = HashMap::with_capacity(nodes.len());

		for (i, (_, _, pi)) in nodes.iter().enumerate() {
			relocations.insert(*pi, i);
		}

		let mut nodes = nodes
			.into_iter()
			.map(|(v, p, _)| (v, p))
			.collect::<Vec<_>>();

		for (_, p) in &mut nodes {
			*p = *relocations.get(p).unwrap();
		}

		Self {
			inner: Tree { nodes },
		}
	}

	/// Get the value in the node at `idx`
	pub fn get(&self, idx: usize) -> Option<&T> {
		self.inner.get(idx)
	}

	/// Get the parent of the node at `idx`, returning `None` for the root node
	pub fn parent(&self, idx: usize) -> Option<usize> {
		self.inner.parent(idx)
	}

	/// Search for the index of a node with the given value (if there are
	/// multiple nodes with the same value, an arbitrary one is returned)
	#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
	pub fn search(&self, val: &T) -> Option<usize>
	where
		T: Ord,
	{
		self.inner.nodes.binary_search_by_key(&val, |(v, _)| v).ok()
	}
}

/// An append-only tree using indexes as "pointers" to the parent node
#[derive(Debug, Clone)]
pub struct Tree<T> {
	nodes: Vec<(T, usize)>,
}

impl<T> Tree<T> {
	/// Create a new tree with the given value as the root node
	pub fn new(root: T) -> Self {
		Self {
			nodes: vec![(root, 0)],
		}
	}

	/// Get the value in the node at `idx`
	pub fn get(&self, idx: usize) -> Option<&T> {
		self.nodes.get(idx).map(|(v, _)| v)
	}

	/// Append a new node with the given value to the node at `parent`
	pub fn append(&mut self, val: T, parent: usize) {
		self.nodes.push((val, parent));
	}

	/// Get the parent of the node at `idx`, returning `None` for the root node
	pub fn parent(&self, idx: usize) -> Option<usize> {
		let parent = self.nodes.get(idx)?.1;

		if idx == parent {
			None
		} else {
			Some(parent)
		}
	}

	/// Search for the index of a node with the given value (if there are
	/// multiple nodes with the same value, the first-inserted one is returned)
	#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
	pub fn search(&self, val: &T) -> Option<usize>
	where
		T: PartialEq,
	{
		self.nodes.iter().position(|(v, _)| v == val)
	}
}

/// Get all reachable neighbours of the tile `tile` at `pos`
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

/// Solve the given maze, returning a minimum-distance tree with `start` as the
/// root node
#[cfg_attr(feature = "debug", tracing::instrument(skip(maze)))]
pub fn solve_maze(maze: &Maze, start: TilePos, params: MazeParams) -> SortedTree<TilePos> {
	let mut tree = Tree::new(start);

	// Mark all nodes as unvisited
	let mut unvisited = (params.margin_x()..params.margin_x() + params.width())
		.flat_map(|x| {
			(params.margin_y()..params.margin_y() + params.height()).map(move |y| TilePos { x, y })
		})
		.collect::<HashSet<_>>();

	// Assign to every node a distance from the start, initially infinity
	// (`u32::MAX`)
	let mut distances = unvisited
		.iter()
		.map(|&p| (p, u32::MAX))
		.collect::<HashMap<_, _>>();

	// The start node has a distance to start of 0
	*distances.get_mut(&start).unwrap() = 0;
	let mut current = start;

	loop {
		// Update the distances of all reachable unvisited neighbours of the current
		// node to the minimum of their current distances and the current node's
		// distance plus one.
		#[allow(clippy::needless_collect)]
		for unvisited_neighbour in reachable_neighbours(maze.get(current), current, params)
			.filter(|&p| unvisited.contains(&p))
			.collect::<Vec<_>>()
		{
			let current_distance = *distances.get(&current).unwrap();
			let neighbour_distance = distances.get_mut(&unvisited_neighbour).unwrap();
			*neighbour_distance = (*neighbour_distance).min(current_distance + 1);
		}

		// Mark the current node as visited
		unvisited.remove(&current);

		// Append the current node to its neighbour with the minimum distance
		let min_neighbour = reachable_neighbours(maze.get(current), current, params)
			.min_by_key(|n| *distances.get(n).unwrap())
			.filter(|n| *distances.get(n).unwrap() != u32::MAX)
			.unwrap_or(start);
		tree.append(current, tree.search(&min_neighbour).unwrap_or_default());

		// Go to the unvisited node with the smallest finite current distance
		current = if let Some(new) = unvisited
			.iter()
			.filter(|&n| *distances.get(n).unwrap() != u32::MAX)
			.min_by_key(|&n| *distances.get(n).unwrap())
		{
			*new
		} else {
			// There are no more reachable unvisited node, the algorithm is done
			break;
		}
	}

	SortedTree::new(tree)
}
