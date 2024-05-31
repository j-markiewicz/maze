use bevy::{app::AppExit, prelude::*};
use bevy_simple_text_input::{
	TextInputBundle, TextInputCursorPos, TextInputInactive, TextInputSettings, TextInputTextStyle,
	TextInputValue,
};

use crate::{
	algorithms::{DirectionalBias, MazeParams},
	maze::{RegenerateMaze, MAX_MAZE_SIZE, MIN_MAZE_SIZE},
};

const ACTIVE_SELECTOR_COLOR: Color = Color::WHITE;
const INACTIVE_SELECTOR_COLOR: Color = Color::BLACK;

#[derive(Debug, Clone, Copy, Resource)]
pub struct Ui(Option<Entity>);

#[derive(Debug, Clone, Copy, Component)]
pub enum UiButton {
	Generate,
	Close,
}

#[derive(Debug, Clone, Copy, Component)]
pub struct UiSelector(pub DirectionalBias);

impl UiSelector {
	pub fn text(self) -> String {
		match self.0 {
			DirectionalBias::None => "+",
			DirectionalBias::Vertical => "|",
			DirectionalBias::Horizontal => "-",
			DirectionalBias::VeryVertical => "||",
			DirectionalBias::VeryHorizontal => "--",
		}
		.to_string()
	}
}

#[derive(Debug, Clone, Copy, Component)]
pub enum UiInput {
	Width,
	Height,
	Rooms,
}

impl UiInput {
	fn text(self) -> String {
		match self {
			Self::Width => "Szerokosc",
			Self::Height => "Wysokosc",
			Self::Rooms => "Pokoje",
		}
		.to_string()
	}

	fn get(self, params: MazeParams) -> String {
		match self {
			Self::Width => params.width,
			Self::Height => params.height,
			Self::Rooms => params.rooms,
		}
		.to_string()
	}
}

#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn initialize(mut commands: Commands, asset_server: Res<AssetServer>, params: Res<MazeParams>) {
	let ui = spawn(&mut commands, asset_server, *params);
	commands.insert_resource(Ui(Some(ui)));
}

#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn open_close(
	mut ui: ResMut<Ui>,
	mut commands: Commands,
	key_input: Res<ButtonInput<KeyCode>>,
	gamepads: Res<Gamepads>,
	pad_input: Res<ButtonInput<GamepadButton>>,
	asset_server: Res<AssetServer>,
	params: Res<MazeParams>,
) {
	let mut just_pressed = false;

	for gamepad in gamepads.iter() {
		if pad_input.just_pressed(GamepadButton {
			gamepad,
			button_type: GamepadButtonType::Start,
		}) {
			just_pressed = true;
		}
	}

	if key_input.any_just_pressed([KeyCode::Tab, KeyCode::Escape]) {
		just_pressed = true;
	}

	if just_pressed {
		if let Some(e) = ui.0 {
			commands.entity(e).despawn_recursive();
			ui.0 = None;
		} else {
			ui.0 = Some(spawn(&mut commands, asset_server, *params));
		}
	}
}

#[allow(clippy::type_complexity)]
#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn click(
	mut interaction: Query<(&Interaction, &UiButton), (Changed<Interaction>, With<Button>)>,
	mut app_exit_events: EventWriter<AppExit>,
	mut events: EventWriter<RegenerateMaze>,
) {
	for (interaction, button) in &mut interaction {
		if *interaction == Interaction::Pressed {
			match button {
				UiButton::Generate => {
					events.send(RegenerateMaze);
				}
				UiButton::Close => {
					if !cfg!(target_arch = "wasm32") {
						app_exit_events.send(AppExit);
					}
				}
			}
		}
	}
}

#[allow(clippy::type_complexity)]
#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
pub fn select(
	mut query: Query<(&Interaction, &UiSelector, &mut BorderColor)>,
	mut maze_params: ResMut<MazeParams>,
) {
	for (interaction, selector, mut color) in &mut query {
		if *interaction == Interaction::Pressed {
			maze_params.bias = selector.0;
		}

		if selector.0 == maze_params.bias {
			color.0 = ACTIVE_SELECTOR_COLOR;
		} else {
			color.0 = INACTIVE_SELECTOR_COLOR;
		}
	}
}

pub fn focus(
	query: Query<(Entity, &Interaction), Changed<Interaction>>,
	mut text_inputs: Query<(
		Entity,
		&mut TextInputInactive,
		&TextInputValue,
		&mut TextInputCursorPos,
	)>,
) {
	for (interaction_entity, interaction) in &query {
		if *interaction == Interaction::Pressed {
			for (entity, mut inactive, value, mut pos) in &mut text_inputs {
				inactive.0 = entity != interaction_entity;
				pos.0 = value.0.len();
			}
		}
	}
}

