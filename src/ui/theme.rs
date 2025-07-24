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
        let mut manager = Self {
            current_theme: Self::default_synthwave_theme(),
            available_themes: HashMap::new(),
        };
        
        // Add default themes
        let dark_theme = Self::default_dark_theme();
        let synthwave_theme = Self::default_synthwave_theme();
        
        manager.available_themes.insert(dark_theme.name.clone(), dark_theme.clone());
        manager.available_themes.insert(synthwave_theme.name.clone(), synthwave_theme);
        
        // Try to load themes from files
        let _ = manager.load_theme_from_file("Dark", "assets/themes/dark_theme.json");
        let _ = manager.load_theme_from_file("Synthwave", "assets/themes/synthwave_theme.json");
        
        manager
    }

    pub fn load_theme_from_file(&mut self, theme_name: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let theme_content = std::fs::read_to_string(file_path)?;
        let theme: GameTheme = serde_json::from_str(&theme_content)?;
        self.available_themes.insert(theme_name.to_string(), theme);
        Ok(())
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

    fn default_dark_theme() -> GameTheme {
        GameTheme {
            name: "Dark".to_string(),
            colors: ThemeColors {
                primary: ColorData { r: 0.15, g: 0.15, b: 0.15, a: 1.0 },
                secondary: ColorData { r: 0.25, g: 0.25, b: 0.25, a: 1.0 },
                accent: ColorData { r: 0.35, g: 0.75, b: 0.35, a: 1.0 },
                background: ColorData { r: 0.1, g: 0.1, b: 0.1, a: 1.0 },
                surface: ColorData { r: 0.2, g: 0.2, b: 0.2, a: 1.0 },
                text_primary: ColorData { r: 0.9, g: 0.9, b: 0.9, a: 1.0 },
                text_secondary: ColorData { r: 0.7, g: 0.7, b: 0.7, a: 1.0 },
                button_normal: ColorData { r: 0.15, g: 0.15, b: 0.15, a: 1.0 },
                button_hovered: ColorData { r: 0.25, g: 0.25, b: 0.25, a: 1.0 },
                button_pressed: ColorData { r: 0.35, g: 0.75, b: 0.35, a: 1.0 },
                title_background: ColorData { r: 0.05, g: 0.1, b: 0.2, a: 1.0 },
                content_background: ColorData { r: 0.3, g: 0.05, b: 0.05, a: 1.0 },
                footer_background: ColorData { r: 0.2, g: 0.0, b: 0.4, a: 1.0 },
            },
        }
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
