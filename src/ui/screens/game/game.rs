use std::time::Duration;

use bevy::prelude::*;
use crate::{
    ai::lazy_smp::lazy_smp_search,
    core::{board::Player, moves::RuleValidator, state::GameState}, 
    ui::{
        app::{AppState, GameSettings}, 
        config::GameConfig,
        screens::{
            game::{
                board::{BoardRoot, BoardUtils, PreviewDot}, 
                settings::{spawn_settings_panel, BackToMenuButton, ResetBoardButton, VolumeDisplay, VolumeDown, VolumeUp}
            }, menu::GameAudio, splash::PreloadedStones, utils::despawn_screen
        },
    }
};

// Game status resource
#[derive(Resource, Default)]
pub enum GameStatus {
    #[default]
    AwaitingUserInput,
    Paused,
    GameOver,
}

#[derive(Component, Clone)]
pub struct OnGameScreen;
#[derive(Component)]
pub struct Stone(Player);
#[derive(Component)]
pub struct AvailableArea;
#[derive(Event)]
pub struct StonePlacement {
    x: usize,
    y: usize,
}
#[derive(Event)]
pub struct MovePlayed;
#[derive(Event)]
pub struct GameEnded {
    winner: Option<Player>,
}

#[derive(Event)]
pub struct ResetBoard;

#[derive(Component)]
pub struct GameOverOverlay;

#[derive(Component)]
pub enum GameOverAction {
    RestartGame,
    BackToMenu,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridCell {
    pub x: usize,
    pub y: usize,
}

pub fn game_plugin(app: &mut App) {
    app.init_resource::<GameStatus>()
        .init_resource::<AITimeTaken>()
        .init_resource::<AIDepthReached>()
        .add_event::<GameEnded>()
        .add_event::<StonePlacement>()
        .add_event::<MovePlayed>()
        .add_event::<UpdateAITimeDisplay>()
        .add_event::<UpdateAIDepthDisplay>()
        .add_event::<ResetBoard>()
        .add_systems(OnEnter(AppState::Game), (update_game_settings_from_config, setup_game_ui, update_available_placement).chain())
        .add_systems(
            Update,
            (
                handle_player_placement,
                place_stone.run_if(on_event::<StonePlacement>),
                process_next_round.run_if(on_event::<MovePlayed>),
                update_available_placement.run_if(on_event::<MovePlayed>),
                reset_board.run_if(on_event::<ResetBoard>),
                toggle_pause,
                update_ai_time_display.run_if(on_event::<UpdateAITimeDisplay>),
                update_ai_depth_display.run_if(on_event::<UpdateAIDepthDisplay>),
                handle_game_volume_control,
                update_game_volume_display,
                handle_reset_board_button,
                handle_back_to_menu_button,
                show_game_over_screen.run_if(on_event::<GameEnded>),
                handle_game_over_actions,
            ).run_if(in_state(AppState::Game)),
        )
        .add_systems(OnExit(AppState::Game), despawn_screen::<OnGameScreen>);
}

fn update_game_settings_from_config(
    config: Res<GameConfig>,
    mut game_settings: ResMut<GameSettings>,
    mut game_state: ResMut<GameState>,
) {
    // Get current settings from config
    let (board_size, win_condition, ai_max_depth, ai_time_limit, pair_captures_to_win) = config.get_game_settings();
    
    println!("📋 update_game_settings_from_config:");
    println!("  board_size: {}", board_size);
    println!("  win_condition: {}", win_condition);
    println!("  ai_max_depth: {:?}", ai_max_depth);
    println!("  ai_time_limit: {:?}", ai_time_limit);
    println!("  pair_captures_to_win: {}", pair_captures_to_win);
    
    // Convert AI parameters
    // If ai_max_depth is None (unlimited), default to depth 6 for reasonable performance
    let ai_depth = match ai_max_depth {
        Some(depth) => depth as i32,
        None => 6, // Unlimited depth defaults to 6 for safety
    };
    
    // Convert time_limit from Option<u64> to Option<usize>
    let time_limit = ai_time_limit.map(|ms| ms as usize);
    
    println!("  Converted ai_depth: {}", ai_depth);
    println!("  Converted time_limit: {:?}", time_limit);
    
    // Update GameSettings resource
    *game_settings = GameSettings {
        board_size: board_size as usize,
        total_capture_to_win: pair_captures_to_win as usize,
        minimum_chain_to_win: win_condition as usize,
        ai_depth,
        alpha_beta_enabled: true,
        versus_ai: true,
        time_limit,
    };
    
    // Create a new GameState with updated settings
    *game_state = GameState::new(game_settings.board_size, game_settings.minimum_chain_to_win, game_settings.total_capture_to_win);
    
    info!("Updated game settings from config: Board {}x{}, Win Condition: {}, AI Max Depth: {:?} (using depth: {}), AI Time Limit: {:?}, Pair Captures: {}", 
          board_size, board_size, win_condition, ai_max_depth, ai_depth, ai_time_limit, pair_captures_to_win);
}

fn setup_game_ui(
    mut commands: Commands, 
    game_settings: Res<GameSettings>,
) {
    commands
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(40.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            OnGameScreen,
        ))
        .with_children(|builder| {
            builder.spawn((
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
            )).with_children(|builder| {
                BoardUtils::spawn_board(builder, &game_settings);
            });
            spawn_settings_panel(builder, &game_settings);
        });
}

