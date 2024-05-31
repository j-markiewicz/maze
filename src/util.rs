//! Utilities and other miscellaneous things.

#[cfg(all(feature = "console_log", target_arch = "wasm32"))]
use std::fmt::{Error as FmtError, Result as FmtResult};
#[cfg(feature = "debug")]
use std::{
	alloc::{GlobalAlloc, Layout},
	sync::atomic::{AtomicUsize, Ordering},
};

use bevy::prelude::*;
#[cfg(all(feature = "console_log", target_arch = "wasm32"))]
use tracing_core::{subscriber::Interest, Level, Metadata};
#[cfg(all(feature = "console_log", target_arch = "wasm32"))]
use tracing_subscriber::{
	filter::LevelFilter,
	fmt::{format::Writer, time::FormatTime},
	layer::{Context, Filter},
};
use turborand::rng::AtomicRng;
pub use turborand::TurboRand;

/// Random number generator resource
#[derive(Debug, Resource, Deref, DerefMut)]
pub struct Rand(AtomicRng);

impl Rand {
	#[allow(clippy::new_without_default)]
	#[must_use]
	pub fn new() -> Self {
		Self(AtomicRng::new())
	}
}

/// Up/down/left/right movement input within the range from `-1.0` to `1.0`
///
/// If the input for either axis is within the deadzone, it is set to exactly
/// `0.0`
#[derive(Debug, Clone, Copy, Resource, Default)]
pub struct PlayerInput {
	pub up: f32,
	pub right: f32,
}

impl PlayerInput {
	/// Whether there is any input
	#[must_use]
	pub fn is_moving(self) -> bool {
		self.up != 0.0 || self.right != 0.0
	}
}

impl From<PlayerInput> for Vec2 {
	fn from(PlayerInput { up, right }: PlayerInput) -> Self {
		Self { x: right, y: up }
	}
}

impl From<PlayerInput> for Vec3 {
	fn from(PlayerInput { up, right }: PlayerInput) -> Self {
		Self {
			x: right,
			y: up,
			z: 0.0,
		}
	}
}

/// A system for processing up/down/left/right movement input, shared across
/// games
///
/// # Usage
///
/// Insert the [`PlayerInput`] resource into the app on startup (this is not
/// done automatically), then register this system, ideally before any
/// movement/animation processing (e.g. in the `PreUpdate`) schedule
#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn input(
	mut input: ResMut<PlayerInput>,
	key_input: Res<ButtonInput<KeyCode>>,
	gamepads: Res<Gamepads>,
	pad_input: Res<ButtonInput<GamepadButton>>,
	stick_input: Res<Axis<GamepadAxis>>,
) {
	const DEADZONE: f32 = 0.05;

	let mut up = 0.0;
	let mut right = 0.0;

	// Keyboard WASD
	if key_input.pressed(KeyCode::KeyW) {
		up += 1.0;
	}

	if key_input.pressed(KeyCode::KeyS) {
		up -= 1.0;
	}

	if key_input.pressed(KeyCode::KeyD) {
		right += 1.0;
	}

	if key_input.pressed(KeyCode::KeyA) {
		right -= 1.0;
	}

	// Keyboard arrow keys
	if key_input.pressed(KeyCode::ArrowUp) {
		up += 1.0;
	}

	if key_input.pressed(KeyCode::ArrowDown) {
		up -= 1.0;
	}

	if key_input.pressed(KeyCode::ArrowRight) {
		right += 1.0;
	}

	if key_input.pressed(KeyCode::ArrowLeft) {
		right -= 1.0;
	}

	for gamepad in gamepads.iter() {
		// Gamepad buttons
		if pad_input.pressed(GamepadButton {
			gamepad,
			button_type: GamepadButtonType::DPadUp,
		}) {
			up += 1.0;
		}

		if pad_input.pressed(GamepadButton {
			gamepad,
			button_type: GamepadButtonType::DPadDown,
		}) {
			up -= 1.0;
		}

		if pad_input.pressed(GamepadButton {
			gamepad,
			button_type: GamepadButtonType::DPadRight,
		}) {
			right += 1.0;
		}

		if pad_input.pressed(GamepadButton {
			gamepad,
			button_type: GamepadButtonType::DPadLeft,
		}) {
			right -= 1.0;
		}

		// Gamepad stick
		if let Some(i) = stick_input.get(GamepadAxis {
			gamepad,
			axis_type: GamepadAxisType::LeftStickY,
		}) {
			if i.abs() > DEADZONE {
				up += i;
			}
		}

		if let Some(i) = stick_input.get(GamepadAxis {
			gamepad,
			axis_type: GamepadAxisType::LeftStickX,
		}) {
			if i.abs() > DEADZONE {
				right += i;
			}
		}
	}

	let up = if up.abs() > DEADZONE { up } else { 0.0 };
	let right = if right.abs() > DEADZONE { right } else { 0.0 };

	*input = PlayerInput {
		up: up.clamp(-1.0, 1.0),
		right: right.clamp(-1.0, 1.0),
	}
}

