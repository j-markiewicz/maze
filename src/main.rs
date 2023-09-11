#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::needless_pass_by_value)] // A bunch of Bevy things require this
#![allow(clippy::module_name_repetitions)]

pub mod util;

use std::time::Duration;

use bevy::{asset::ChangeWatcher, log::LogPlugin, prelude::*, window::WindowMode};
#[cfg(feature = "debug")]
use bevy::{
	diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
	log::Level,
	window::close_on_esc,
};
use bevy_embedded_assets::EmbeddedAssetPlugin;
use rand::seq::SliceRandom;

games! {
	"asteroids" => asteroids,
	// "maze" => maze,
	// "portoom" => portoom,
	// "racecar" => racecar,
	// "lander" => lander,
	// "astroguessr" => astroguessr,
	// "mapman" => mapman,
}

#[bevy_main]
#[allow(clippy::missing_panics_doc)]
pub fn main() {
	#[cfg(target_arch = "wasm32")]
	console_error_panic_hook::set_once();

	let game = GAMES
		.choose(&mut rand::thread_rng())
		.expect("there are no games");

	println!("Starting game \"{}\"", game.name);

	let mut app = App::new();

	let default_plugins = DefaultPlugins
		.set(WindowPlugin {
			primary_window: Some(Window {
				mode: WindowMode::BorderlessFullscreen,
				resizable: true,
				fit_canvas_to_parent: true,
				canvas: cfg!(target_arch = "wasm32").then(|| "#background".to_string()),
				title: if cfg!(target_arch = "wasm32") {
					""
				} else {
					"web-bg"
				}
				.to_string(),
				..default()
			}),
			..default()
		})
		.set(ImagePlugin::default_nearest())
		.set(AssetPlugin {
			watch_for_changes: cfg!(feature = "debug").then_some(ChangeWatcher {
				delay: Duration::from_millis(500),
			}),
			..default()
		})
		.add_before::<AssetPlugin, _>(EmbeddedAssetPlugin)
		.disable::<LogPlugin>();

	app.insert_resource(ClearColor(Color::NONE))
		.insert_resource(Msaa::Sample4)
		.add_plugins(default_plugins);

	#[cfg(feature = "debug")]
	{
		app.add_plugins((
			LogPlugin {
				level: Level::DEBUG,
				..default()
			},
			LogDiagnosticsPlugin::default(),
			FrameTimeDiagnosticsPlugin,
		));
		app.add_systems(Update, close_on_esc);
	}

	(game.start)(&mut app);
	app.run();
}
