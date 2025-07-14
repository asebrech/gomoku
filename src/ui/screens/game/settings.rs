use bevy::prelude::*;

use crate::ui::{app::GameSettings, screens::game::game::AITimeText};

#[derive(Component)]
pub struct GameSettingsPanel;

pub fn spawn_settings_panel(builder: &mut ChildSpawnerCommands, game_settings: &GameSettings) {
    builder
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(12.0),
                min_width: Val::Px(250.0),
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

            // Board Size
            spawn_setting_row(builder, "Board Size", &format!("{}x{}", game_settings.board_size, game_settings.board_size));

            // Win Condition
            spawn_setting_row(builder, "Chain to Win", &game_settings.minimum_chain_to_win.to_string());

            // Capture to Win
            spawn_setting_row(builder, "Captures to Win", &game_settings.total_capture_to_win.to_string());

            // Game Mode
            let game_mode = if game_settings.versus_ai { "vs AI" } else { "Multiplayer" };
            spawn_setting_row(builder, "Game Mode", game_mode);

            // AI Settings (only if vs AI)
            if game_settings.versus_ai {
                spawn_setting_row(builder, "AI Depth", &game_settings.ai_depth.to_string());
                
                let alpha_beta = if game_settings.alpha_beta_enabled { "Enabled" } else { "Disabled" };
                spawn_setting_row(builder, "Alpha-Beta", alpha_beta);

                // AI Time Taken
                //spawn_setting_row(builder, "AI Time", "0.00s");
				spawn_timer_row(builder, "AI Time", "");
            }


            // Time Limit
            let time_limit = match game_settings.time_limit {
                Some(seconds) => format!("{}s", seconds),
                None => "Unlimited".to_string(),
            };
            spawn_setting_row(builder, "Time Limit", &time_limit);
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
            // Label
            builder.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));

            // Value
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

fn spawn_timer_row(builder: &mut ChildSpawnerCommands, label: &str, value: &str) {
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
            // Label
            builder.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));

            // Value
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