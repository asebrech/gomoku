use bevy::prelude::*;

use bevy::window::{PresentMode, WindowTheme};
use bevy::color::palettes::css::CRIMSON;

use crate::core::state::GameState;
use crate::ui::display::display::make_visible;
use crate::ui::screens::game::game::game_plugin;
use crate::ui::screens::menu::menu_plugin;
use crate::ui::screens::splash::splash_plugin;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum AppState {
	#[default]
	Splash,
	Menu,
	GameOptions,
	Game,
	Credit
}

#[derive(Resource, Debug, Component, PartialEq, Clone, Copy)]
struct ColorScheme {
	pub button_text_color: Color,
	pub button_background_color: Color,
	title_text_color: Srgba,
}

#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Copy)]
pub struct GameSettings {
	pub board_size: usize, //default to 19
	pub total_capture_to_win: usize, //default to 10
	pub minimum_chain_to_win: usize, //5 pallet 
	pub ai_depth: i32, //default to 2
	pub alpha_beta_enabled: bool, //wether deep checking is enabled or not
	pub versus_ai: bool, //if the user is against an AI or multiplayer
	pub time_limit: Option<usize> // time limit in seconds, optional
}

impl GameSettings {
	pub fn new() -> Self {
		GameSettings {
			board_size: 19,
			total_capture_to_win: 10,
			minimum_chain_to_win: 5,
			ai_depth: 5,
			alpha_beta_enabled: true,
			versus_ai: true,
			time_limit: None
		}
	}
}


impl ColorScheme {
	pub fn new() -> Self {
		let button_text_color = Color::srgb(0.9, 0.9, 0.9);
		let button_background_color = Color::BLACK;
		let title_text_color = CRIMSON;
		ColorScheme { button_text_color, button_background_color, title_text_color }
	}
}

fn setup(mut commands: Commands) {
	commands.spawn(Camera2d);
}


pub struct GomokuApp {
	pub app: App,
}

impl GomokuApp {
	pub fn new() -> Self {
		println!("Initializing Gomoku App");
		let app = App::new();
		GomokuApp { app }
	}

	pub fn init(&mut self) {
		self.init_window();
		self.init_resources();
		self.init_plugins();
	}

	fn init_window(&mut self) {
		self.app.add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "I am a window!".into(),
                    name: Some("bevy.app".into()),
                    resolution: (1240., 720.).into(),
                    present_mode: PresentMode::AutoVsync,
                    // Tells Wasm to resize the window according to the available canvas
                    fit_canvas_to_parent: true,
                    // Tells Wasm not to override default event handling, like F5, Ctrl+R etc.
                    prevent_default_event_handling: false,
                    window_theme: Some(WindowTheme::Dark),
                    enabled_buttons: bevy::window::EnabledButtons {
                        maximize: false,
                        ..Default::default()
                    },
                    // This will spawn an invisible window
                    // The window will be made visible in the make_visible() system after 3 frames.
                    // This is useful when you want to avoid the white window that shows up before the GPU is ready to render the app.
                    visible: false,
                    ..default()
                }),
                ..default()
            }),
            //LogDiagnosticsPlugin::default(),
            //FrameTimeDiagnosticsPlugin::default(),
        ));
	}

	fn init_resources(&mut self) {
		let settings = GameSettings::new();
		self.app
		.insert_resource(GameState::new(settings.board_size, settings.minimum_chain_to_win))
        .insert_resource(settings)
        .insert_resource(ColorScheme::new());

	}

	fn init_plugins(&mut self) {
		self.app
        // Declare the game state, whose starting value is determined by the `Default` trait
        .init_state::<AppState>()
        .add_systems(Startup, setup)
		        .add_systems(
            Update,
            (
                make_visible,
            ),
        )
        // Adds the plugins for each state
        .add_plugins((splash_plugin, menu_plugin, game_plugin));
	}

	pub fn start(&mut self) {
		println!("Gomoku App Started.");
		self.app.run();
	}
}