/// A timer for `tracing_subscriber` using a timestamp from JS `performance.now`
#[derive(Debug, Clone, Copy)]
#[cfg(all(feature = "console_log", target_arch = "wasm32"))]
pub struct PerformanceTimer;

#[cfg(all(feature = "console_log", target_arch = "wasm32"))]
impl FormatTime for PerformanceTimer {
	fn format_time(&self, w: &mut Writer<'_>) -> FmtResult {
		let now = web_sys::window()
			.ok_or(FmtError)?
			.performance()
			.ok_or(FmtError)?
			.now();
		w.write_fmt(format_args!("{now}"))
	}
}

/// A filter for `tracing_subscriber` similar to the default bevy filter
/// (`"wgpu=error,naga=warn,maze=debug", otherwise info`)
#[derive(Debug, Clone, Copy)]
#[cfg(all(feature = "console_log", target_arch = "wasm32"))]
pub struct LogFilter;

#[cfg(all(feature = "console_log", target_arch = "wasm32"))]
impl LogFilter {
	fn is_enabled(meta: &Metadata<'_>) -> bool {
		let path = meta.module_path().unwrap_or("");
		if path.starts_with("wgpu") {
			meta.level() <= &Level::ERROR
		} else if path.starts_with("naga") {
			meta.level() <= &Level::WARN
		} else if path.starts_with("maze") {
			meta.level() <= &Level::DEBUG
		} else {
			meta.level() <= &Level::INFO
		}
	}
}

#[cfg(all(feature = "console_log", target_arch = "wasm32"))]
impl<S> Filter<S> for LogFilter {
	fn enabled(&self, meta: &Metadata<'_>, _: &Context<'_, S>) -> bool {
		Self::is_enabled(meta)
	}

	fn callsite_enabled(&self, meta: &'static Metadata<'static>) -> Interest {
		if Self::is_enabled(meta) {
			Interest::always()
		} else {
			Interest::never()
		}
	}

	fn max_level_hint(&self) -> Option<LevelFilter> {
		Some(LevelFilter::DEBUG)
	}
}

#[cfg(feature = "debug")]
#[derive(Debug)]
pub struct LogMemoryUsagePlugin;

#[cfg(feature = "debug")]
impl Plugin for LogMemoryUsagePlugin {
	fn name(&self) -> &str {
		"log memory usage"
	}

	fn build(&self, app: &mut App) {
		#[derive(Debug, Resource)]
		struct MemoryTimer(Timer);

		fn track_memory_usage(time: Res<Time>, mut timer: ResMut<MemoryTimer>) {
			timer.0.tick(time.delta());

			if timer.0.just_finished() {
				let allocated_bytes = crate::ALLOC.allocated_bytes.load(Ordering::Relaxed);
				let allocated = allocated_bytes / 1024 / 1024;
				info!(%allocated_bytes, "currently allocated memory: {allocated} MiB");
			}
		}

		app.insert_resource(MemoryTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));
		app.add_systems(Update, track_memory_usage);
	}
}

#[cfg(feature = "debug")]
pub struct TrackingAlloc<A: GlobalAlloc> {
	underlying: A,
	allocated_bytes: AtomicUsize,
}

#[cfg(feature = "debug")]
impl<A: GlobalAlloc> TrackingAlloc<A> {
	pub const fn new(allocator: A) -> Self {
		Self {
			underlying: allocator,
			allocated_bytes: AtomicUsize::new(0),
		}
	}
}

#[cfg(feature = "debug")]
#[allow(unsafe_code)]
// SAFETY: This impl directly forwards to the `GlobalAlloc` impl of `A`
unsafe impl<A: GlobalAlloc> GlobalAlloc for TrackingAlloc<A> {
	#[allow(unsafe_code)]
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		self.allocated_bytes
			.fetch_add(layout.size(), Ordering::Relaxed);

		// SAFETY: This method has the exact same preconditions as the current method,
		// which the caller must uphold
		unsafe { self.underlying.alloc(layout) }
	}

	#[allow(unsafe_code)]
	unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
		self.allocated_bytes
			.fetch_add(layout.size(), Ordering::Relaxed);

		// SAFETY: This method has the exact same preconditions as the current method,
		// which the caller must uphold
		self.underlying.alloc_zeroed(layout)
	}

	#[allow(unsafe_code)]
	unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
		let difference = layout.size().abs_diff(new_size);

		if layout.size() > new_size {
			self.allocated_bytes
				.fetch_sub(difference, Ordering::Relaxed);
		} else {
			self.allocated_bytes
				.fetch_add(difference, Ordering::Relaxed);
		}

		// SAFETY: This method has the exact same preconditions as the current method,
		// which the caller must uphold
		self.underlying.realloc(ptr, layout, new_size)
	}

	#[allow(unsafe_code)]
	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		self.allocated_bytes
			.fetch_sub(layout.size(), Ordering::Relaxed);

		// SAFETY: This method has the exact same preconditions as the current method,
		// which the caller must uphold
		self.underlying.dealloc(ptr, layout);
	}
}
