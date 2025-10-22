use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    #[serde(default, rename = "devMode")]
    pub dev_mode: bool,
    pub assets: AssetConfig,
    pub colors: ColorConfig,
    pub ui: UiConfig,
    pub game: GameSettings,
    pub settings: UserSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetConfig {
    pub backgrounds: BackgroundAssets,
    pub icons: IconAssets,
    pub sounds: SoundAssets,
    pub animations: AnimationAssets,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundAssets {
    pub splash: String,
    pub main_menu: String,
    pub game: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconAssets {
    pub logo: String,
    pub volume_up: String,
    pub volume_down: String,
    pub settings: String,
    pub synthwave: SynthwaveIcons,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthwaveIcons {
    pub play: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundAssets {
    pub menu_theme: String,
    pub button_click: String,
    pub stone_place: String,
    pub game_win: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationAssets {
    pub main_menu_frames: FrameAnimation,
    pub transition_frames: FrameAnimation,
    pub game_background_frames: FrameAnimation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameAnimation {
    pub path_pattern: String,
    pub frame_count: u32,
    pub fps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorConfig {
    pub primary: ColorData,
    pub secondary: ColorData,
    pub accent: ColorData,
    pub background: ColorData,
    pub surface: ColorData,
    pub text_primary: ColorData,
    pub text_secondary: ColorData,
    pub button_normal: ColorData,
    pub button_hovered: ColorData,
    pub button_pressed: ColorData,
    pub stone_player1: ColorData,
    pub stone_player2: ColorData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorData {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<ColorData> for Color {
    fn from(color_data: ColorData) -> Self {
        Color::srgba(color_data.r, color_data.g, color_data.b, color_data.a)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub window_titles: Vec<String>,
    pub font_sizes: FontSizes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSizes {
    pub title: f32,
    pub subtitle: f32,
    pub button: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    pub board_size: u32,
    pub win_condition: u32,
    pub ai_max_depth: Option<u32>,  // None = unlimited depth
    pub ai_time_limit: Option<u64>, // Time limit in milliseconds, None = no time limit
    pub pair_captures_to_win: u32,
    #[serde(default)]
    pub ai_vs_ai_auto_play: bool,  // Auto-play for AI vs AI mode
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySettings {
    pub fullscreen: bool,
    pub vsync: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameplaySettings {
    pub show_move_hints: bool,
    pub animation_speed: f32,
    pub auto_save: bool,
    #[serde(default = "default_show_double_three_markers")]
    pub show_double_three_markers: bool,
}

fn default_show_double_three_markers() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub audio: AudioSettings,
    pub display: DisplaySettings,
    pub gameplay: GameplaySettings,
    pub theme: ThemeSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSettings {
    pub current_theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    pub volume: f32,
    pub muted: bool,
}

impl GameConfig {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = std::fs::read_to_string(path)?;
        let config: GameConfig = serde_json::from_str(&config_content)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let config_json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, config_json)?;
        Ok(())
    }



    pub fn get_animation_frame_path(&self, frame_number: u32) -> String {
        self.assets.animations.main_menu_frames.path_pattern
            .replace("{:04}", &format!("{:04}", frame_number))
    }

    // Save current audio settings to the config file
    pub fn save_audio_settings(&mut self, volume: f32, muted: bool) -> Result<(), Box<dyn std::error::Error>> {
        self.settings.audio.volume = volume;
        self.settings.audio.muted = muted;
        self.save_to_file("config/config.json")
    }

    // Get current audio settings
    pub fn get_audio_settings(&self) -> (f32, bool) {
        (self.settings.audio.volume, self.settings.audio.muted)
    }

    // Save display settings
    pub fn save_display_settings(&mut self, fullscreen: bool, vsync: bool) -> Result<(), Box<dyn std::error::Error>> {
        self.settings.display.fullscreen = fullscreen;
        self.settings.display.vsync = vsync;
        self.save_to_file("config/config.json")
    }

    // Get display settings
    pub fn get_display_settings(&self) -> (bool, bool) {
        (self.settings.display.fullscreen, self.settings.display.vsync)
    }

    // Save gameplay settings
    pub fn save_gameplay_settings(&mut self, show_move_hints: bool, animation_speed: f32, auto_save: bool) -> Result<(), Box<dyn std::error::Error>> {
        self.settings.gameplay.show_move_hints = show_move_hints;
        self.settings.gameplay.animation_speed = animation_speed;
        self.settings.gameplay.auto_save = auto_save;
        self.save_to_file("config/config.json")
    }

    // Get gameplay settings
    pub fn get_gameplay_settings(&self) -> (bool, f32, bool) {
        (self.settings.gameplay.show_move_hints, self.settings.gameplay.animation_speed, self.settings.gameplay.auto_save)
    }

    // Save double three markers visibility setting
    pub fn save_double_three_markers_visibility(&mut self, show: bool) -> Result<(), Box<dyn std::error::Error>> {
        self.settings.gameplay.show_double_three_markers = show;
        self.save_to_file("config/config.json")
    }

    // Get double three markers visibility setting
    pub fn get_double_three_markers_visibility(&self) -> bool {
        self.settings.gameplay.show_double_three_markers
    }

    // Save theme preference
    pub fn save_theme(&mut self, theme_name: String) -> Result<(), Box<dyn std::error::Error>> {
        self.settings.theme.current_theme = theme_name;
        self.save_to_file("config/config.json")
    }

    // Get current theme
    pub fn get_current_theme(&self) -> String {
        self.settings.theme.current_theme.clone()
    }

    // Save AI vs AI auto-play setting
    pub fn save_auto_play(&mut self, auto_play: bool) -> Result<(), Box<dyn std::error::Error>> {
        self.game.ai_vs_ai_auto_play = auto_play;
        self.save_to_file("config/config.json")
    }

    // Get AI vs AI auto-play setting
    pub fn get_auto_play(&self) -> bool {
        self.game.ai_vs_ai_auto_play
    }

    // Sync colors from theme manager
    pub fn sync_colors_from_theme(&mut self, theme_colors: &crate::ui::theme::ThemeColors) {
        self.colors.primary = ColorData {
            r: theme_colors.primary.r,
            g: theme_colors.primary.g,
            b: theme_colors.primary.b,
            a: theme_colors.primary.a,
        };
        self.colors.secondary = ColorData {
            r: theme_colors.secondary.r,
            g: theme_colors.secondary.g,
            b: theme_colors.secondary.b,
            a: theme_colors.secondary.a,
        };
        self.colors.accent = ColorData {
            r: theme_colors.accent.r,
            g: theme_colors.accent.g,
            b: theme_colors.accent.b,
            a: theme_colors.accent.a,
        };
        self.colors.background = ColorData {
            r: theme_colors.background.r,
            g: theme_colors.background.g,
            b: theme_colors.background.b,
            a: theme_colors.background.a,
        };
        self.colors.surface = ColorData {
            r: theme_colors.surface.r,
            g: theme_colors.surface.g,
            b: theme_colors.surface.b,
            a: theme_colors.surface.a,
        };
        self.colors.text_primary = ColorData {
            r: theme_colors.text_primary.r,
            g: theme_colors.text_primary.g,
            b: theme_colors.text_primary.b,
            a: theme_colors.text_primary.a,
        };
        self.colors.text_secondary = ColorData {
            r: theme_colors.text_secondary.r,
            g: theme_colors.text_secondary.g,
            b: theme_colors.text_secondary.b,
            a: theme_colors.text_secondary.a,
        };
        self.colors.button_normal = ColorData {
            r: theme_colors.button_normal.r,
            g: theme_colors.button_normal.g,
            b: theme_colors.button_normal.b,
            a: theme_colors.button_normal.a,
        };
        self.colors.button_hovered = ColorData {
            r: theme_colors.button_hovered.r,
            g: theme_colors.button_hovered.g,
            b: theme_colors.button_hovered.b,
            a: theme_colors.button_hovered.a,
        };
        self.colors.button_pressed = ColorData {
            r: theme_colors.button_pressed.r,
            g: theme_colors.button_pressed.g,
            b: theme_colors.button_pressed.b,
            a: theme_colors.button_pressed.a,
        };
        self.colors.stone_player1 = ColorData {
            r: theme_colors.stone_player1.r,
            g: theme_colors.stone_player1.g,
            b: theme_colors.stone_player1.b,
            a: theme_colors.stone_player1.a,
        };
        self.colors.stone_player2 = ColorData {
            r: theme_colors.stone_player2.r,
            g: theme_colors.stone_player2.g,
            b: theme_colors.stone_player2.b,
            a: theme_colors.stone_player2.a,
        };
    }

    // Save game settings
    pub fn save_game_settings(&mut self, board_size: u32, win_condition: u32, ai_max_depth: Option<u32>, ai_time_limit: Option<u64>, pair_captures_to_win: u32) -> Result<(), Box<dyn std::error::Error>> {
        self.game.board_size = board_size;
        self.game.win_condition = win_condition;
        self.game.ai_max_depth = ai_max_depth;
        self.game.ai_time_limit = ai_time_limit;
        self.game.pair_captures_to_win = pair_captures_to_win;
        
        let result = self.save_to_file("config/config.json");

        result
    }

    // Get game settings
    pub fn get_game_settings(&self) -> (u32, u32, Option<u32>, Option<u64>, u32) {
        (self.game.board_size, self.game.win_condition, self.game.ai_max_depth, self.game.ai_time_limit, self.game.pair_captures_to_win)
    }

    pub fn default() -> Self {
        Self {
            dev_mode: false,
            assets: AssetConfig {
                backgrounds: BackgroundAssets {
                    splash: "backgrounds/login/gomoku-splash.png".to_string(),
                    main_menu: "backgrounds/dolphin/frame_0001.png".to_string(),
                    game: "backgrounds/game-board.png".to_string(),
                },
                icons: IconAssets {
                    logo: "logo/gomoku-logo.png".to_string(),
                    volume_up: "icons/volume-up.png".to_string(),
                    volume_down: "icons/volume-down.png".to_string(),
                    settings: "icons/settings.png".to_string(),
                    synthwave: SynthwaveIcons {
                        play: "icons/synthwave/play_icon.png".to_string(),
                    },
                },
                sounds: SoundAssets {
                    menu_theme: "sound/menu-theme.mp3".to_string(),
                    button_click: "sound/button-click.wav".to_string(),
                    stone_place: "sound/stone-place.wav".to_string(),
                    game_win: "sound/victory.wav".to_string(),
                },
                animations: AnimationAssets {
                    main_menu_frames: FrameAnimation {
                        path_pattern: "backgrounds/dolphin/frame_{:04}.png".to_string(),
                        frame_count: 120,
                        fps: 15,
                    },
                    transition_frames: FrameAnimation {
                        path_pattern: "transitions/frames/frame_{:04}.png".to_string(),
                        frame_count: 300,
                        fps: 30,
                    },
                    game_background_frames: FrameAnimation {
                        path_pattern: "backgrounds/ingame-background/frame_{:04}.jpg".to_string(),
                        frame_count: 600,
                        fps: 30,
                    },
                },
            },
            colors: ColorConfig {
                primary: ColorData { r: 1.0, g: 0.0, b: 1.0, a: 1.0 },
                secondary: ColorData { r: 0.0, g: 1.0, b: 1.0, a: 1.0 },
                accent: ColorData { r: 1.0, g: 0.2, b: 0.8, a: 1.0 },
                background: ColorData { r: 0.05, g: 0.0, b: 0.15, a: 1.0 },
                surface: ColorData { r: 0.1, g: 0.0, b: 0.2, a: 1.0 },
                text_primary: ColorData { r: 0.0, g: 1.0, b: 1.0, a: 1.0 },
                text_secondary: ColorData { r: 1.0, g: 0.0, b: 1.0, a: 1.0 },
                button_normal: ColorData { r: 0.2, g: 0.0, b: 0.4, a: 1.0 },
                button_hovered: ColorData { r: 0.4, g: 0.0, b: 0.6, a: 1.0 },
                button_pressed: ColorData { r: 1.0, g: 0.2, b: 0.8, a: 1.0 },
                stone_player1: ColorData { r: 1.0, g: 0.4, b: 0.7, a: 1.0 }, // Pink
                stone_player2: ColorData { r: 0.2, g: 0.6, b: 1.0, a: 1.0 }, // Blue
            },
            ui: UiConfig {
                window_titles: vec![
                    "Gomoku: Now with 100% more stones!".to_string(),
                    "Gomoku: Still better than your code!".to_string(),
                    "Gomoku: Five in a row or go home!".to_string(),
                ],
                font_sizes: FontSizes {
                    title: 32.0,
                    subtitle: 18.0,
                    button: 28.0,
                },
            },
            game: GameSettings {
                board_size: 15,
                win_condition: 5,
                ai_max_depth: Some(4),      // Default depth of 4
                ai_time_limit: Some(1000),  // Default 1 second time limit
                pair_captures_to_win: 10,
                ai_vs_ai_auto_play: false,  // Default auto-play off
            },
            settings: UserSettings {
                audio: AudioSettings {
                    volume: 0.5,
                    muted: false,
                },
                display: DisplaySettings {
                    fullscreen: false,
                    vsync: true,
                },
                gameplay: GameplaySettings {
                    show_move_hints: true,
                    animation_speed: 1.0,
                    auto_save: true,
                    show_double_three_markers: true,
                },
                theme: ThemeSettings {
                    current_theme: "Synthwave".to_string(),
                },
            },
        }
    }
}

// Component to mark UI elements for config updates
#[derive(Component)]
pub struct ConfigElement {
    pub element_type: ConfigElementType,
}

#[derive(Debug, Clone)]
pub enum ConfigElementType {
    ButtonNormal,
    ButtonHovered,
    ButtonPressed,
    TextPrimary,
    TextSecondary,
    Surface,
    Background,
}

// System to update config elements when config changes
pub fn update_config_elements(
    config: Res<GameConfig>,
    mut query: Query<(&ConfigElement, &mut BackgroundColor)>,
    mut text_query: Query<(&ConfigElement, &mut TextColor), Without<BackgroundColor>>,
) {
    if !config.is_changed() {
        return;
    }

    // Update background colors
    for (config_element, mut bg_color) in query.iter_mut() {
        let new_color = match config_element.element_type {
            ConfigElementType::ButtonNormal => config.colors.button_normal.clone().into(),
            ConfigElementType::ButtonHovered => config.colors.button_hovered.clone().into(),
            ConfigElementType::ButtonPressed => config.colors.button_pressed.clone().into(),
            ConfigElementType::Surface => config.colors.surface.clone().into(),
            ConfigElementType::Background => config.colors.background.clone().into(),
            _ => continue,
        };
        *bg_color = BackgroundColor(new_color);
    }

    // Update text colors
    for (config_element, mut text_color) in text_query.iter_mut() {
        let new_color = match config_element.element_type {
            ConfigElementType::TextPrimary => config.colors.text_primary.clone().into(),
            ConfigElementType::TextSecondary => config.colors.text_secondary.clone().into(),
            _ => continue,
        };
        *text_color = TextColor(new_color);
    }
}

// Plugin to initialize the config system
pub fn config_plugin(app: &mut App) {
    // Don't reload config here - it's already loaded in init_resources
    // This plugin just adds the update system
    app.add_systems(Update, update_config_elements);
}