pub fn update_available_placement(
    mut commands: Commands,
    mut ev_board_update: EventReader<MovePlayed>,
    game_state: Res<GameState>,
    parents: Query<(Entity, &Children, &GridCell), With<GridCell>>,
    mut dots: Query<(&mut BackgroundColor, &mut Visibility), With<PreviewDot>>,
) {
    // Consume events
    for _ in ev_board_update.read() {}

    info!("Updating stone preview...");
    for (entity, children, cell) in parents.iter() {
        // Check if position is empty and doesn't create double-three
        let is_valid = game_state.board.is_empty_position(cell.x, cell.y)
            && !RuleValidator::creates_double_three(&game_state.board, cell.x, cell.y, game_state.current_player);
        
        if is_valid {
            for &child in children {
                if let Ok((mut bg, mut visibility)) = dots.get_mut(child) {
                    *bg = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.4));
                    *visibility = Visibility::Visible;
                    commands.entity(entity).insert(AvailableArea);
                }
            }
        } else {
            for &child in children {
                if let Ok((mut bg, mut visibility)) = dots.get_mut(child) {
                    *bg = BackgroundColor(Color::NONE);
                    *visibility = Visibility::Hidden;
                    commands.entity(entity).remove::<AvailableArea>();
                }
            }
        }
    }
}

pub fn place_stone(
    mut commands: Commands,
    preloaded_stones: Res<PreloadedStones>,
    board_query: Query<Entity, With<BoardRoot>>,
    mut game_state: ResMut<GameState>,
    mut ev_stone_placement: EventReader<StonePlacement>,
    mut move_played: EventWriter<MovePlayed>,
    stones: Query<(Entity, &GridCell, &Stone)>,
) {
    for ev in ev_stone_placement.read() {
        info!("Stone placed at x: {}, y: {}", ev.x, ev.y);
        game_state.make_move((ev.x, ev.y));

        let player = game_state.current_player;

        // Despawn captured stones cleanly
        for (stone_entity, stone_cell, _) in stones.iter() {
            if game_state.board.is_empty_position(stone_cell.x, stone_cell.y) {
                info!("Despawning captured stone at x: {}, y: {}", stone_cell.x, stone_cell.y);
                commands.entity(stone_entity).despawn();
            }
        }

        // Spawn new stone
        if let Ok(board_entity) = board_query.single() {
            let is_black = player == Player::Min;
            
            commands.entity(board_entity).with_children(|builder| {
                // Use preloaded stone images
                let stone_handle = if is_black {
                    preloaded_stones.pink_stone.clone()  // Pink for black stones
                } else {
                    preloaded_stones.blue_stone.clone()  // Blue for white stones  
                };
                
                // Spawn stone with preloaded image
                builder.spawn((
                    BoardUtils::stone_node(ev.x, ev.y, BoardUtils::STONE_SIZE),
                    ImageNode::new(stone_handle),
                    Stone(player),
                    ZIndex(15),
                    OnGameScreen,
                    GridCell { x: ev.x, y: ev.y },
                ));
            });
        }
        move_played.write(MovePlayed);
    }
}

