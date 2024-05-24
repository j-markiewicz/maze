//! Utilities and other miscellaneous things.

#[cfg(all(feature = "console_log", target_arch = "wasm32"))]
use std::fmt::{Error as FmtError, Result as FmtResult};

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

	pub const fn from_rng(rng: AtomicRng) -> Self {
		Self(rng)
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
