use bevy::prelude::*;
use bevy::window::{PresentMode, WindowTheme};

use crate::core::state::GameState;
use crate::ai::transposition::TranspositionTable;
use crate::ui::display::display::make_visible;
use crate::ui::screens::game::game::game_plugin;
use crate::ui::screens::menu::menu_plugin;
use crate::ui::screens::splash::splash_plugin;
use crate::ui::screens::tutorial::tutorial_plugin;
use crate::ui::config::{config_plugin, GameConfig};
use crate::ui::theme::ThemeManager;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
	#[default]
	Splash,
	Menu,
	GameOptions,
	Game,
	HowToPlay,
	Credit
}

#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Copy)]
pub struct GameSettings {
	pub board_size: usize, //default to 19
	pub total_capture_to_win: usize, //default to 10
	pub minimum_chain_to_win: usize, //5 pallet 
	pub ai_depth: i32, //default to 2
	pub alpha_beta_enabled: bool, //wether deep checking is enabled or not
	pub versus_ai: bool, //if the user is against an AI or multiplayer
	pub time_limit: Option<usize> // time limit in milliseconds, optional
}

impl GameSettings {
	pub fn new() -> Self {
		GameSettings {
			board_size: 19,
			total_capture_to_win: 10,
			minimum_chain_to_win: 5,
			ai_depth: 10, // Increased since iterative deepening can handle higher depths
			alpha_beta_enabled: true,
			versus_ai: true,
			time_limit: Some(500) // 1000ms (1 second) time limit for AI by default
		}
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
                    resolution: (1600., 1000.).into(),
                    present_mode: PresentMode::AutoVsync,
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    window_theme: Some(WindowTheme::Dark),
                    enabled_buttons: bevy::window::EnabledButtons {
                        maximize: false,
                        ..Default::default()
                    },
                    visible: false,
                    ..default()
                }),
                ..default()
            }),
        ));
	}

	fn init_resources(&mut self) {
		// Load game settings from config
		let config = GameConfig::load_from_file("config/config.json")
			.unwrap_or_else(|_| GameConfig::default());
		let (board_size, win_condition, ai_difficulty, pair_captures_to_win) = config.get_game_settings();
		
		// Map AI difficulty to actual values
		let (ai_depth, time_limit) = match ai_difficulty.to_lowercase().as_str() {
			"easy" => (2, Some(300)),     // Depth 2, 300ms
			"medium" => (4, Some(800)),   // Depth 4, 800ms  
			"hard" => (6, Some(1500)),    // Depth 6, 1500ms
			_ => (4, Some(800)),          // Default to medium
		};
		
		let settings = GameSettings {
			board_size: board_size as usize,
			total_capture_to_win: pair_captures_to_win as usize,
			minimum_chain_to_win: win_condition as usize,
			ai_depth,
			alpha_beta_enabled: true,
			versus_ai: true,
			time_limit,
		};
		
		self.app
		.insert_resource(GameState::new(settings.board_size, settings.minimum_chain_to_win, settings.total_capture_to_win))
        .insert_resource(settings)
        .insert_resource(config)
        .insert_resource(ThemeManager::new())
        .init_resource::<TranspositionTable>();

	}

	fn init_plugins(&mut self) {
		self.app
        .init_state::<AppState>()
        .add_systems(Startup, setup)
		        .add_systems(
            Update,
            (
                make_visible,
            ),
        )
        .add_plugins((splash_plugin, menu_plugin, game_plugin, tutorial_plugin, config_plugin));
	}

	pub fn start(&mut self) {
		println!("Gomoku App Started.");
		self.app.run();
	}
}