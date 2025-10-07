use bevy::prelude::*;
use crate::ui::config::ColorConfig;

/// Button style variants
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Success,
    Danger,
    Small,
}

/// Button size presets
#[derive(Clone, Copy, Debug)]
pub enum ButtonSize {
    Small,      // 25x25px - for +/- controls
    Medium,     // 100x30px - for cycle buttons
    Large,      // 280x60px - for main menu
    ExtraLarge, // 300x60px - for game over screen
    Full,       // 100% width, custom height
    Custom(f32, f32), // Custom width, height in pixels
}

impl ButtonSize {
    pub fn to_size(&self) -> (Val, Val) {
        match self {
            ButtonSize::Small => (Val::Px(25.0), Val::Px(25.0)),
            ButtonSize::Medium => (Val::Px(100.0), Val::Px(30.0)),
            ButtonSize::Large => (Val::Px(280.0), Val::Px(60.0)),
            ButtonSize::ExtraLarge => (Val::Px(300.0), Val::Px(60.0)),
            ButtonSize::Full => (Val::Percent(100.0), Val::Px(40.0)),
            ButtonSize::Custom(w, h) => (Val::Px(*w), Val::Px(*h)),
        }
    }
}

/// Builder for creating consistent buttons across the app
pub struct ButtonBuilder {
    pub text: String,
    pub style: ButtonStyle,
    pub size: ButtonSize,
    pub icon: Option<Handle<Image>>,
    pub border_width: f32,
    pub border_radius: f32,
    pub font_size: f32,
    pub margin: UiRect,
}

impl Default for ButtonBuilder {
    fn default() -> Self {
        Self {
            text: String::new(),
            style: ButtonStyle::Secondary,
            size: ButtonSize::Large,
            icon: None,
            border_width: 2.0,
            border_radius: 4.0,
            font_size: 18.0,
            margin: UiRect::all(Val::Px(0.0)),
        }
    }
}

impl ButtonBuilder {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Default::default()
        }
    }

    pub fn with_style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn with_icon(mut self, icon: Handle<Image>) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    pub fn with_margin(mut self, margin: UiRect) -> Self {
        self.margin = margin;
        self
    }

    pub fn with_border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    /// Get the background and border colors for the button style
    fn get_colors(&self, colors: &ColorConfig) -> (crate::ui::config::ColorData, crate::ui::config::ColorData) {
        match self.style {
            ButtonStyle::Primary => (colors.accent.clone(), colors.accent.clone()),
            ButtonStyle::Secondary => (colors.button_normal.clone(), colors.secondary.clone()),
            ButtonStyle::Success => {
                (
                    crate::ui::config::ColorData { r: 0.3, g: 0.5, b: 0.3, a: 1.0 },
                    crate::ui::config::ColorData { r: 0.4, g: 0.6, b: 0.4, a: 1.0 },
                )
            }
            ButtonStyle::Danger => {
                (
                    crate::ui::config::ColorData { r: 0.5, g: 0.3, b: 0.3, a: 1.0 },
                    crate::ui::config::ColorData { r: 0.6, g: 0.4, b: 0.4, a: 1.0 },
                )
            }
            ButtonStyle::Small => (colors.button_normal.clone(), colors.secondary.clone()),
        }
    }

    /// Spawn the button with the given marker component
    pub fn spawn<T: Component>(
        self,
        parent: &mut ChildSpawnerCommands,
        marker: T,
        colors: &ColorConfig,
    ) {
        let (bg_color, border_color) = self.get_colors(colors);
        let (width, height) = self.size.to_size();

        parent
            .spawn((
                Button,
                Node {
                    width,
                    height,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(self.border_width)),
                    margin: self.margin,
                    ..default()
                },
                BackgroundColor(bg_color.into()),
                BorderColor(border_color.into()),
                BorderRadius::all(Val::Px(self.border_radius)),
                marker,
            ))
            .with_children(|parent| {
                // Container for icon + text
                if self.icon.is_some() {
                    parent
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                column_gap: Val::Px(8.0),
                                ..default()
                            },
                        ))
                        .with_children(|parent| {
                            // Icon
                            if let Some(icon) = &self.icon {
                                parent.spawn((
                                    ImageNode::new(icon.clone()),
                                    Node {
                                        width: Val::Px(24.0),
                                        height: Val::Px(24.0),
                                        ..default()
                                    },
                                ));
                            }
                            
                            // Text
                            parent.spawn((
                                Text::new(&self.text),
                                TextFont {
                                    font_size: self.font_size,
                                    ..default()
                                },
                                TextColor(colors.text_primary.clone().into()),
                            ));
                        });
                } else {
                    // Just text
                    parent.spawn((
                        Text::new(&self.text),
                        TextFont {
                            font_size: self.font_size,
                            ..default()
                        },
                        TextColor(colors.text_primary.clone().into()),
                    ));
                }
            });
    }
}

/// Generic button interaction system
/// This handles the hover/press/normal states for all buttons
pub fn button_interaction_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor, Option<&ButtonStateOverride>),
        (Changed<Interaction>, With<Button>),
    >,
    config: Res<crate::ui::config::GameConfig>,
) {
    let colors = &config.colors;
    
    for (interaction, mut background_color, mut border_color, state_override) in &mut interaction_query {
        // Skip if there's a state override (for toggle buttons, etc.)
        if state_override.is_some() {
            continue;
        }

        match *interaction {
            Interaction::Pressed => {
                *background_color = BackgroundColor(colors.accent.clone().into());
                *border_color = BorderColor(colors.accent.clone().into());
            }
            Interaction::Hovered => {
                *background_color = BackgroundColor(colors.button_hovered.clone().into());
                *border_color = BorderColor(colors.secondary.clone().into());
            }
            Interaction::None => {
                *background_color = BackgroundColor(colors.button_normal.clone().into());
                *border_color = BorderColor(colors.secondary.clone().into());
            }
        }
    }
}

/// Marker component to indicate that a button has custom state management
/// and should not be affected by the generic button_interaction_system
#[derive(Component)]
pub struct ButtonStateOverride;

/// Helper function to create a simple icon-only button (like +/-)
pub fn spawn_icon_button<T: Component>(
    parent: &mut ChildSpawnerCommands,
    icon_text: &str,
    marker: T,
    colors: &ColorConfig,
) {
    ButtonBuilder::new(icon_text)
        .with_style(ButtonStyle::Small)
        .with_size(ButtonSize::Small)
        .with_font_size(14.0)
        .with_border_radius(4.0)
        .spawn(parent, marker, colors);
}

/// Helper function to create a toggle button (for settings)
pub fn spawn_toggle_button<T: Component>(
    parent: &mut ChildSpawnerCommands,
    enabled: bool,
    marker: T,
    colors: &ColorConfig,
) {
    let (width, height) = ButtonSize::Full.to_size();
    let bg_color = if enabled {
        colors.accent.clone()
    } else {
        colors.button_normal.clone()
    };

    parent
        .spawn((
            Button,
            Node {
                width,
                height: Val::Px(30.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(bg_color.into()),
            BorderColor(colors.secondary.clone().into()),
            ButtonStateOverride, // This button manages its own state
            marker,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(if enabled { "ON" } else { "OFF" }),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(colors.text_primary.clone().into()),
            ));
        });
}

/// Helper for volume control buttons
pub fn spawn_volume_button<T: Component>(
    parent: &mut ChildSpawnerCommands,
    icon: &str,
    marker: T,
    colors: &ColorConfig,
) {
    spawn_icon_button(parent, icon, marker, colors);
}
