use bevy::prelude::*;

use crate::{
    audio::PlayClickSound,
    ui::{
        app::GameSettings,
        config::GameConfig,
        screens::game::game::{AITimeText, AIDepthText, RoundNumberText},
    }
};

#[derive(Component)]
pub struct GameSettingsPanel;

#[derive(Component)]
pub struct VolumeUp;

#[derive(Component)]
pub struct VolumeDown;

#[derive(Component)]
pub struct VolumeDisplay;

#[derive(Component)]
pub struct ResetBoardButton;

#[derive(Component)]
pub struct BackToMenuButton;

pub fn spawn_settings_panel(builder: &mut ChildSpawnerCommands, game_settings: &GameSettings, config: &GameConfig) {
    let colors = &config.colors;
    
    builder
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                ..default()
            },
            GameSettingsPanel,
        ))
        .with_children(|builder| {
            // Turns display (at the top)
            builder.spawn((
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(15.0)),
                    width: Val::Px(350.0),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(
                    colors.surface.r * 0.8,
                    colors.surface.g * 0.8,
                    colors.surface.b * 0.8,
                    0.8
                )),
                BorderColor(colors.accent.clone().into()),
                BorderRadius::all(Val::Px(8.0)),
            )).with_children(|builder| {
                builder.spawn((
                    Text::new("Turns: 1"),
                    TextFont {
                        font_size: 22.0,
                        ..default()
                    },
                    TextColor(colors.accent.clone().into()),
                    RoundNumberText, // Component for updating the turn count
                ));
            });
            
            // Settings content container
            builder.spawn((
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(20.0)),
                    row_gap: Val::Px(12.0),
                    width: Val::Px(350.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(colors.surface.clone().into()),
                BorderColor(colors.accent.clone().into()),
                BorderRadius::all(Val::Px(8.0)),
            ))
            .with_children(|builder| {
                builder.spawn((
                    Text::new("Game Settings"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(colors.accent.clone().into()),
                    Node {
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    },
                ));

                spawn_setting_row(builder, config, "Board Size", &format!("{}x{}", game_settings.board_size, game_settings.board_size));

                spawn_setting_row(builder, config, "Chain to Win", &game_settings.minimum_chain_to_win.to_string());

                spawn_setting_row(builder, config, "Captures to Win", &game_settings.total_capture_to_win.to_string());

                let game_mode = if game_settings.versus_ai { "vs AI" } else { "Multiplayer" };
                spawn_setting_row(builder, config, "Game Mode", game_mode);

                if game_settings.versus_ai {
                    let max_depth_display = if game_settings.ai_depth == 0 {
                        "Unlimited".to_string()
                    } else {
                        game_settings.ai_depth.to_string()
                    };
                    spawn_setting_row(builder, config, "Maximum Depth", &max_depth_display);

                    spawn_timer_row(builder, config, "AI Time", "");
                    spawn_depth_row(builder, config, "Depth Reached", "");
                }

                // Time Limit
                let time_limit = match game_settings.time_limit {
                    Some(seconds) => format!("{}s", seconds),
                    None => "Unlimited".to_string(),
                };
                spawn_setting_row(builder, config, "Time Limit", &time_limit);

                // Volume Control Section
                spawn_volume_control(builder, config);

                // Reset Board button (inside settings panel)
                spawn_reset_button(builder, config);
            });

            // Back to Menu button (outside settings panel, separate container)
            spawn_back_to_menu_button(builder, config);
        });
}

fn spawn_setting_row(builder: &mut ChildSpawnerCommands, config: &GameConfig, label: &str, value: &str) {
    let colors = &config.colors;
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
            BackgroundColor(colors.surface.clone().into()),
            BorderRadius::all(Val::Px(4.0)),
        ))
        .with_children(|builder| {
            builder.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(colors.text_primary.clone().into()),
            ));

            builder.spawn((
                Text::new(value),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(colors.accent.clone().into()),
            ));
        });
}

fn spawn_timer_row(builder: &mut ChildSpawnerCommands, config: &GameConfig, label: &str, _value: &str) {
    let colors = &config.colors;
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
            BackgroundColor(colors.surface.clone().into()),
            BorderRadius::all(Val::Px(4.0)),
        ))
        .with_children(|builder| {
            builder.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(colors.text_primary.clone().into()),
            ));

				builder.spawn((
                    Text::new(" 0.00s"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(colors.accent.clone().into()),
                    AITimeText,
                    Node {
                        margin: UiRect::top(Val::Px(8.0)),
                        ..default()
                    },
                ));
        });
}

