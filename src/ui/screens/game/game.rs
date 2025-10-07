use std::time::Duration;

use bevy::prelude::*;
use crate::{
    ai::lazy_smp::lazy_smp_search,
    audio::{PlayStonePlacementSound, PlayWinSound, PlayLoseSound},
    core::{board::Player, moves::RuleValidator, state::GameState}, 
    ui::{
        app::{AppState, GameSettings}, 
        config::GameConfig,
        components::button::{ButtonBuilder, ButtonStyle, ButtonSize, button_interaction_system},
        screens::{
            game::{
                board::{BoardRoot, BoardUtils, PreviewDot}, 
                settings::{spawn_settings_panel, BackToMenuButton, ResetBoardButton, VolumeDisplay, VolumeDown, VolumeUp}
            }, menu::GameAudio, splash::PreloadedStones, utils::despawn_screen
        },
    }
};

// Game status resource
#[derive(Resource, Default, PartialEq)]
pub enum GameStatus {
    #[default]
    AwaitingUserInput,
    AIThinking,  // New state to show AI is about to think
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

#[derive(Event)]
pub struct UpdatePlayerDisplay;

#[derive(Component)]
pub struct GameOverOverlay;

#[derive(Component)]
pub enum GameOverAction {
    Continue,     // Dismiss popup to review the board
    RestartGame,
    BackToMenu,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridCell {
    pub x: usize,
    pub y: usize,
}

#[derive(Component)]
pub struct CurrentPlayerText;

#[derive(Component)]
pub struct RoundNumberText;

#[derive(Component)]
pub struct Player1CapturesText;

#[derive(Component)]
pub struct Player2CapturesText;

pub fn game_plugin(app: &mut App) {
    app.init_resource::<GameStatus>()
        .init_resource::<AITimeTaken>()
        .init_resource::<AIDepthReached>()
        .init_resource::<AIThinkingFrames>()
        .add_event::<GameEnded>()
        .add_event::<StonePlacement>()
        .add_event::<MovePlayed>()
        .add_event::<UpdateAITimeDisplay>()
        .add_event::<UpdateAIDepthDisplay>()
        .add_event::<ResetBoard>()
        .add_event::<UpdatePlayerDisplay>()
        .add_systems(OnEnter(AppState::Game), (
            update_game_settings_from_config,
            setup_game_ui,
        ).chain())
        .add_systems(
            Update,
            (
                button_interaction_system,
                handle_player_placement,
                place_stone.run_if(on_event::<StonePlacement>),
                process_next_round.run_if(on_event::<MovePlayed>),
                handle_ai_turn,  // New system to handle AI thinking
                update_available_placement.run_if(on_event::<MovePlayed>),
                update_current_player_display.run_if(
                    resource_changed::<GameState>
                        .or(resource_changed::<GameStatus>)
                        .or(on_event::<UpdatePlayerDisplay>)
                ),
                update_round_number_display,
                update_captures_display,
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
    mut game_status: ResMut<GameStatus>,
    mut ai_time: ResMut<AITimeTaken>,
    mut ai_depth: ResMut<AIDepthReached>,
    mut move_played: EventWriter<MovePlayed>,
) {
    // Get current settings from config
    let (board_size, win_condition, ai_max_depth, ai_time_limit, pair_captures_to_win) = config.get_game_settings();
    
    println!("======================================");
    println!("   INITIALIZING NEW GAME SESSION");
    println!("======================================");
    println!("Game settings from config:");
    println!("  board_size: {}", board_size);
    println!("  win_condition: {}", win_condition);
    println!("  ai_max_depth: {:?}", ai_max_depth);
    println!("  ai_time_limit: {:?}", ai_time_limit);
    println!("  pair_captures_to_win: {}", pair_captures_to_win);
    
    // Convert AI parameters
    // If ai_max_depth is None (unlimited), default to depth 6 for reasonable performance
    let ai_depth_value = match ai_max_depth {
        Some(depth) => depth as i32,
        None => 6, // Unlimited depth defaults to 6 for safety
    };
    
    // Convert time_limit from Option<u64> to Option<usize>
    let time_limit = ai_time_limit.map(|ms| ms as usize);
    
    println!("  Converted ai_depth: {}", ai_depth_value);
    println!("  Converted time_limit: {:?}", time_limit);
    
    // Update GameSettings resource
    *game_settings = GameSettings {
        board_size: board_size as usize,
        total_capture_to_win: pair_captures_to_win as usize,
        minimum_chain_to_win: win_condition as usize,
        ai_depth: ai_depth_value,
        alpha_beta_enabled: true,
        versus_ai: true,
        time_limit,
    };
    
    // Create a new GameState with updated settings
    *game_state = GameState::new(game_settings.board_size, game_settings.minimum_chain_to_win, game_settings.total_capture_to_win);
    
    // RESET ALL GAME RESOURCES TO INITIAL STATE
    *game_status = GameStatus::AwaitingUserInput;
    ai_time.micros = 0;
    ai_depth.depth = 0;
    
    println!("\nInitial game state:");
    println!("  Current Player: {:?} (Player::Max = Human/Pink)", game_state.current_player);
    println!("  Game Status: AwaitingUserInput");
    println!("  Versus AI: {}", game_settings.versus_ai);
    println!("======================================\n");
    
    // Trigger board update to show available placements
    move_played.write(MovePlayed);
}

fn setup_game_ui(
    mut commands: Commands, 
    game_settings: Res<GameSettings>,
    config: Res<GameConfig>,
    game_state: Res<GameState>,
) {
    let colors = &config.colors;
    
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
            // Board container with title and round number
            builder.spawn((
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(20.0),
                    ..default()
                },
            )).with_children(|builder| {
                // Current player turn display (above board)
                let initial_text = if game_settings.versus_ai && game_state.current_player == Player::Min {
                    "AI is thinking..."
                } else if game_state.current_player == Player::Max {
                    "Your Turn (Pink)"
                } else {
                    "Player 2's Turn (Blue)"
                };
                
                builder.spawn((
                    Text::new(initial_text),
                    TextFont {
                        font_size: 32.0,
                        ..default()
                    },
                    TextColor(colors.accent.clone().into()),
                    CurrentPlayerText,
                    Node {
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    },
                ));
                
                // Board
                BoardUtils::spawn_board(builder, &game_settings);
                
                // Capture and round info display (below board)
                builder.spawn(Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceEvenly,
                    align_items: AlignItems::Center,
                    width: Val::Percent(100.0),
                    margin: UiRect::top(Val::Px(15.0)),
                    ..default()
                }).with_children(|builder| {
                    // Player 1 (Pink) captures box
                    builder.spawn((
                        Node {
                            padding: UiRect::all(Val::Px(12.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            min_width: Val::Px(120.0),
                            ..default()
                        },
                        BorderColor(crate::ui::config::ColorData { r: 1.0, g: 0.4, b: 0.7, a: 0.6 }.into()), // Pink border
                        BackgroundColor(crate::ui::config::ColorData { r: 0.15, g: 0.15, b: 0.2, a: 0.8 }.into()),
                    )).with_children(|builder| {
                        builder.spawn((
                            Text::new("You: 0"),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(crate::ui::config::ColorData { r: 1.0, g: 0.4, b: 0.7, a: 1.0 }.into()), // Pink
                            Player1CapturesText,
                        ));
                    });
                    
                    // Round number box (center)
                    builder.spawn((
                        Node {
                            padding: UiRect::all(Val::Px(12.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            min_width: Val::Px(120.0),
                            ..default()
                        },
                        BorderColor(colors.accent.clone().into()),
                        BackgroundColor(crate::ui::config::ColorData { r: 0.15, g: 0.15, b: 0.2, a: 0.8 }.into()),
                    )).with_children(|builder| {
                        builder.spawn((
                            Text::new("Round 1"),
                            TextFont {
                                font_size: 22.0,
                                ..default()
                            },
                            TextColor(colors.accent.clone().into()),
                            RoundNumberText,
                        ));
                    });
                    
                    // Player 2 (Blue) captures box
                    builder.spawn((
                        Node {
                            padding: UiRect::all(Val::Px(12.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            min_width: Val::Px(120.0),
                            ..default()
                        },
                        BorderColor(crate::ui::config::ColorData { r: 0.2, g: 0.6, b: 1.0, a: 0.6 }.into()), // Blue border
                        BackgroundColor(crate::ui::config::ColorData { r: 0.15, g: 0.15, b: 0.2, a: 0.8 }.into()),
                    )).with_children(|builder| {
                        builder.spawn((
                            Text::new("AI: 0"),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(crate::ui::config::ColorData { r: 0.2, g: 0.6, b: 1.0, a: 1.0 }.into()), // Blue
                            Player2CapturesText,
                        ));
                    });
                });
            });
            
            // Settings panel on the right
            spawn_settings_panel(builder, &game_settings, &config);
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
    mut stone_sound: EventWriter<PlayStonePlacementSound>,
    stones: Query<(Entity, &GridCell, &Stone)>,
) {
    for ev in ev_stone_placement.read() {
        info!("Stone placed at x: {}, y: {}", ev.x, ev.y);
        
        // Play stone placement sound (randomized)
        stone_sound.send(PlayStonePlacementSound);
        
        // Get the player BEFORE making the move (they're the one placing the stone)
        let player = game_state.current_player;
        
        // Now make the move (this will switch to the next player)
        game_state.make_move((ev.x, ev.y));

        // Despawn captured stones cleanly
        for (stone_entity, stone_cell, _) in stones.iter() {
            if game_state.board.is_empty_position(stone_cell.x, stone_cell.y) {
                info!("Despawning captured stone at x: {}, y: {}", stone_cell.x, stone_cell.y);
                commands.entity(stone_entity).despawn();
            }
        }

        // Spawn new stone
        if let Ok(board_entity) = board_query.single() {
            // In this codebase: Player::Max moves first (human player)
            // Traditional Gomoku: Black moves first
            // Color mapping: Player::Max (first player) = Pink, Player::Min (second player/AI) = Blue
            let is_first_player = player == Player::Max;
            
            commands.entity(board_entity).with_children(|builder| {
                // Use preloaded stone images
                let stone_handle = if is_first_player {
                    preloaded_stones.pink_stone.clone()  // Pink for first player (human)
                } else {
                    preloaded_stones.blue_stone.clone()  // Blue for second player (AI)
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
    mut game_event: EventWriter<GameEnded>,
    settings: Res<GameSettings>,
    game_state: Res<GameState>,
    mut game_status: ResMut<GameStatus>,
    mut win_sound: EventWriter<PlayWinSound>,
    mut lose_sound: EventWriter<PlayLoseSound>,
    mut player_text_query: Query<&mut Text, With<CurrentPlayerText>>,
) {
    for _ in move_played.read() {
        // Check for game end first
        if game_state.is_terminal() {
            let winner = game_state.check_winner();
            game_event.write(GameEnded { winner });
            *game_status = GameStatus::GameOver;
            
            match winner {
                Some(player) => {
                    println!("Game Over! Winner: {:?}", player);
                    if settings.versus_ai {
                        // In vs AI mode, check if human won or lost
                        if player == Player::Max {
                            // Human won (Player::Max)
                            win_sound.send(PlayWinSound);
                        } else {
                            // AI won (Player::Min)
                            lose_sound.send(PlayLoseSound);
                        }
                    } else {
                        // In multiplayer mode, always play win sound
                        win_sound.send(PlayWinSound);
                    }
                }
                None => {
                    println!("Game Over! It's a draw.");
                    // Could add a draw sound here if you want
                }
            }
            return;
        }

        // Handle next player's turn
        if game_state.current_player == Player::Max || (game_state.current_player == Player::Min && !settings.versus_ai) {
            info!("Awaiting user click");
            *game_status = GameStatus::AwaitingUserInput;
            // Update display to show it's the player's turn
            for mut text in player_text_query.iter_mut() {
                if settings.versus_ai {
                    text.0 = "Your Turn (Pink)".to_string();
                } else {
                    text.0 = if game_state.current_player == Player::Max {
                        "Player 1's Turn (Pink)".to_string()
                    } else {
                        "Player 2's Turn (Blue)".to_string()
                    };
                }
            }
        } else if settings.versus_ai {
            // AI's turn - set status and update display
            // The actual AI computation will happen in handle_ai_turn system next frame
            info!("AI's turn - setting AIThinking status");
            *game_status = GameStatus::AIThinking;
            
            // Update display immediately to show AI is thinking
            for mut text in player_text_query.iter_mut() {
                text.0 = "AI is thinking...".to_string();
            }
        }
    }
}

// New system: Handles the actual AI computation after ensuring at least one frame has rendered
fn handle_ai_turn(
    mut stone_placement: EventWriter<StonePlacement>,
    mut game_event: EventWriter<GameEnded>,
    settings: Res<GameSettings>,
    mut game_state: ResMut<GameState>,
    mut game_status: ResMut<GameStatus>,
    mut ai_time: ResMut<AITimeTaken>,
    mut ai_depth: ResMut<AIDepthReached>,
    mut update_ai_time: EventWriter<UpdateAITimeDisplay>,
    mut update_ai_depth: EventWriter<UpdateAIDepthDisplay>,
    mut ai_frames: ResMut<AIThinkingFrames>,
) {
    // Only run if AI is thinking
    if *game_status != GameStatus::AIThinking {
        return;
    }
    
    // Wait for at least 2 frames to guarantee UI has rendered
    // Frame 0: Status changes to AIThinking, text updates
    // Frame 1: UI renders with new text
    // Frame 2: AI computation starts
    ai_frames.frames_waited += 1;
    
    if ai_frames.frames_waited < 2 {
        // Still waiting for frames to render
        return;
    }
    
    // Reset frame counter for next time
    ai_frames.frames_waited = 0;
    
    info!("AI computation starting (after {} frames)...", ai_frames.frames_waited);
    
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
            info!("AI chose move: ({}, {})", x, y);
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
			text.0 = ai_depth.depth.to_string();
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

#[derive(Resource, Default)]
pub struct AIThinkingFrames {
    pub frames_waited: u32,
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
            GameStatus::AIThinking => {
                println!("Cannot pause while AI is thinking");
                GameStatus::AIThinking
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
            println!("Reset Board button clicked - sending reset event...");
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
    mut update_ai_time: EventWriter<UpdateAITimeDisplay>,
    mut update_ai_depth: EventWriter<UpdateAIDepthDisplay>,
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
        GameStatus::AIThinking => "AIThinking",
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
    
    println!("\nCreating fresh GameState...");
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
    
    // Trigger UI updates for AI stats
    update_ai_time.write(UpdateAITimeDisplay);
    update_ai_depth.write(UpdateAIDepthDisplay);
    
    // Print state AFTER reset
    println!("\n[STATE AFTER RESET]");
    println!("  Current Player: {:?}", game_state.current_player);
    println!("  Moves Played: {}", game_state.move_history.len());
    println!("  Game Status: {}", match *game_status {
        GameStatus::AwaitingUserInput => "AwaitingUserInput",
        GameStatus::AIThinking => "AIThinking",
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
                    "Player 1 (Pink) Wins!"
                },
                colors.accent.clone(), // Victory color
            ),
            Some(Player::Min) => (
                if game_settings.versus_ai { "DEFEAT" } else { "VICTORY!" },
                if game_settings.versus_ai {
                    "The AI has won..."
                } else {
                    "Player 2 (Blue) Wins!"
                },
                if game_settings.versus_ai {
                    crate::ui::config::ColorData { r: 0.8, g: 0.2, b: 0.3, a: 1.0 } // Red for defeat
                } else {
                    crate::ui::config::ColorData { r: 0.2, g: 0.6, b: 1.0, a: 1.0 } // Blue for player 2 victory
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
                                // Continue button to dismiss popup
                                ButtonBuilder::new("CONTINUE")
                                    .with_style(ButtonStyle::Success)
                                    .with_size(ButtonSize::ExtraLarge)
                                    .with_font_size(20.0)
                                    .spawn(parent, GameOverAction::Continue, colors);
                                
                                // Restart Game button
                                ButtonBuilder::new("RESTART GAME")
                                    .with_style(ButtonStyle::Primary)
                                    .with_size(ButtonSize::ExtraLarge)
                                    .with_font_size(20.0)
                                    .spawn(parent, GameOverAction::RestartGame, colors);
                                
                                // Back to Menu button
                                ButtonBuilder::new("BACK TO MENU")
                                    .with_style(ButtonStyle::Secondary)
                                    .with_size(ButtonSize::ExtraLarge)
                                    .with_font_size(20.0)
                                    .spawn(parent, GameOverAction::BackToMenu, colors);
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
) {
    for (interaction, action) in button_query.iter() {
        if *interaction == Interaction::Pressed {
            match action {
                GameOverAction::Continue => {
                    info!("Continue button pressed - dismissing game over overlay");
                    
                    // Despawn game over overlay to let user review the board
                    for entity in overlay_query.iter() {
                        commands.entity(entity).despawn();
                    }
                }
                GameOverAction::RestartGame => {
                    info!("Restart Game button pressed!");
                    
                    // Despawn game over overlay
                    for entity in overlay_query.iter() {
                        commands.entity(entity).despawn();
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
    }
}

fn update_current_player_display(
    game_state: Res<GameState>,
    game_settings: Res<GameSettings>,
    game_status: Res<GameStatus>,
    mut query: Query<&mut Text, With<CurrentPlayerText>>,
) {
    if !game_state.is_changed() && !game_status.is_changed() {
        return;
    }
    
    for mut text in query.iter_mut() {
        // Check if game is over
        if matches!(*game_status, GameStatus::GameOver) {
            // Determine winner message with win reason
            let message = if let Some(winner) = game_state.winner {
                let winner_name = match winner {
                    Player::Max => {
                        if game_settings.versus_ai {
                            "You Won!"
                        } else {
                            "Player 1 (Pink) Won!"
                        }
                    }
                    Player::Min => {
                        if game_settings.versus_ai {
                            "AI Won"
                        } else {
                            "Player 2 (Blue) Won!"
                        }
                    }
                };

                // Add win reason
                let reason = match game_state.win_reason {
                    Some(crate::core::state::WinReason::Alignment) => {
                        format!("by {} in a row!", game_state.win_condition)
                    }
                    Some(crate::core::state::WinReason::Captures) => {
                        format!("by capturing {} pairs!", game_state.capture_to_win)
                    }
                    None => "".to_string(),
                };

                format!("Game Over - {}\n{}", winner_name, reason)
            } else {
                "Game Over - Draw".to_string()
            };
            
            text.0 = message;
            continue;
        }
        
        // Determine the message based on current player and game mode
        let message = if game_settings.versus_ai {
            // vs AI mode
            // Player::Max = Human = Pink stones (first player)
            // Player::Min = AI = Blue stones (second player)
            if game_state.current_player == Player::Max {
                "Your Turn (Pink)".to_string()
            } else {
                "AI is thinking...".to_string()
            }
        } else {
            // Multiplayer mode
            // Player::Max = Pink stones (first player) 
            // Player::Min = Blue stones (second player)
            if game_state.current_player == Player::Max {
                "Player 1's Turn (Pink)".to_string()
            } else {
                "Player 2's Turn (Blue)".to_string()
            }
        };
        
        text.0 = message;
    }
}

fn update_round_number_display(
    game_state: Res<GameState>,
    mut query: Query<&mut Text, With<RoundNumberText>>,
) {
    if !game_state.is_changed() {
        return;
    }
    
    for mut text in query.iter_mut() {
        // Calculate round number (each round = 2 moves, one per player)
        let round_number = (game_state.move_history.len() / 2) + 1;
        text.0 = format!("Round {}", round_number);
    }
}

fn update_captures_display(
    game_state: Res<GameState>,
    game_settings: Res<GameSettings>,
    mut player1_query: Query<&mut Text, (With<Player1CapturesText>, Without<Player2CapturesText>)>,
    mut player2_query: Query<&mut Text, With<Player2CapturesText>>,
) {
    if !game_state.is_changed() {
        return;
    }
    
    // Update Player 1 (Pink/Max) captures
    for mut text in player1_query.iter_mut() {
        if game_settings.versus_ai {
            text.0 = format!("You: {}", game_state.max_captures);
        } else {
            text.0 = format!("Player 1: {}", game_state.max_captures);
        }
    }
    
    // Update Player 2 (Blue/Min) captures
    for mut text in player2_query.iter_mut() {
        if game_settings.versus_ai {
            text.0 = format!("AI: {}", game_state.min_captures);
        } else {
            text.0 = format!("Player 2: {}", game_state.min_captures);
        }
    }
}
