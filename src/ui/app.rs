use bevy::prelude::*;
use bevy::window::{PresentMode, WindowTheme, WindowMode, MonitorSelection, WindowResized};
use bevy_gstreamer::GstreamerPlugin;

use crate::core::state::GameState;
use crate::ai::transposition::TranspositionTable;
use crate::ai::config::AIConfig;
use crate::audio::audio_plugin;
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
	pub ai_vs_ai: bool, //if both players are AI
	pub time_limit: Option<usize>, // time limit in milliseconds, optional
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
			ai_vs_ai: false,
			time_limit: Some(500), // 500ms time limit for AI by default
		}
	}
}

fn setup(mut commands: Commands) {
	commands.spawn(Camera2d);
}

fn maintain_aspect_ratio(
	mut resize_events: EventReader<WindowResized>,
	_windows: Query<&mut Window>,
	mut last_size: Local<Option<(f32, f32)>>,
) {
	// Disable aspect ratio maintenance to prevent UI coordinate issues
	// This was causing mouse click misalignment with video backgrounds
	
	// Just track the size changes without forcing aspect ratio
	for event in resize_events.read() {
		*last_size = Some((event.width, event.height));
	}
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
		// Load display settings from config
		let config = GameConfig::load_from_file("config/config.json")
			.unwrap_or_else(|_| GameConfig::default());
		let (fullscreen, _vsync) = config.get_display_settings();
		
		// Get a random window title
		use rand::Rng;
		let window_title = if !config.ui.window_titles.is_empty() {
			let idx = rand::rng().random_range(0..config.ui.window_titles.len());
			config.ui.window_titles[idx].clone()
		} else {
			"Gomoku".to_string()
		};
		
		// Get the executable's directory and navigate to project root, then to assets folder
		let asset_path = if cfg!(debug_assertions) {
			// In debug mode, use assets folder in current directory
			"assets".to_string()
		} else {
			// In release mode, executable is in target/release/, so go up two levels to project root, then into assets
			std::env::current_exe()
				.ok()
				.and_then(|exe_path| {
					exe_path.parent() // target/release
						.and_then(|p| p.parent()) // target
						.and_then(|p| p.parent()) // project root
						.map(|p| p.join("assets").to_string_lossy().into_owned())
				})
				.unwrap_or_else(|| "assets".to_string())
		};
		
		self.app.add_plugins((
            DefaultPlugins
				.set(AssetPlugin {
					file_path: asset_path,
					..default()
				})
				.set(WindowPlugin {
                primary_window: Some(Window {
                    title: window_title.into(),
                    name: Some("bevy.app".into()),
                    resolution: (1600., 900.).into(), // 16:9 aspect ratio
                    present_mode: PresentMode::AutoVsync,
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    window_theme: Some(WindowTheme::Dark),
                    enabled_buttons: bevy::window::EnabledButtons {
                        maximize: true,
                        ..Default::default()
                    },
                    visible: false,
                    resizable: true, // Allow user to resize
                    mode: if fullscreen {
                        WindowMode::BorderlessFullscreen(MonitorSelection::Current)
                    } else {
                        WindowMode::Windowed
                    },
                    ..default()
                }),
                ..default()
            }),
        ));
	}

	fn init_resources(&mut self) {
		// Load game settings from config
		let mut config = GameConfig::load_from_file("config/config.json")
			.unwrap_or_else(|_| GameConfig::default());
		
		// Load AI configuration
		let ai_config = AIConfig::load_or_default("config/ai_config.toml");
		
		let (board_size, win_condition, ai_max_depth, ai_time_limit, pair_captures_to_win) = config.get_game_settings();
		// Use the configured values directly
		let ai_depth = ai_max_depth.unwrap_or(6) as i32; // Default to 6 if unlimited
		let time_limit = ai_time_limit.map(|t| t as usize); // Convert u64 to usize
		let settings = GameSettings {
			board_size: board_size as usize,
			total_capture_to_win: pair_captures_to_win as usize,
			minimum_chain_to_win: win_condition as usize,
			ai_depth,
			alpha_beta_enabled: true,
			versus_ai: true,
			ai_vs_ai: false,
			time_limit,
		};
		
		// Get the saved theme from config
		let current_theme = config.get_current_theme();
		
		// Create theme manager with saved theme
		let theme_manager = ThemeManager::with_theme(&current_theme);
		
		// Sync config colors from theme
		config.sync_colors_from_theme(&theme_manager.current_theme.colors);
		
		self.app
		.insert_resource(GameState::new(settings.board_size, settings.minimum_chain_to_win))
        .insert_resource(settings)
        .insert_resource(config)
        .insert_resource(ai_config)
        .insert_resource(theme_manager)
        .init_resource::<TranspositionTable>();

	}

	fn init_plugins(&mut self) {
		self.app.init_state::<AppState>();
		let config = self.app.world().get_resource::<GameConfig>()
			.expect("GameConfig should be initialized before plugins");
		
		if config.dev_mode {
			self.app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::Menu);
		}
		
		self.app
        .add_systems(Startup, setup)
		        .add_systems(
            Update,
            (
                make_visible,
                maintain_aspect_ratio,
            ),
        )
        .add_plugins((
            GstreamerPlugin,
            splash_plugin, 
            menu_plugin, 
            game_plugin, 
            tutorial_plugin, 
            config_plugin, 
            audio_plugin
        ));
	}

	pub fn start(&mut self) {
		println!("Gomoku App Started.");
		self.app.run();
	}
}