pub fn handle_player_placement(
    mut stone_placement: EventWriter<StonePlacement>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut interaction_query: Query<
        (&Interaction, &GridCell),
        (With<AvailableArea>, Without<Stone>),
    >,
    game_state: ResMut<GameState>,
    game_status: Res<GameStatus>,
) {
    if matches!(*game_status, GameStatus::AwaitingUserInput) && buttons.just_pressed(MouseButton::Left) {
        for (interaction, cell) in interaction_query.iter_mut() {
            if *interaction == Interaction::Pressed
                && game_state.board.get_player(cell.x, cell.y).is_none()
            {
                stone_placement.write(StonePlacement {
                    x: cell.x,
                    y: cell.y,
                });
            }
        }
    }
}

pub fn process_next_round(
    mut move_played: EventReader<MovePlayed>,
    mut stone_placement: EventWriter<StonePlacement>,
    mut game_event: EventWriter<GameEnded>,
    settings: Res<GameSettings>,
    mut game_state: ResMut<GameState>,
    mut game_status: ResMut<GameStatus>,
    mut ai_time: ResMut<AITimeTaken>,
    mut ai_depth: ResMut<AIDepthReached>,
    mut update_ai_time: EventWriter<UpdateAITimeDisplay>,
    mut update_ai_depth: EventWriter<UpdateAIDepthDisplay>,
) {
    for _ in move_played.read() {
        // Check for game end first
        if game_state.is_terminal() {
            let winner = game_state.check_winner();
            game_event.write(GameEnded { winner });
            *game_status = GameStatus::GameOver;
            
            match winner {
                Some(player) => println!("Game Over! Winner: {:?}", player),
                None => println!("Game Over! It's a draw."),
            }
            return;
        }

        // Handle next player's turn
        if game_state.current_player == Player::Max || (game_state.current_player == Player::Min && !settings.versus_ai) {
            info!("Awaiting user click");
            *game_status = GameStatus::AwaitingUserInput;
        } else if settings.versus_ai {
            // AI's turn
            
            if !game_state.is_terminal() {
                let placement = if let Some(time_limit_ms) = settings.time_limit {
                    let time_limit = Duration::from_millis(time_limit_ms as u64);
                    info!("AI using Lazy SMP search with {}ms limit", time_limit_ms);
                    lazy_smp_search(&mut game_state, settings.ai_depth, Some(time_limit), None)
                } else {
                    info!("AI using Lazy SMP search to depth {}", settings.ai_depth);
                    lazy_smp_search(&mut game_state, settings.ai_depth, None, None)
                };
                ai_time.micros = placement.time_elapsed.as_micros();
                ai_depth.depth = placement.depth_reached;
                update_ai_time.write(UpdateAITimeDisplay);
                update_ai_depth.write(UpdateAIDepthDisplay);

                if let Some((x, y)) = placement.best_move {
                    stone_placement.write(StonePlacement { x, y });
                    *game_status = GameStatus::AwaitingUserInput;
                } else {
                    // AI has no moves but game isn't terminal - this shouldn't happen
                    // But if it does, it means the game is likely a draw
                    println!("AI has no valid moves available");
                    game_event.write(GameEnded { winner: None });
                    *game_status = GameStatus::GameOver;
                }
            }
        }
    }
}

pub fn update_ai_time_display(
    mut query: Query<&mut Text, With<AITimeText>>,
    ai_time: Res<AITimeTaken>,
    mut events: EventReader<UpdateAITimeDisplay>,
) {
    for _ in events.read() {
        let time_ms = ai_time.micros as f64 / 1000.0;
        info!("Updating AI time display: {:.1}ms", time_ms);
        for mut text in query.iter_mut() {
			text.0 = format!("{:.1}ms", time_ms);
        }
    }
}

pub fn update_ai_depth_display(
    mut query: Query<&mut Text, With<AIDepthText>>,
    ai_depth: Res<AIDepthReached>,
    mut events: EventReader<UpdateAIDepthDisplay>,
) {
    for _ in events.read() {
        info!("Updating AI depth display: depth {}", ai_depth.depth);
        for mut text in query.iter_mut() {
			text.0 = format!("depth {}", ai_depth.depth);
        }
    }
}

#[derive(Component)]
pub struct AITimeText;

#[derive(Resource, Default)]
pub struct AITimeTaken {
    pub micros: u128,
}

#[derive(Event)]
pub struct UpdateAITimeDisplay;

#[derive(Component)]
pub struct AIDepthText;

#[derive(Resource, Default)]
pub struct AIDepthReached {
    pub depth: i32,
}

#[derive(Event)]
pub struct UpdateAIDepthDisplay;


