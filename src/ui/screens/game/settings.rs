use bevy::prelude::*;

use crate::ui::{app::GameSettings, screens::game::game::{AITimeText, AIAverageTimeText, TotalMovesText, PlayerCapturesText, AICapturesText, UndoButton, RestartButton}};

#[derive(Component)]
pub struct GameSettingsPanel;

// Events for settings changes
#[derive(Event)]
pub struct SettingsChanged;

// Components for interactive settings
#[derive(Component, Clone)]
pub struct SettingsBoardSizeButton {
    pub increment: bool, // true for +, false for -
}

#[derive(Component, Clone)]
pub struct SettingsChainToWinButton {
    pub increment: bool,
}

#[derive(Component, Clone)]
pub struct SettingsCaptureToWinButton {
    pub increment: bool,
}

#[derive(Component, Clone)]
pub struct SettingsAIDepthButton {
    pub increment: bool,
}

#[derive(Component, Clone)]
pub struct SettingsTimeLimitButton {
    pub increment: bool,
}

#[derive(Component, Clone)]
pub struct SettingsTimeLimitToggleButton;

// Text display markers for settings values
#[derive(Component)]
pub struct BoardSizeText;

#[derive(Component)]
pub struct ChainToWinText;

#[derive(Component)]
pub struct CaptureToWinText;

#[derive(Component)]
pub struct AIDepthText;

#[derive(Component)]
pub struct TimeLimitText;

pub fn spawn_settings_panel(builder: &mut ChildSpawnerCommands, game_settings: &GameSettings) {
    builder
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(12.0),
                min_width: Val::Px(300.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            BorderColor(Color::srgb(0.3, 0.3, 0.3)),
            BorderRadius::all(Val::Px(8.0)),
            GameSettingsPanel,
        ))
        .with_children(|builder| {
            // Title
            builder.spawn((
                Text::new("Game Settings"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            // Board Size Setting
            spawn_interactive_setting_row(
                builder, 
                "Board Size", 
                &format!("{}x{}", game_settings.board_size, game_settings.board_size),
                Some((SettingsBoardSizeButton { increment: false }, SettingsBoardSizeButton { increment: true })),
                BoardSizeText,
            );

            // Chain to Win Setting
            spawn_interactive_setting_row(
                builder, 
                "Chain to Win", 
                &game_settings.minimum_chain_to_win.to_string(),
                Some((SettingsChainToWinButton { increment: false }, SettingsChainToWinButton { increment: true })),
                ChainToWinText,
            );

            // Captures to Win Setting
            spawn_interactive_setting_row(
                builder, 
                "Captures to Win", 
                &game_settings.total_capture_to_win.to_string(),
                Some((SettingsCaptureToWinButton { increment: false }, SettingsCaptureToWinButton { increment: true })),
                CaptureToWinText,
            );

            if game_settings.versus_ai {
                // AI Depth Setting
                spawn_interactive_setting_row(
                    builder, 
                    "AI Depth", 
                    &game_settings.ai_depth.to_string(),
                    Some((SettingsAIDepthButton { increment: false }, SettingsAIDepthButton { increment: true })),
                    AIDepthText,
                );
                
                // Time Limit Setting
                if let Some(time_limit) = game_settings.time_limit {
                    spawn_interactive_setting_row(
                        builder, 
                        "Time Limit (ms)", 
                        &time_limit.to_string(),
                        Some((SettingsTimeLimitButton { increment: false }, SettingsTimeLimitButton { increment: true })),
                        TimeLimitText,
                    );
                    
                    spawn_toggle_setting_row(builder, "Disable Time Limit", "Click to disable", SettingsTimeLimitToggleButton, TimeLimitText);
                } else {
                    spawn_toggle_setting_row(builder, "Enable Time Limit", "Click to enable", SettingsTimeLimitToggleButton, TimeLimitText);
                }
                
                // Alpha-Beta Setting
                let alpha_beta = if game_settings.alpha_beta_enabled { "Enabled" } else { "Disabled" };
                spawn_setting_row(builder, "Alpha-Beta", alpha_beta);

                // AI Timer rows
                spawn_timer_row(builder, "AI Time", "");
                spawn_average_timer_row(builder, "AI Avg Time", "");
            }
            
            // Game Statistics Section
            builder.spawn((
                Text::new("Game Statistics"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::vertical(Val::Px(12.0)),
                    ..default()
                },
            ));

            // Total moves counter
            spawn_stat_row(builder, "Total Moves", "0", TotalMovesText);
            
            // Player captures
            spawn_stat_row(builder, "Player Captures", "0", PlayerCapturesText);
            
            // AI captures  
            spawn_stat_row(builder, "AI Captures", "0", AICapturesText);
            
            // Undo button
            builder.spawn((
                Button,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(40.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(12.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                BorderRadius::all(Val::Px(4.0)),
                UndoButton,
            )).with_children(|builder| {
                builder.spawn((
                    Text::new("Undo Move"),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::WHITE),
                ));
            });
            
            // Restart button
            builder.spawn((
                Button,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(40.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.7, 0.3, 0.3)),
                BorderRadius::all(Val::Px(4.0)),
                RestartButton,
            )).with_children(|builder| {
                builder.spawn((
                    Text::new("Restart Game"),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::WHITE),
                ));
            });
        });
}

fn spawn_setting_row(builder: &mut ChildSpawnerCommands, label: &str, value: &str) {
    builder
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.08, 0.08)),
            BorderRadius::all(Val::Px(4.0)),
        ))
        .with_children(|builder| {
            builder.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));

            builder.spawn((
                Text::new(value),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn spawn_timer_row(builder: &mut ChildSpawnerCommands, label: &str, _value: &str) {
    builder
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.08, 0.08)),
            BorderRadius::all(Val::Px(4.0)),
        ))
        .with_children(|builder| {
            builder.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));

				builder.spawn((
                    Text::new(" 0.00s"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    AITimeText,
                    Node {
                        margin: UiRect::top(Val::Px(8.0)),
                        ..default()
                    },
                ));
        });
}