fn spawn_depth_row(builder: &mut ChildSpawnerCommands, config: &GameConfig, label: &str, _value: &str) {
    let colors = &config.colors;
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
            BackgroundColor(colors.surface.clone().into()),
            BorderRadius::all(Val::Px(4.0)),
        ))
        .with_children(|builder| {
            builder.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(colors.text_primary.clone().into()),
            ));

            builder.spawn((
                Text::new("0"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(colors.accent.clone().into()),
                AIDepthText,
                Node {
                    margin: UiRect::top(Val::Px(8.0)),
                    ..default()
                },
            ));
        });
}

fn spawn_volume_control(builder: &mut ChildSpawnerCommands, config: &GameConfig) {
    let colors = &config.colors;
    // Get current volume settings
    let (volume, muted) = config.get_audio_settings();
    let volume_text = if muted {
        "MUTED".to_string()
    } else {
        format!("{}%", (volume * 100.0) as u32)
    };
    
    // Volume control section header
    builder.spawn((
        Text::new("Audio"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(colors.accent.clone().into()),
        Node {
            margin: UiRect::vertical(Val::Px(10.0)),
            ..default()
        },
    ));

    // Volume control row
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
            BackgroundColor(colors.surface.clone().into()),
            BorderRadius::all(Val::Px(4.0)),
        ))
        .with_children(|builder| {
            // Volume label
            builder.spawn((
                Text::new("Volume"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(colors.text_primary.clone().into()),
            ));

            // Volume controls container
            builder.spawn((
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                },
            )).with_children(|builder| {
                // Volume down button - using button component pattern
                builder.spawn((
                    Button,
                    PlayClickSound,
                    Node {
                        width: Val::Px(25.0),
                        height: Val::Px(25.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(colors.button_normal.clone().into()),
                    BorderColor(colors.accent.clone().into()),
                    BorderRadius::all(Val::Px(4.0)),
                    VolumeDown,
                )).with_children(|builder| {
                    builder.spawn((
                        Text::new("-"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(colors.text_primary.clone().into()),
                    ));
                });

                // Volume display
                builder.spawn((
                    Text::new(volume_text),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(colors.accent.clone().into()),
                    VolumeDisplay,
                    Node {
                        width: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ));

                // Volume up button
                builder.spawn((
                    Button,
                    PlayClickSound,
                    Node {
                        width: Val::Px(25.0),
                        height: Val::Px(25.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(colors.button_normal.clone().into()),
                    BorderColor(colors.accent.clone().into()),
                    BorderRadius::all(Val::Px(4.0)),
                    VolumeUp,
                )).with_children(|builder| {
                    builder.spawn((
                        Text::new("+"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(colors.text_primary.clone().into()),
                    ));
                });
            });
        });
}

fn spawn_reset_button(builder: &mut ChildSpawnerCommands, config: &GameConfig) {
    let colors = &config.colors;
    // Game control section header
    builder.spawn((
        Text::new("Game Controls"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(colors.accent.clone().into()),
        Node {
            margin: UiRect::vertical(Val::Px(10.0)),
            ..default()
        },
    ));

    // Reset Board button
    builder.spawn((
        Button,
        PlayClickSound,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(40.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(colors.button_normal.clone().into()),
        BorderColor(colors.accent.clone().into()),
        BorderRadius::all(Val::Px(4.0)),
        ResetBoardButton,
    )).with_children(|builder| {
        builder.spawn((
            Text::new("RESET BOARD"),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(colors.text_primary.clone().into()),
        ));
    });
}

fn spawn_back_to_menu_button(builder: &mut ChildSpawnerCommands, config: &GameConfig) {
    let colors = &config.colors;
    // Back to Menu button - in its own container, not constrained by column
    builder.spawn((
        Button,
        PlayClickSound,
        Node {
            width: Val::Px(350.0),
            height: Val::Px(50.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(8.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(colors.button_normal.clone().into()),
        BorderColor(colors.accent.clone().into()),
        BorderRadius::all(Val::Px(4.0)),
        BackToMenuButton,
    )).with_children(|builder| {
        builder.spawn((
            Text::new("BACK TO MENU"),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(colors.accent.clone().into()),
            TextLayout {
                justify: JustifyText::Center,
                linebreak: LineBreak::NoWrap,
            },
        ));
    });
}