pub fn toggle_pause(
    mut game_status: ResMut<GameStatus>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyP) {
        *game_status = match *game_status {
            GameStatus::Paused => {
                println!("Game unpaused !");
                GameStatus::AwaitingUserInput
            }
            GameStatus::AwaitingUserInput => {
                println!("Game paused !");
                GameStatus::Paused
            }
            GameStatus::GameOver => GameStatus::GameOver,
        };
    }
}

pub fn handle_game_volume_control(
    volume_up_query: Query<&Interaction, (Changed<Interaction>, With<VolumeUp>, With<Button>)>,
    volume_down_query: Query<&Interaction, (Changed<Interaction>, With<VolumeDown>, With<Button>)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<GameConfig>,
    mut audio_sink_query: Query<&mut AudioSink>,
    game_audio: Option<Res<GameAudio>>,
) {
    let mut volume_changed = false;
    let (current_volume, mut muted) = config.get_audio_settings();
    
    // Convert to percentage (0-10) for clean increments
    let mut volume_percent = (current_volume * 10.0).round() as i32;
    
    // Handle volume up button
    for interaction in volume_up_query.iter() {
        if *interaction == Interaction::Pressed {
            let old_percent = volume_percent;
            volume_percent = (volume_percent + 1).min(10);
            muted = false;
            volume_changed = true;
            info!("GAME VOLUME UP: {}% -> {}%", old_percent * 10, volume_percent * 10);
        }
    }
    
    // Handle volume down button
    for interaction in volume_down_query.iter() {
        if *interaction == Interaction::Pressed {
            let old_percent = volume_percent;
            volume_percent = (volume_percent - 1).max(0);
            if volume_percent == 0 {
                muted = true;
            }
            volume_changed = true;
            info!("GAME VOLUME DOWN: {}% -> {}%", old_percent * 10, volume_percent * 10);
        }
    }
    
    // Handle keyboard controls for volume (only if not paused)
    if keyboard_input.just_pressed(KeyCode::Equal) || keyboard_input.just_pressed(KeyCode::NumpadAdd) {
        let old_percent = volume_percent;
        volume_percent = (volume_percent + 1).min(10);
        muted = false;
        volume_changed = true;
        info!("GAME KEYBOARD UP: {}% -> {}%", old_percent * 10, volume_percent * 10);
    }
    if keyboard_input.just_pressed(KeyCode::Minus) || keyboard_input.just_pressed(KeyCode::NumpadSubtract) {
        let old_percent = volume_percent;
        volume_percent = (volume_percent - 1).max(0);
        if volume_percent == 0 {
            muted = true;
        }
        volume_changed = true;
        info!("GAME KEYBOARD DOWN: {}% -> {}%", old_percent * 10, volume_percent * 10);
    }
    
    // Apply volume changes and save to config
    if volume_changed {
        // Convert back to float (0.0-1.0) with clean values
        let volume = volume_percent as f32 / 10.0;
        
        // Save to persistent config
        if let Err(e) = config.save_audio_settings(volume, muted) {
            info!("Failed to save audio settings: {}", e);
        } else {
            info!("Saved audio settings: volume={}, muted={}", volume, muted);
        }
        
        // Apply to current audio
        let effective_volume = if muted { 0.0 } else { volume };
        if let Some(audio) = game_audio {
            if let Some(entity) = audio.music_entity {
                if let Ok(mut sink) = audio_sink_query.get_mut(entity) {
                    sink.set_volume(bevy::audio::Volume::Linear(effective_volume));
                    info!("Updated game AudioSink volume to: {}", effective_volume);
                } else {
                    info!("Could not find AudioSink component on entity in game");
                }
            }
        }
    }
}

pub fn update_game_volume_display(
    config: Res<GameConfig>,
    mut volume_display_query: Query<&mut Text, With<VolumeDisplay>>,
) {
    if config.is_changed() {
        let (volume, muted) = config.get_audio_settings();
        for mut text in volume_display_query.iter_mut() {
            if muted {
                text.0 = "MUTED".to_string();
            } else {
                text.0 = format!("{}%", (volume * 100.0) as u32);
            }
        }
    }
}
fn handle_reset_board_button(
    button_query: Query<&Interaction, (Changed<Interaction>, With<ResetBoardButton>)>,
    mut reset_event: EventWriter<ResetBoard>,
) {
    for interaction in button_query.iter() {
        if *interaction == Interaction::Pressed {
            println!("🔘 Reset Board button clicked - sending reset event...");
            reset_event.write(ResetBoard);
        }
    }
}