fn spawn_average_timer_row(builder: &mut ChildSpawnerCommands, label: &str, _value: &str) {
    builder
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.08, 0.08)),
            BorderRadius::all(Val::Px(4.0)),
        ))
        .with_children(|builder| {
            builder.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));

				builder.spawn((
                    Text::new("0.0ms"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    AIAverageTimeText,
                    Node {
                        margin: UiRect::top(Val::Px(8.0)),
                        ..default()
                    },
                ));
        });
}

fn spawn_stat_row<T: Component>(builder: &mut ChildSpawnerCommands, label: &str, initial_value: &str, marker: T) {
    builder
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.08, 0.08)),
            BorderRadius::all(Val::Px(4.0)),
        ))
        .with_children(|builder| {
            builder.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));

            builder.spawn((
                Text::new(initial_value),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                marker,
            ));
        });
}

fn spawn_interactive_setting_row<T1: Component + Clone, T2: Component, TM: Component>(
    builder: &mut ChildSpawnerCommands, 
    label: &str, 
    value: &str,
    buttons: Option<(T1, T2)>,
    text_marker: TM,
) {
    builder
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.08, 0.08)),
            BorderRadius::all(Val::Px(4.0)),
        ))
        .with_children(|builder| {
            builder.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));

            // Value and buttons container
            builder.spawn((
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                },
            )).with_children(|builder| {
                // Decrease button
                if let Some((decrease_component, _)) = &buttons {
                    builder.spawn((
                        Button,
                        Node {
                            width: Val::Px(24.0),
                            height: Val::Px(24.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                        BorderRadius::all(Val::Px(4.0)),
                        decrease_component.clone(),
                    )).with_children(|builder| {
                        builder.spawn((
                            Text::new("-"),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                    });
                }

                // Value text with marker
                builder.spawn((
                    Text::new(value),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        min_width: Val::Px(60.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    text_marker,
                ));

                // Increase button
                if let Some((_, increase_component)) = buttons {
                    builder.spawn((
                        Button,
                        Node {
                            width: Val::Px(24.0),
                            height: Val::Px(24.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                        BorderRadius::all(Val::Px(4.0)),
                        increase_component,
                    )).with_children(|builder| {
                        builder.spawn((
                            Text::new("+"),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                    });
                }
            });
        });
}

fn spawn_toggle_setting_row<T: Component, TM: Component>(
    builder: &mut ChildSpawnerCommands, 
    label: &str, 
    current_value: &str,
    button_component: T,
    text_marker: TM,
) {
    builder
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.08, 0.08)),
            BorderRadius::all(Val::Px(4.0)),
        ))
        .with_children(|builder| {
            builder.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));

            // Toggle button
            builder.spawn((
                Button,
                Node {
                    padding: UiRect::all(Val::Px(8.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.3, 0.5, 0.3)),
                BorderRadius::all(Val::Px(4.0)),
                button_component,
            )).with_children(|builder| {
                builder.spawn((
                    Text::new(current_value),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::WHITE),
                    text_marker,
                ));
            });
        });
}