use bevy::prelude::*;

// pub fn init(mut commands: Commands, asset_server: Res<AssetServer>) {
// 	commands
// 		.spawn(ImageBundle {
// 			background_color: Color::WHITE.into(),
// 			style: Style {
// 				position_type: PositionType::Absolute,
// 				bottom: Val::Percent(5.0),
// 				right: Val::Percent(5.0),
// 				width: Val::Px(128.0),
// 				height: Val::Px(128.0),
// 				display: Display::Flex,
// 				align_items: AlignItems::Center,
// 				justify_content: JustifyContent::Center,
// 				..default()
// 			},
// 			image: UiImage {
// 				texture: plate,
// 				..default()
// 			},
// 			..default()
// 		})
// 		.with_children(|builder| {
// 			builder.spawn((TextBundle::from_section(
// 				"0",
// 				TextStyle {
// 					font: asset_server.load("fonts/pixel.ttf"),
// 					font_size: 64.0,
// 					color: Color::BLACK,
// 				},
// 			)
// 			.with_text_justify(JustifyText::Center)
// 			.with_style(Style {
// 				position_type: PositionType::Relative,
// 				..default()
// 			}),));
// 		});
// }
