use std::time::Instant;

use bevy::prelude::*;
use bevy::tasks::{Task, AsyncComputeTaskPool};
use futures_lite::future;
use crate::{
    ai::lazy_smp::{lazy_smp_search, SearchResult},
    audio::{PlayStonePlacementSound, PlayWinSound, PlayLoseSound},
    core::{board::Player, rules::GameRules, state::GameState}, 
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

#[derive(Component)]
struct GameVideoBackground {
    current_frame: usize,
    timer: Timer,
    total_frames: usize,
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
pub struct CurrentPlayerIndicator;

#[derive(Component)]
pub struct PlayerTurnCircle;

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
        .init_resource::<AINodesSearched>()
        .init_resource::<AIThinkingFrames>()
        .add_event::<GameEnded>()
        .add_event::<StonePlacement>()
        .add_event::<MovePlayed>()
        .add_event::<UpdateAITimeDisplay>()
        .add_event::<UpdateAIDepthDisplay>()
        .add_event::<UpdateAINodesDisplay>()
        .add_event::<ResetBoard>()
        .add_event::<UpdatePlayerDisplay>()
        .add_systems(OnEnter(AppState::Game), (
            update_game_settings_from_config,
            setup_game_ui,
            setup_game_background,
        ).chain())
        .add_systems(
            Update,
            (
                button_interaction_system,
                handle_player_placement,
                place_stone.run_if(on_event::<StonePlacement>),
                process_next_round.run_if(on_event::<MovePlayed>),
                start_ai_computation,  // Start async AI computation
                poll_ai_computation,   // Poll for AI computation results
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
                handle_escape_key,
                animate_game_background,
            ).run_if(in_state(AppState::Game)),
        )
        .add_systems(
            Update,
            (
                update_ai_time_display.run_if(on_event::<UpdateAITimeDisplay>),
                update_ai_time_realtime,  // Real-time AI timer update (runs every frame)
                update_ai_depth_display.run_if(on_event::<UpdateAIDepthDisplay>),
                update_ai_nodes_display.run_if(on_event::<UpdateAINodesDisplay>),
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
    mut ai_nodes: ResMut<AINodesSearched>,
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
    
    // Preserve the versus_ai setting (set by menu)
    let current_versus_ai = game_settings.versus_ai;
    
    // Update GameSettings resource
    *game_settings = GameSettings {
        board_size: board_size as usize,
        total_capture_to_win: pair_captures_to_win as usize,
        minimum_chain_to_win: win_condition as usize,
        ai_depth: ai_depth_value,
        alpha_beta_enabled: true,
        versus_ai: current_versus_ai,  // Preserve menu selection
        time_limit,
    };
    
    // Create a new GameState with updated settings
    *game_state = GameState::new(game_settings.board_size, game_settings.minimum_chain_to_win, game_settings.total_capture_to_win);
    
    // RESET ALL GAME RESOURCES TO INITIAL STATE
    *game_status = GameStatus::AwaitingUserInput;
    ai_time.micros = 0;
    ai_depth.depth = 0;
    ai_nodes.nodes = 0;
    
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
            BackgroundColor(Color::NONE), // Transparent to show animated background
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
                // Current player turn display (above board) with colored circle
                let is_ai_turn = game_settings.versus_ai && game_state.current_player == Player::Min;
                let is_player1 = game_state.current_player == Player::Max;
                
                builder.spawn((
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(12.0),
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    },
                    CurrentPlayerIndicator,
                )).with_children(|builder| {
                    if !is_ai_turn {
                        // Show colored circle for player turns
                        builder.spawn((
                            Node {
                                width: Val::Px(24.0),
                                height: Val::Px(24.0),
                                ..default()
                            },
                            BackgroundColor(if is_player1 {
                                Color::srgba(
                                    colors.stone_player1.r,
                                    colors.stone_player1.g,
                                    colors.stone_player1.b,
                                    colors.stone_player1.a,
                                )
                            } else {
                                Color::srgba(
                                    colors.stone_player2.r,
                                    colors.stone_player2.g,
                                    colors.stone_player2.b,
                                    colors.stone_player2.a,
                                )
                            }),
                            BorderRadius::all(Val::Percent(50.0)),
                            PlayerTurnCircle,
                        ));
                    }
                    
                    // Turn text
                    let initial_text = if is_ai_turn {
                        "AI is thinking..."
                    } else if game_settings.versus_ai {
                        "Your Turn"
                    } else {
                        "Turn"
                    };
                    
                    builder.spawn((
                        Text::new(initial_text),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(colors.accent.clone().into()),
                        CurrentPlayerText,
                    ));
                });
                
                // Board
                BoardUtils::spawn_board(builder, &game_settings, &config);
                
                // Capture info display (below board)
                builder.spawn(Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    width: Val::Percent(100.0),
                    margin: UiRect::top(Val::Px(15.0)),
                    row_gap: Val::Px(10.0),
                    ..default()
                }).with_children(|builder| {
                    // Score display box
                    builder.spawn((
                        Node {
                            padding: UiRect::all(Val::Px(15.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            min_width: Val::Px(300.0),
                            ..default()
                        },
                        BorderColor(colors.accent.clone().into()),
                        BackgroundColor(Color::srgba(
                            colors.surface.r * 0.8,
                            colors.surface.g * 0.8,
                            colors.surface.b * 0.8,
                            0.8
                        )),
                    )).with_children(|builder| {
                        // Score display with colored circles
                        builder.spawn(Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(15.0),
                            ..default()
                        }).with_children(|builder| {
                            // Player 1 circle
                            builder.spawn((
                                Node {
                                    width: Val::Px(20.0),
                                    height: Val::Px(20.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(
                                    colors.stone_player1.r,
                                    colors.stone_player1.g,
                                    colors.stone_player1.b,
                                    colors.stone_player1.a,
                                )),
                                BorderRadius::all(Val::Percent(50.0)),
                            ));
                            
                            // Player 1 score
                            builder.spawn((
                                Text::new("0"),
                                TextFont {
                                    font_size: 24.0,
                                    ..default()
                                },
                                TextColor(colors.text_primary.clone().into()),
                                Player1CapturesText,
                            ));
                            
                            // Separator
                            builder.spawn((
                                Text::new("-"),
                                TextFont {
                                    font_size: 24.0,
                                    ..default()
                                },
                                TextColor(colors.text_primary.clone().into()),
                            ));
                            
                            // Player 2 score
                            builder.spawn((
                                Text::new("0"),
                                TextFont {
                                    font_size: 24.0,
                                    ..default()
                                },
                                TextColor(colors.text_primary.clone().into()),
                                Player2CapturesText,
                            ));
                            
                            // Player 2 circle
                            builder.spawn((
                                Node {
                                    width: Val::Px(20.0),
                                    height: Val::Px(20.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(
                                    colors.stone_player2.r,
                                    colors.stone_player2.g,
                                    colors.stone_player2.b,
                                    colors.stone_player2.a,
                                )),
                                BorderRadius::all(Val::Percent(50.0)),
                            ));
                        });
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
            && !GameRules::creates_double_three(&game_state.board, cell.x, cell.y, game_state.current_player);
        
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
    config: Res<GameConfig>,
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
            
            // Check if current theme is Synthwave
            let current_theme = config.get_current_theme();
            let is_synthwave = current_theme == "Synthwave";
            
            commands.entity(board_entity).with_children(|builder| {
                if is_synthwave {
                    // Use image assets for Synthwave theme
                    let stone_handle = if is_first_player {
                        preloaded_stones.pink_stone.clone()
                    } else {
                        preloaded_stones.blue_stone.clone()
                    };
                    
                    builder.spawn((
                        BoardUtils::stone_node(ev.x, ev.y, BoardUtils::STONE_SIZE),
                        ImageNode::new(stone_handle),
                        Stone(player),
                        ZIndex(15),
                        OnGameScreen,
                        GridCell { x: ev.x, y: ev.y },
                    ));
                } else {
                    // Use theme colors for other themes - circular UI nodes
                    let stone_color = if is_first_player {
                        Color::srgba(
                            config.colors.stone_player1.r,
                            config.colors.stone_player1.g,
                            config.colors.stone_player1.b,
                            config.colors.stone_player1.a,
                        )
                    } else {
                        Color::srgba(
                            config.colors.stone_player2.r,
                            config.colors.stone_player2.g,
                            config.colors.stone_player2.b,
                            config.colors.stone_player2.a,
                        )
                    };
                    
                    builder.spawn((
                        BoardUtils::stone_node(ev.x, ev.y, BoardUtils::STONE_SIZE),
                        BackgroundColor(stone_color),
                        BorderRadius::all(Val::Percent(50.0)), // Perfect circle
                        Stone(player),
                        ZIndex(15),
                        OnGameScreen,
                        GridCell { x: ev.x, y: ev.y },
                    ));
                }
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

// New system: Start AI computation as an async task
fn start_ai_computation(
    mut commands: Commands,
    settings: Res<GameSettings>,
    game_state: Res<GameState>,
    game_status: Res<GameStatus>,
    mut ai_frames: ResMut<AIThinkingFrames>,
    existing_task: Option<Res<AIComputeTask>>,
    mut query: Query<&mut Text, With<CurrentPlayerText>>,
) {
    // Only run if versus AI is enabled
    if !settings.versus_ai {
        return;
    }
    
    // Only run if AI is thinking
    if *game_status != GameStatus::AIThinking {
        return;
    }
    
    // Don't start a new task if one is already running
    if existing_task.is_some() {
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
    
    info!("Starting async AI computation...");
    
    // Update display immediately to show AI is thinking
    for mut text in query.iter_mut() {
        text.0 = "AI is thinking...".to_string();
    }
    
    // Record the start time for real-time timer updates
    commands.insert_resource(AIThinkingStartTime(Instant::now()));
    
    // Clone the data we need for the task
    let game_state_clone = game_state.clone();
    let ai_depth = settings.ai_depth;
    let time_limit_ms = settings.time_limit.unwrap_or(500); // Default 500ms if not set
    // Reduce actual time given to AI by 25ms to ensure it stays within limit
    let actual_time_ms = time_limit_ms.saturating_sub(25);
    
    // Spawn the AI computation on the async compute thread pool
    let thread_pool = AsyncComputeTaskPool::get();
    let task = thread_pool.spawn(async move {
        let mut state = game_state_clone;
        info!("AI using Lazy SMP search with {}ms time limit (actual: {}ms) and max depth {}", time_limit_ms, actual_time_ms, ai_depth);
        lazy_smp_search(&mut state, actual_time_ms as u64, ai_depth, None)
    });
    
    // Store the task as a resource
    commands.insert_resource(AIComputeTask(task));
}

// New system: Poll the AI computation task for results
fn poll_ai_computation(
    mut commands: Commands,
    mut stone_placement: EventWriter<StonePlacement>,
    mut game_event: EventWriter<GameEnded>,
    mut game_status: ResMut<GameStatus>,
    mut ai_time: ResMut<AITimeTaken>,
    mut ai_depth: ResMut<AIDepthReached>,
    mut ai_nodes: ResMut<AINodesSearched>,
    mut update_ai_time: EventWriter<UpdateAITimeDisplay>,
    mut update_ai_depth: EventWriter<UpdateAIDepthDisplay>,
    mut update_ai_nodes: EventWriter<UpdateAINodesDisplay>,
    task: Option<ResMut<AIComputeTask>>,
) {
    // Only run if we have an active task
    let Some(mut task_res) = task else {
        return;
    };
    
    // Poll the task to see if it's complete
    if let Some(result) = future::block_on(future::poll_once(&mut task_res.0)) {
        info!("AI computation complete!");
        
        // Update AI statistics
        ai_time.micros = result.time_elapsed.as_micros();
        ai_depth.depth = result.depth_reached;
        ai_nodes.nodes = result.nodes_searched;
        update_ai_time.write(UpdateAITimeDisplay);
        update_ai_depth.write(UpdateAIDepthDisplay);
        update_ai_nodes.write(UpdateAINodesDisplay);
        
        // Handle the result
        if let Some((x, y)) = result.best_move {
            info!("AI chose move: ({}, {})", x, y);
            stone_placement.write(StonePlacement { x, y });
            *game_status = GameStatus::AwaitingUserInput;
        } else {
            // AI has no moves but game isn't terminal - this shouldn't happen
            println!("AI has no valid moves available");
            game_event.write(GameEnded { winner: None });
            *game_status = GameStatus::GameOver;
        }
        
        // Remove the task resource now that it's complete
        commands.remove_resource::<AIComputeTask>();
        commands.remove_resource::<AIThinkingStartTime>();
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

/// Update AI time display in real-time while AI is thinking (runs every frame)
pub fn update_ai_time_realtime(
    mut query: Query<&mut Text, With<AITimeText>>,
    start_time: Option<Res<AIThinkingStartTime>>,
    game_status: Res<GameStatus>,
) {
    // Only update while AI is actively thinking
    if *game_status != GameStatus::AIThinking {
        return;
    }
    
    // Only update if we have a start time
    if let Some(start) = start_time {
        let elapsed = start.0.elapsed();
        let time_ms = elapsed.as_secs_f64() * 1000.0;
        
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

pub fn update_ai_nodes_display(
    mut query: Query<&mut Text, With<AINodesText>>,
    ai_nodes: Res<AINodesSearched>,
    mut events: EventReader<UpdateAINodesDisplay>,
) {
    for _ in events.read() {
        // Format nodes with comma separators for readability
        let formatted_nodes = format_number_with_commas(ai_nodes.nodes);
        info!("Updating AI nodes display: {} nodes", formatted_nodes);
        for mut text in query.iter_mut() {
            text.0 = formatted_nodes.clone();
        }
    }
}

fn format_number_with_commas(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    
    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*ch);
    }
    
    result
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

#[derive(Component)]
pub struct AINodesText;

#[derive(Resource, Default)]
pub struct AINodesSearched {
    pub nodes: u64,
}

#[derive(Event)]
pub struct UpdateAINodesDisplay;

#[derive(Resource, Default)]
pub struct AIThinkingFrames {
    pub frames_waited: u32,
}

/// Resource to hold the async AI computation task
#[derive(Resource)]
pub struct AIComputeTask(Task<SearchResult>);

/// Resource to track when AI started thinking (for real-time timer)
#[derive(Resource)]
pub struct AIThinkingStartTime(Instant);

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

pub fn handle_escape_key(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_state: ResMut<NextState<AppState>>,
    game_status: Res<GameStatus>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        // Don't allow going back to menu while AI is thinking
        if *game_status != GameStatus::AIThinking {
            app_state.set(AppState::Menu);
        }
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
                    colors.secondary.clone() // Use secondary color for AI defeat
                } else {
                    colors.secondary.clone() // Use secondary color for player 2 victory
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
                BackgroundColor(Color::srgba(
                    colors.background.r * 0.5,
                    colors.background.g * 0.5,
                    colors.background.b * 0.5,
                    0.8
                )),
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
                        BackgroundColor(Color::srgba(
                            colors.surface.r * 0.9,
                            colors.surface.g * 0.9,
                            colors.surface.b * 0.9,
                            0.95
                        )),
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
    config: Res<GameConfig>,
    mut text_query: Query<&mut Text, With<CurrentPlayerText>>,
    mut circle_query: Query<&mut BackgroundColor, With<PlayerTurnCircle>>,
) {
    if !game_state.is_changed() && !game_status.is_changed() {
        return;
    }
    
    let colors = &config.colors;
    
    for mut text in text_query.iter_mut() {
        // Check if game is over
        if matches!(*game_status, GameStatus::GameOver) {
            // Hide the circle when game is over
            for mut circle_bg in circle_query.iter_mut() {
                *circle_bg = BackgroundColor(Color::NONE);
            }
            
            // Determine winner message with win reason
            let message = if let Some(winner) = game_state.winner {
                let winner_name = match winner {
                    Player::Max => {
                        if game_settings.versus_ai {
                            "You Won!"
                        } else {
                            "Player 1 Won!"
                        }
                    }
                    Player::Min => {
                        if game_settings.versus_ai {
                            "AI Won"
                        } else {
                            "Player 2 Won!"
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
        
        // Update circle color based on current player
        let is_ai_turn = game_settings.versus_ai && game_state.current_player == Player::Min;
        let is_player1 = game_state.current_player == Player::Max;
        
        for mut circle_bg in circle_query.iter_mut() {
            if is_ai_turn {
                // Hide circle during AI turn
                *circle_bg = BackgroundColor(Color::NONE);
            } else {
                // Show colored circle for player turns
                *circle_bg = BackgroundColor(if is_player1 {
                    Color::srgba(
                        colors.stone_player1.r,
                        colors.stone_player1.g,
                        colors.stone_player1.b,
                        colors.stone_player1.a,
                    )
                } else {
                    Color::srgba(
                        colors.stone_player2.r,
                        colors.stone_player2.g,
                        colors.stone_player2.b,
                        colors.stone_player2.a,
                    )
                });
            }
        }
        
        // Determine the message based on current player and game mode
        let message = if is_ai_turn {
            "AI is thinking...".to_string()
        } else if game_settings.versus_ai {
            // vs AI: Just say "Your Turn" since circle shows the color
            "Your Turn".to_string()
        } else {
            // Multiplayer: Show "Turn" with the circle indicating which player
            "Turn".to_string()
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
        // Calculate turn number (total moves made)
        let turn_number = (game_state.move_history.len() / 2) + 1;
        text.0 = format!("Turns: {}", turn_number);
    }
}

fn update_captures_display(
    game_state: Res<GameState>,
    mut player1_query: Query<&mut Text, (With<Player1CapturesText>, Without<Player2CapturesText>)>,
    mut player2_query: Query<&mut Text, With<Player2CapturesText>>,
) {
    if !game_state.is_changed() {
        return;
    }
    
    // Update player 1 score
    for mut text in player1_query.iter_mut() {
        text.0 = format!("{}", game_state.max_captures);
    }
    
    // Update player 2 score
    for mut text in player2_query.iter_mut() {
        text.0 = format!("{}", game_state.min_captures);
    }
}

fn setup_game_background(
    mut commands: Commands,
    config: Res<GameConfig>,
    game_bg_frames: Option<Res<crate::ui::screens::menu::GameBackgroundFrames>>,
) {
    println!("[GAME BACKGROUND] Setting up game background...");
    
    // Skip in devMode
    if config.dev_mode {
        println!("[GAME BACKGROUND] Skipping in devMode");
        return;
    }

    // Get frames from the loading screen resource
    let Some(bg_frames) = game_bg_frames else {
        println!("[GAME BACKGROUND] ERROR: No background frames resource found!");
        return;
    };

    println!("[GAME BACKGROUND] Found {} frames", bg_frames.frames.len());

    if bg_frames.frames.is_empty() {
        println!("[GAME BACKGROUND] ERROR: Frames vector is empty!");
        return;
    }

    let animation_config = &config.assets.animations.game_background_frames;
    let fps = animation_config.fps as f32;
    let frame_duration = 1.0 / fps;

    println!("[GAME BACKGROUND] Spawning background with {} frames at {} fps", bg_frames.frames.len(), fps);

    // Spawn background as a full-screen image behind everything
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            ..default()
        },
        ZIndex(-1000), // Behind everything
        ImageNode {
            image: bg_frames.frames[0].clone(),
            ..default()
        },
        GameVideoBackground {
            current_frame: 0,
            timer: Timer::from_seconds(frame_duration, TimerMode::Repeating),
            total_frames: bg_frames.frames.len(),
        },
        OnGameScreen,
    ));
    
    println!("[GAME BACKGROUND] Background entity spawned successfully!");
}

fn animate_game_background(
    time: Res<Time>,
    mut video_backgrounds: Query<(&mut GameVideoBackground, &mut ImageNode)>,
    game_bg_frames: Option<Res<crate::ui::screens::menu::GameBackgroundFrames>>,
    config: Res<GameConfig>,
) {
    // Skip animation in devMode
    if config.dev_mode {
        return;
    }

    if let Some(frames) = game_bg_frames {
        for (mut video_bg, mut image_node) in video_backgrounds.iter_mut() {
            video_bg.timer.tick(time.delta());

            if video_bg.timer.just_finished() {
                video_bg.current_frame = (video_bg.current_frame + 1) % video_bg.total_frames;

                if video_bg.current_frame < frames.frames.len() {
                    image_node.image = frames.frames[video_bg.current_frame].clone();
                }
            }
        }
    }
}