fn reset_board(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    game_settings: Res<GameSettings>,
    mut game_status: ResMut<GameStatus>,
    mut ai_time_taken: ResMut<AITimeTaken>,
    mut ai_depth_reached: ResMut<AIDepthReached>,
    stone_query: Query<Entity, With<Stone>>,
    mut move_played: EventWriter<MovePlayed>,
) {
    println!("======================================");
    println!("       RESETTING GAME BOARD");
    println!("======================================");
    
    // Print state BEFORE reset
    println!("\n[STATE BEFORE RESET]");
    println!("  Current Player: {:?}", game_state.current_player);
    println!("  Moves Played: {}", game_state.move_history.len());
    println!("  Game Status: {}", match *game_status {
        GameStatus::AwaitingUserInput => "AwaitingUserInput",
        GameStatus::Paused => "Paused",
        GameStatus::GameOver => "GameOver",
    });
    println!("  Max Player Captures: {}", game_state.max_captures);
    println!("  Min Player Captures: {}", game_state.min_captures);
    println!("  AI Time Taken: {:.3}s", ai_time_taken.micros as f64 / 1_000_000.0);
    println!("  AI Depth Reached: {}", ai_depth_reached.depth);
    
    // Count stones to despawn
    let stone_count = stone_query.iter().count();
    println!("\n[DESPAWNING] Despawning {} stones...", stone_count);
    
    // Despawn all stones
    for entity in stone_query.iter() {
        commands.entity(entity).despawn();
    }
    
    println!("\n🔧 Creating fresh GameState...");
    // Reset the game state
    *game_state = GameState::new(
        game_settings.board_size,
        game_settings.minimum_chain_to_win,
        game_settings.total_capture_to_win,
    );
    
    // Reset game status
    *game_status = GameStatus::AwaitingUserInput;
    
    // Reset AI tracking
    ai_time_taken.micros = 0;
    ai_depth_reached.depth = 0;
    
    // Print state AFTER reset
    println!("\n[STATE AFTER RESET]");
    println!("  Current Player: {:?}", game_state.current_player);
    println!("  Moves Played: {}", game_state.move_history.len());
    println!("  Game Status: {}", match *game_status {
        GameStatus::AwaitingUserInput => "AwaitingUserInput",
        GameStatus::Paused => "Paused",
        GameStatus::GameOver => "GameOver",
    });
    println!("  Max Player Captures: {}", game_state.max_captures);
    println!("  Min Player Captures: {}", game_state.min_captures);
    println!("  Board Size: {}x{}", game_settings.board_size, game_settings.board_size);
    println!("  Win Condition: {}", game_settings.minimum_chain_to_win);
    println!("  Captures to Win: {}", game_settings.total_capture_to_win);
    
    // Trigger update_available_placement by sending MovePlayed event
    println!("\n[TRIGGERING] Sending MovePlayed event to update available placements...");
    move_played.write(MovePlayed);
    
    println!("\n[RESET COMPLETE] Game board reset complete!");
    println!("======================================\n");
}

fn handle_back_to_menu_button(
    button_query: Query<&Interaction, (Changed<Interaction>, With<BackToMenuButton>)>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    for interaction in button_query.iter() {
        if *interaction == Interaction::Pressed {
            info!("Back to Menu button pressed!");
            app_state.set(AppState::Menu);
        }
    }
}

