use std::f32;

use bevy::{prelude::*, render::camera::ClearColorConfig, window::PrimaryWindow};

use crate::player::Player;

const SUN_BRIGHTNESS: f32 = 50_000.0;

pub fn initialization(mut commands: Commands) {
	commands.spawn((
		Camera2dBundle {
			camera: Camera {
				order: 1,
				clear_color: ClearColorConfig::None,
				..default()
			},
			..default()
		},
		InheritedVisibility::default(),
		ViewVisibility::default(),
	));

	commands
		.spawn((
			Camera3dBundle {
				camera: Camera {
					order: 0,
					..default()
				},
				projection: Projection::Orthographic(OrthographicProjection::default()),
				transform: Transform {
					translation: Vec3 {
						x: 0.0,
						y: 0.0,
						z: 10.0,
					},
					..default()
				},
				..default()
			},
			InheritedVisibility::default(),
			ViewVisibility::default(),
		))
		.with_children(|builder| {
			builder.spawn(DirectionalLightBundle {
				directional_light: DirectionalLight {
					color: Color::ANTIQUE_WHITE,
					illuminance: SUN_BRIGHTNESS,
					shadows_enabled: true,
					..default()
				},
				..default()
			});
		});
}

pub fn movement(
	mut cameras: Query<&mut Transform, (With<Camera>, Without<Player>)>,
	player: Query<&Transform, With<Player>>,
	window: Query<&Window, With<PrimaryWindow>>,
) {
	/// The free movement space on each side of the screen as a proportion of
	/// the width/height of the screen
	const FREE_MOVEMENT_SPACE_PROPORTION: f32 = 0.2;

	for mut camera in &mut cameras {
		let player = player.single();
		let window = window.single();

		let (width, height) = (
			window.width() * FREE_MOVEMENT_SPACE_PROPORTION,
			window.height() * FREE_MOVEMENT_SPACE_PROPORTION,
		);
		let player_displacement = player.translation - camera.translation;

		let deadzoned_displacement_x = player_displacement.x.abs() - width;
		let deadzoned_displacement_x = if deadzoned_displacement_x.is_sign_negative() {
			0.0
		} else {
			deadzoned_displacement_x
		};
		let deadzoned_displacement_x = deadzoned_displacement_x.copysign(player_displacement.x);

		let deadzoned_displacement_y = player_displacement.y.abs() - height;
		let deadzoned_displacement_y = if deadzoned_displacement_y.is_sign_negative() {
			0.0
		} else {
			deadzoned_displacement_y
		};
		let deadzoned_displacement_y = deadzoned_displacement_y.copysign(player_displacement.y);

		camera.translation += Vec3 {
			x: deadzoned_displacement_x,
			y: deadzoned_displacement_y,
			z: 0.0,
		};
	}
}
