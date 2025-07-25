use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
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
    pub loading_messages: HashMap<String, String>,
    pub font_sizes: FontSizes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSizes {
    pub title: f32,
    pub subtitle: f32,
    pub button: f32,
    pub loading: f32,
    pub percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    pub board_size: u32,
    pub win_condition: u32,
    pub ai_difficulty: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub audio: AudioSettings,
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

    pub fn get_loading_message(&self, percentage: u32) -> String {
        let key = match percentage {
            0..=10 => "0-10",
            11..=25 => "11-25", 
            26..=40 => "26-40",
            41..=60 => "41-60",
            61..=80 => "61-80",
            81..=95 => "81-95",
            96..=99 => "96-99",
            _ => "100",
        };
        
        self.ui.loading_messages
            .get(key)
            .cloned()
            .unwrap_or_else(|| "Loading...".to_string())
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

    pub fn default() -> Self {
        Self {
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
            },
            ui: UiConfig {
                loading_messages: {
                    let mut messages = HashMap::new();
                    messages.insert("0-10".to_string(), "Initializing...".to_string());
                    messages.insert("11-25".to_string(), "Starting motors...".to_string());
                    messages.insert("26-40".to_string(), "Loading assets...".to_string());
                    messages.insert("41-60".to_string(), "Looking for neo-cat...".to_string());
                    messages.insert("61-80".to_string(), "Smelling good code...".to_string());
                    messages.insert("81-95".to_string(), "Finalizing...".to_string());
                    messages.insert("96-99".to_string(), "Almost ready...".to_string());
                    messages.insert("100".to_string(), "Complete!".to_string());
                    messages
                },
                font_sizes: FontSizes {
                    title: 32.0,
                    subtitle: 18.0,
                    button: 28.0,
                    loading: 20.0,
                    percentage: 16.0,
                },
            },
            game: GameSettings {
                board_size: 15,
                win_condition: 5,
                ai_difficulty: "medium".to_string(),
            },
            settings: UserSettings {
                audio: AudioSettings {
                    volume: 0.5,
                    muted: false,
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
    let config = GameConfig::load_from_file("config/config.json")
        .unwrap_or_else(|_| {
            println!("Could not load config.json, using default config");
            GameConfig::default()
        });
    
    app.insert_resource(config)
        .add_systems(Update, update_config_elements);
}