fn show_game_over_screen(
    mut commands: Commands,
    mut game_ended_events: EventReader<GameEnded>,
    config: Res<GameConfig>,
    game_settings: Res<GameSettings>,
) {
    for event in game_ended_events.read() {
        info!("Game Over! Winner: {:?}", event.winner);
        
        let colors = &config.colors;
        
        // Determine the title and message based on winner
        let (title, message, title_color) = match event.winner {
            Some(Player::Max) => (
                "VICTORY!",
                if game_settings.versus_ai {
                    "You defeated the AI!"
                } else {
                    "Player 1 (Blue) Wins!"
                },
                colors.accent.clone(), // Victory color
            ),
            Some(Player::Min) => (
                if game_settings.versus_ai { "DEFEAT" } else { "VICTORY!" },
                if game_settings.versus_ai {
                    "The AI has won..."
                } else {
                    "Player 2 (Pink) Wins!"
                },
                if game_settings.versus_ai {
                    crate::ui::config::ColorData { r: 0.8, g: 0.2, b: 0.3, a: 1.0 } // Red for defeat
                } else {
                    crate::ui::config::ColorData { r: 1.0, g: 0.2, b: 0.8, a: 1.0 } // Pink for player 2 victory
                },
            ),
            None => (
                "DRAW",
                "The game ended in a draw",
                colors.text_secondary.clone(),
            ),
        };
        
        // Spawn game over overlay
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)), // Dark overlay
                ZIndex(100),
                GameOverOverlay,
                OnGameScreen,
            ))
            .with_children(|parent| {
                // Game over panel
                parent
                    .spawn((
                        Node {
                            width: Val::Px(600.0),
                            height: Val::Auto,
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            padding: UiRect::all(Val::Px(50.0)),
                            border: UiRect::all(Val::Px(3.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.05, 0.0, 0.15, 0.95)),
                        BorderColor(title_color.clone().into()),
                        BorderRadius::all(Val::Px(15.0)),
                    ))
                    .with_children(|parent| {
                        // Title
                        parent.spawn((
                            Text::new(title),
                            TextFont {
                                font_size: 64.0,
                                ..default()
                            },
                            TextColor(title_color.into()),
                            Node {
                                margin: UiRect::bottom(Val::Px(20.0)),
                                ..default()
                            },
                        ));
                        
                        // Message
                        parent.spawn((
                            Text::new(message),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(colors.text_primary.clone().into()),
                            Node {
                                margin: UiRect::bottom(Val::Px(40.0)),
                                ..default()
                            },
                        ));
                        
                        // Buttons container
                        parent
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    row_gap: Val::Px(15.0),
                                    ..default()
                                },
                            ))
                            .with_children(|parent| {
                                // Restart Game button
                                parent
                                    .spawn((
                                        Button,
                                        Node {
                                            width: Val::Px(300.0),
                                            height: Val::Px(60.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(2.0)),
                                            ..default()
                                        },
                                        BackgroundColor(colors.accent.clone().into()),
                                        BorderColor(colors.accent.clone().into()),
                                        GameOverAction::RestartGame,
                                    ))
                                    .with_children(|parent| {
                                        parent.spawn((
                                            Text::new("RESTART GAME"),
                                            TextFont {
                                                font_size: 20.0,
                                                ..default()
                                            },
                                            TextColor(colors.text_primary.clone().into()),
                                        ));
                                    });
                                
                                // Back to Menu button
                                parent
                                    .spawn((
                                        Button,
                                        Node {
                                            width: Val::Px(300.0),
                                            height: Val::Px(60.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(2.0)),
                                            ..default()
                                        },
                                        BackgroundColor(colors.button_normal.clone().into()),
                                        BorderColor(colors.secondary.clone().into()),
                                        GameOverAction::BackToMenu,
                                    ))
                                    .with_children(|parent| {
                                        parent.spawn((
                                            Text::new("BACK TO MENU"),
                                            TextFont {
                                                font_size: 20.0,
                                                ..default()
                                            },
                                            TextColor(colors.text_primary.clone().into()),
                                        ));
                                    });
                            });
                    });
            });
    }
}

fn handle_game_over_actions(
    mut commands: Commands,
    button_query: Query<(&Interaction, &GameOverAction), (Changed<Interaction>, With<Button>)>,
    overlay_query: Query<Entity, With<GameOverOverlay>>,
    mut app_state: ResMut<NextState<AppState>>,
    mut reset_board: EventWriter<ResetBoard>,
    config: Res<GameConfig>,
) {
    let colors = &config.colors;
    
    for (interaction, action) in button_query.iter() {
        if *interaction == Interaction::Pressed {
            match action {
                GameOverAction::RestartGame => {
                    info!("Restart Game button pressed!");
                    
                    // Despawn game over overlay
                    for entity in overlay_query.iter() {
                        commands.entity(entity).despawn_recursive();
                    }
                    
                    // Reset the board
                    reset_board.write(ResetBoard);
                }
                GameOverAction::BackToMenu => {
                    info!("Back to Menu button pressed from game over screen!");
                    app_state.set(AppState::Menu);
                }
            }
        }
        
        // Add hover effects
        if *interaction == Interaction::Hovered {
            // Handled by existing button_system if we want
        }
    }
}
