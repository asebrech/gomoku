use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Resource, Debug, Clone)]
pub struct ThemeManager {
    pub current_theme: GameTheme,
    pub available_themes: HashMap<String, GameTheme>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameTheme {
    pub name: String,
    pub colors: ThemeColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
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
    pub title_background: ColorData,
    pub content_background: ColorData,
    pub footer_background: ColorData,
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

impl ThemeManager {
    pub fn new() -> Self {
        Self::with_theme("Synthwave")
    }

    pub fn with_theme(theme_name: &str) -> Self {
        let mut manager = Self {
            current_theme: Self::default_synthwave_theme(),
            available_themes: HashMap::new(),
        };
        
        // Add default themes as fallbacks
        let synthwave_theme = Self::default_synthwave_theme();
        let blackwhite_theme = Self::default_blackwhite_theme();
        
        manager.available_themes.insert(synthwave_theme.name.clone(), synthwave_theme);
        manager.available_themes.insert(blackwhite_theme.name.clone(), blackwhite_theme);
        
        // Dynamically load all themes from the themes folder
        if let Err(e) = manager.load_all_themes_from_folder("assets/themes") {
            eprintln!("Warning: Could not load themes from folder: {}", e);
        }
        
        // Set the requested theme
        manager.set_theme(theme_name);
        
        manager
    }

    /// Dynamically loads all .json files from the specified folder as themes
    pub fn load_all_themes_from_folder(&mut self, folder_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Read the directory
        let entries = std::fs::read_dir(folder_path)?;
        
        let mut loaded_count = 0;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            // Only process .json files
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_theme_from_path(&path) {
                    Ok(theme_name) => {
                        loaded_count += 1;
                        println!("Loaded theme: {} from {:?}", theme_name, path);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to load theme from {:?}: {}", path, e);
                    }
                }
            }
        }
        
        println!("Successfully loaded {} theme(s) from {}", loaded_count, folder_path);
        Ok(())
    }

    /// Loads a theme from a file path, using the theme's "name" field from the JSON
    fn load_theme_from_path(&mut self, path: &std::path::Path) -> Result<String, Box<dyn std::error::Error>> {
        let theme_content = std::fs::read_to_string(path)?;
        let theme: GameTheme = serde_json::from_str(&theme_content)?;
        let theme_name = theme.name.clone();
        self.available_themes.insert(theme_name.clone(), theme);
        Ok(theme_name)
    }

    pub fn set_theme(&mut self, theme_name: &str) -> bool {
        if let Some(theme) = self.available_themes.get(theme_name) {
            self.current_theme = theme.clone();
            true
        } else {
            false
        }
    }

    pub fn get_available_themes(&self) -> Vec<String> {
        self.available_themes.keys().cloned().collect()
    }

    fn default_synthwave_theme() -> GameTheme {
        GameTheme {
            name: "Synthwave".to_string(),
            colors: ThemeColors {
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
                title_background: ColorData { r: 0.1, g: 0.0, b: 0.2, a: 0.8 },
                content_background: ColorData { r: 0.2, g: 0.0, b: 0.4, a: 0.6 },
                footer_background: ColorData { r: 0.3, g: 0.0, b: 0.6, a: 0.7 },
                stone_player1: ColorData { r: 1.0, g: 0.4, b: 0.7, a: 1.0 }, // Pink
                stone_player2: ColorData { r: 0.2, g: 0.6, b: 1.0, a: 1.0 }, // Blue
            },
        }
    }

    fn default_blackwhite_theme() -> GameTheme {
        GameTheme {
            name: "Black & White".to_string(),
            colors: ThemeColors {
                primary: ColorData { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
                secondary: ColorData { r: 0.8, g: 0.8, b: 0.8, a: 1.0 },
                accent: ColorData { r: 0.5, g: 0.5, b: 0.5, a: 1.0 },
                background: ColorData { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
                surface: ColorData { r: 0.1, g: 0.1, b: 0.1, a: 1.0 },
                text_primary: ColorData { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
                text_secondary: ColorData { r: 0.7, g: 0.7, b: 0.7, a: 1.0 },
                button_normal: ColorData { r: 0.2, g: 0.2, b: 0.2, a: 1.0 },
                button_hovered: ColorData { r: 0.4, g: 0.4, b: 0.4, a: 1.0 },
                button_pressed: ColorData { r: 0.6, g: 0.6, b: 0.6, a: 1.0 },
                title_background: ColorData { r: 0.15, g: 0.15, b: 0.15, a: 0.9 },
                content_background: ColorData { r: 0.1, g: 0.1, b: 0.1, a: 0.8 },
                footer_background: ColorData { r: 0.05, g: 0.05, b: 0.05, a: 0.85 },
                stone_player1: ColorData { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }, // White
                stone_player2: ColorData { r: 0.2, g: 0.2, b: 0.2, a: 1.0 }, // Dark gray
            },
        }
    }
}

// Component to mark UI elements for theme updates
#[derive(Component)]
pub struct ThemeElement {
    pub element_type: ThemeElementType,
}

#[derive(Debug, Clone)]
pub enum ThemeElementType {
    ButtonNormal,
    ButtonHovered,
    ButtonPressed,
    TextPrimary,
    TextSecondary,
    TitleBackground,
    ContentBackground,
    FooterBackground,
    Surface,
    Background,
}

// System to update theme elements when theme changes
pub fn update_theme_elements(
    theme_manager: Res<ThemeManager>,
    mut query: Query<(&ThemeElement, &mut BackgroundColor)>,
    mut text_query: Query<(&ThemeElement, &mut TextColor), Without<BackgroundColor>>,
) {
    if !theme_manager.is_changed() {
        return;
    }

    let theme = &theme_manager.current_theme;

    // Update background colors
    for (theme_element, mut bg_color) in query.iter_mut() {
        let new_color = match theme_element.element_type {
            ThemeElementType::ButtonNormal => theme.colors.button_normal.clone().into(),
            ThemeElementType::ButtonHovered => theme.colors.button_hovered.clone().into(),
            ThemeElementType::ButtonPressed => theme.colors.button_pressed.clone().into(),
            ThemeElementType::TitleBackground => theme.colors.title_background.clone().into(),
            ThemeElementType::ContentBackground => theme.colors.content_background.clone().into(),
            ThemeElementType::FooterBackground => theme.colors.footer_background.clone().into(),
            ThemeElementType::Surface => theme.colors.surface.clone().into(),
            ThemeElementType::Background => theme.colors.background.clone().into(),
            _ => continue,
        };
        *bg_color = BackgroundColor(new_color);
    }

    // Update text colors
    for (theme_element, mut text_color) in text_query.iter_mut() {
        let new_color = match theme_element.element_type {
            ThemeElementType::TextPrimary => theme.colors.text_primary.clone().into(),
            ThemeElementType::TextSecondary => theme.colors.text_secondary.clone().into(),
            _ => continue,
        };
        *text_color = TextColor(new_color);
    }
}

// Plugin to initialize the theme system
pub fn theme_plugin(app: &mut App) {
    app.insert_resource(ThemeManager::new())
        .add_systems(Update, update_theme_elements);
}