pub fn update(
	mut input: Query<(&mut TextInputValue, &UiInput), Changed<TextInputValue>>,
	mut maze_params: ResMut<MazeParams>,
) {
	for (mut value, input) in &mut input {
		let current_value = value.0.parse::<u16>().unwrap_or_default();
		value.0 = current_value.to_string();

		match input {
			UiInput::Width => maze_params.width = current_value.clamp(MIN_MAZE_SIZE, MAX_MAZE_SIZE),
			UiInput::Height => {
				maze_params.height = current_value.clamp(MIN_MAZE_SIZE, MAX_MAZE_SIZE);
			}
			UiInput::Rooms => maze_params.rooms = current_value,
		}
	}
}

#[allow(clippy::too_many_lines)]
#[cfg_attr(feature = "debug", tracing::instrument(skip_all))]
fn spawn(commands: &mut Commands, asset_server: Res<AssetServer>, params: MazeParams) -> Entity {
	let menu = asset_server.load("maze/menu.png");

	let elem_style = |x, y| Style {
		width: Val::Percent(50.0),
		height: Val::Percent(80.0),
		margin: UiRect::horizontal(Val::Percent(1.0)),
		grid_column: GridPlacement::start(x),
		grid_row: GridPlacement::start(y),
		..default()
	};

	let text_style = TextStyle {
		font: asset_server.load("fonts/pixel.ttf"),
		font_size: 64.0,
		color: Color::WHITE,
	};

	commands
		.spawn(ImageBundle {
			style: Style {
				position_type: PositionType::Absolute,
				top: Val::ZERO,
				left: Val::ZERO,
				width: Val::Percent(50.0),
				height: Val::Percent(100.0),
				display: Display::Grid,
				grid_template_columns: vec![GridTrack::percent(50.0); 2],
				grid_template_rows: vec![GridTrack::percent(12.0); 8],
				padding: UiRect::axes(Val::Percent(5.0), Val::Percent(5.0)),
				align_items: AlignItems::Center,
				justify_content: JustifyContent::SpaceEvenly,
				..default()
			},
			image: UiImage {
				texture: menu,
				..default()
			},
			..default()
		})
		.with_children(|builder| {
			builder.spawn(
				TextBundle::from_section("Labirynt", text_style.clone())
					.with_style(elem_style(1, 1)),
			);

			for (i, kind) in [UiInput::Width, UiInput::Height, UiInput::Rooms]
				.into_iter()
				.enumerate()
			{
				builder.spawn(TextBundle {
					style: elem_style(1, 2 + i16::try_from(i).unwrap()),
					text: Text::from_section(kind.text(), text_style.clone()),
					..default()
				});

				builder.spawn((
					NodeBundle {
						style: elem_style(2, 2 + i16::try_from(i).unwrap()),
						..default()
					},
					TextInputBundle {
						text_style: TextInputTextStyle(text_style.clone()),
						settings: TextInputSettings {
							retain_on_submit: true,
							..default()
						},
						value: TextInputValue(kind.get(params)),
						inactive: TextInputInactive(true),
						..default()
					},
					kind,
				));
			}

			builder.spawn(TextBundle {
				style: elem_style(1, 5),
				text: Text::from_section("Typ", text_style.clone()),
				..default()
			});

			builder
				.spawn(NodeBundle {
					style: Style {
						display: Display::Flex,
						flex_direction: FlexDirection::Row,
						align_items: AlignItems::Center,
						justify_content: JustifyContent::SpaceBetween,
						..elem_style(2, 5)
					},
					..default()
				})
				.with_children(|builder| {
					use DirectionalBias::{
						Horizontal, None, Vertical, VeryHorizontal, VeryVertical,
					};

					for bias in [VeryHorizontal, Horizontal, None, Vertical, VeryVertical] {
						let selector = UiSelector(bias);

						builder
							.spawn((selector, ButtonBundle {
								style: Style {
									width: Val::Percent(20.0),
									aspect_ratio: Some(1.0),
									align_items: AlignItems::Center,
									justify_content: JustifyContent::Center,
									padding: UiRect::all(Val::Px(5.0)),
									border: UiRect::all(Val::Px(5.0)),
									..default()
								},
								background_color: BackgroundColor(Color::BLACK),
								border_color: BorderColor(INACTIVE_SELECTOR_COLOR),
								..default()
							}))
							.with_children(|parent| {
								let mut style = text_style.clone();
								style.font_size /= 2.0;

								parent.spawn(TextBundle::from_section(selector.text(), style));
							});
					}
				});

			builder
				.spawn((
					ButtonBundle {
						style: elem_style(1, 8),
						background_color: BackgroundColor(Color::BLACK),
						..default()
					},
					UiButton::Generate,
				))
				.with_children(|parent| {
					parent.spawn(TextBundle::from_section("Generuj", text_style.clone()));
				});

			if !cfg!(target_arch = "wasm32") {
				builder
					.spawn((
						ButtonBundle {
							style: elem_style(2, 8),
							background_color: BackgroundColor(Color::BLACK),
							..default()
						},
						UiButton::Close,
					))
					.with_children(|parent| {
						parent.spawn(TextBundle::from_section("Zamknij", text_style));
					});
			}
		})
		.id()
}
