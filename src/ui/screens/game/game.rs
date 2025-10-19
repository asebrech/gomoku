use std::time::Instant;


use bevy::prelude::*;
use bevy::tasks::{Task, AsyncComputeTaskPool};
use futures_lite::future;
use bevy_gstreamer::camera::BackgroundImageMarker;
use gstreamer::prelude::*;
use gstreamer::Element;
use gstreamer_app::AppSink;
use crate::{
    ai::lazy_smp::{lazy_smp_search, SearchResult},
    audio::{PlayStonePlacementSound, PlayWinSound, PlayLoseSound},
    core::{board::Player, rules::{DoubleThreeDetection, CaptureBreaking}, state::GameState}, 
    ui::{
        app::{AppState, GameSettings}, 
        config::GameConfig,
        components::button::{ButtonBuilder, ButtonStyle, ButtonSize, button_interaction_system},
        screens::{
            game::{
                board::{BoardRoot, BoardUtils, PreviewDot, GhostStone}, 
                settings::{spawn_settings_panel, BackToMenuButton, ResetBoardButton, UndoMoveButton, VolumeDisplay, VolumeDown, VolumeUp}
            }, menu::{GameAudio, MenuState}, splash::PreloadedStones, utils::despawn_screen
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
struct GameGStreamerVideoBackground {
    video_path: String,
}

#[derive(Component)]
pub struct PersistentGameVideoBackground;

#[derive(Component)]
struct GameVideoFilePlayer {
    pipeline: Option<Element>,
    app_sink: Option<AppSink>,
    initialized: bool,
    image_handle: Option<Handle<Image>>,
    frame_timer: f32,
    frame_buffer: std::collections::VecDeque<Vec<u8>>,
    video_width: u32,
    video_height: u32,
}

impl Default for GameVideoFilePlayer {
    fn default() -> Self {
        Self {
            pipeline: None,
            app_sink: None,
            initialized: false,
            image_handle: None,
            frame_timer: 0.0,
            frame_buffer: std::collections::VecDeque::with_capacity(3),
            video_width: 0,
            video_height: 0,
        }
    }
}

impl Drop for GameVideoFilePlayer {
    fn drop(&mut self) {
        if let Some(ref pipeline) = self.pipeline {
            println!("GameVideoFilePlayer being dropped - stopping pipeline (initialized: {})", self.initialized);
            if let Err(e) = pipeline.set_state(gstreamer::State::Null) {
                eprintln!("Failed to stop game pipeline in Drop: {}", e);
            }
        }
        // Clear frame buffer to free memory
        self.frame_buffer.clear();
    }
}

#[derive(Component, Clone)]
pub struct OnGameScreen;
#[derive(Component)]
pub struct Stone(#[allow(dead_code)] Player);
#[derive(Component)]
pub struct AvailableArea;
#[derive(Component)]
pub struct ForbiddenMarker;
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

// AI vs AI mode components
#[derive(Component)]
pub struct NextMoveButton;

#[derive(Component)]
pub struct AutoPlayToggle;

#[derive(Resource, Default)]
pub struct AIvsAIState {
    pub auto_play: bool,
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
        .init_resource::<AIvsAIState>()
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
            show_persistent_game_video_background,
            update_available_placement, // Initialize forbidden markers on game start
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
                update_available_placement.run_if(
                    on_event::<MovePlayed>
                        .or(resource_changed::<GameState>)
                ),
                handle_ghost_stone_hover, // Run after update_available_placement
                update_current_player_display.run_if(
                    resource_changed::<GameState>
                        .or(resource_changed::<GameStatus>)
                        .or(resource_changed::<AIvsAIState>)
                        .or(on_event::<UpdatePlayerDisplay>)
                ),
                update_round_number_display,
                update_captures_display,
                reset_board.run_if(on_event::<ResetBoard>),
                toggle_pause,
                handle_escape_key,
            ).run_if(in_state(AppState::Game)),
        )
        .add_systems(
            Update,
            (
                // These video systems run always to keep persistent video working
                initialize_game_video_players,
                update_game_video_players,
                handle_game_video_looping,
            ).chain().run_if(any_with_component::<GameVideoFilePlayer>),
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
                handle_undo_move_button,
                handle_back_to_menu_button,
                handle_next_move_button,
                handle_auto_play_toggle,
                handle_ai_vs_ai_auto_play,
                show_game_over_screen.run_if(on_event::<GameEnded>),
                handle_game_over_actions,
            ).run_if(in_state(AppState::Game)),
        )
        .add_systems(OnExit(AppState::Game), (hide_persistent_game_video_background, despawn_screen::<OnGameScreen>));
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
    
    // Preserve the versus_ai and ai_vs_ai settings (set by menu)
    let current_versus_ai = game_settings.versus_ai;
    let current_ai_vs_ai = game_settings.ai_vs_ai;
    
    // Update GameSettings resource
    *game_settings = GameSettings {
        board_size: board_size as usize,
        total_capture_to_win: pair_captures_to_win as usize,
        minimum_chain_to_win: win_condition as usize,
        ai_depth: ai_depth_value,
        alpha_beta_enabled: true,
        versus_ai: current_versus_ai,  // Preserve menu selection
        ai_vs_ai: current_ai_vs_ai,  // Preserve menu selection
        time_limit,
    };
    
    // Create a new GameState with updated settings
    *game_state = GameState::new(game_settings.board_size, game_settings.minimum_chain_to_win);
    
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
    forbidden_markers: Query<Entity, With<ForbiddenMarker>>,
    board_query: Query<Entity, With<BoardRoot>>,
) {
    for _ in ev_board_update.read() {}

    // Clear all existing forbidden markers
    for marker_entity in forbidden_markers.iter() {
        commands.entity(marker_entity).despawn();
    }

    // Get the board entity
    let Ok(board_entity) = board_query.single() else {
        error!("Failed to find board entity");
        return;
    };

    info!("Updating stone preview...");
    
    let breaking_moves: Option<std::collections::HashSet<(usize, usize)>> = 
        if let Some(player_in_check) = game_state.player_in_check {
            if player_in_check == game_state.current_player.opponent() {
                if let Some(check_pos) = game_state.check_position {
                    let moves = CaptureBreaking::get_breaking_capture_moves(
                        &game_state.board, 
                        check_pos.0, 
                        check_pos.1, 
                        player_in_check
                    );
                    Some(moves.into_iter().collect())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
    
    for (entity, children, cell) in parents.iter() {
        let _is_valid = if let Some(ref breaking_set) = breaking_moves {
            breaking_set.contains(&(cell.x, cell.y))
        } else {
            game_state.board.is_empty_position(cell.x, cell.y)
                && !DoubleThreeDetection::creates_double_three(&game_state.board, cell.x, cell.y, game_state.current_player)
        };
        let is_empty = game_state.board.is_empty_position(cell.x, cell.y);
        let creates_double_three = DoubleThreeDetection::creates_double_three(&game_state.board, cell.x, cell.y, game_state.current_player);
        
        if is_empty && !creates_double_three {
            // Valid placement - show preview dot
            for &child in children {
                if let Ok((mut bg, mut visibility)) = dots.get_mut(child) {
                    *bg = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.4));
                    *visibility = Visibility::Visible;
                    commands.entity(entity).insert(AvailableArea);
                }
            }
        } else if is_empty && creates_double_three {
            // Forbidden placement - hide preview dot and show red cross
            for &child in children {
                if let Ok((mut bg, mut visibility)) = dots.get_mut(child) {
                    *bg = BackgroundColor(Color::NONE);
                    *visibility = Visibility::Hidden;
                    commands.entity(entity).remove::<AvailableArea>();
                }
            }
            // Spawn forbidden cross marker
            spawn_forbidden_cross(&mut commands, board_entity, cell.x, cell.y);
        } else {
            // Occupied position - hide preview dot
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

fn spawn_forbidden_cross(commands: &mut Commands, board_entity: Entity, cell_x: usize, cell_y: usize) {
    let line_thickness = 2.0; // Match tutorial exactly
    let cross_size = BoardUtils::STONE_SIZE * 0.7; // Match tutorial exactly
    
    // Calculate position relative to board (same as stone positioning)
    let cross_center_x = cell_x as f32 * BoardUtils::CELL_SIZE + BoardUtils::CELL_SIZE / 2.0;
    let cross_center_y = cell_y as f32 * BoardUtils::CELL_SIZE + BoardUtils::CELL_SIZE / 2.0;
    
    commands.entity(board_entity).with_children(|builder| {
        // First diagonal line (\)
        builder.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(cross_center_x - cross_size / 2.0),
                top: Val::Px(cross_center_y - line_thickness / 2.0),
                width: Val::Px(cross_size),
                height: Val::Px(line_thickness),
                ..default()
            },
            BackgroundColor(Color::srgb(1.0, 0.0, 0.0)), // Bright pure red
            Transform::from_rotation(Quat::from_rotation_z(std::f32::consts::PI / 4.0)), // 45 degrees
            ZIndex(20), // Above stones and board
            ForbiddenMarker,
            OnGameScreen,
        ));
        
        // Second diagonal line (/)
        builder.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(cross_center_x - cross_size / 2.0),
                top: Val::Px(cross_center_y - line_thickness / 2.0),
                width: Val::Px(cross_size),
                height: Val::Px(line_thickness),
                ..default()
            },
            BackgroundColor(Color::srgb(1.0, 0.0, 0.0)), // Bright pure red
            Transform::from_rotation(Quat::from_rotation_z(-std::f32::consts::PI / 4.0)), // -45 degrees
            ZIndex(20), // Above stones and board
            ForbiddenMarker,
            OnGameScreen,
        ));
    });
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
        stone_sound.write(PlayStonePlacementSound);
        
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
                if game_state.is_move_legal((cell.x, cell.y)) {
                    stone_placement.write(StonePlacement {
                        x: cell.x,
                        y: cell.y,
                    });
                } else {
                    info!("Illegal move attempted at ({}, {})", cell.x, cell.y);
                }
            }
        }
    }
}

pub fn handle_ghost_stone_hover(
    mut commands: Commands,
    mut ghost_stones: Query<(Entity, &mut Visibility, &mut BackgroundColor), With<GhostStone>>,
    mut preview_dots: Query<&mut Visibility, (With<PreviewDot>, Without<GhostStone>)>,
    interaction_query: Query<
        (&Interaction, &Children, &GridCell),
        (With<GridCell>, Without<Stone>, Changed<Interaction>),
    >,
    game_state: Res<GameState>,
    game_status: Res<GameStatus>,
    config: Res<GameConfig>,
    preloaded_stones: Res<PreloadedStones>,
) {
    if !matches!(*game_status, GameStatus::AwaitingUserInput) {
        // Hide all ghost stones when not awaiting input
        for (_, mut visibility, _) in ghost_stones.iter_mut() {
            *visibility = Visibility::Hidden;
        }
        return;
    }

    // Check if current theme is Synthwave
    let current_theme = config.get_current_theme();
    let is_synthwave = current_theme == "Synthwave";

    for (interaction, children, cell) in interaction_query.iter() {
        for child in children.iter() {
            // Handle ghost stone
            if let Ok((entity, mut ghost_visibility, mut background_color)) = ghost_stones.get_mut(child) {
                match *interaction {
                    Interaction::Hovered => {
                        // Check if this is a valid move position
                        let is_valid_move = game_state.board.is_empty_position(cell.x, cell.y)
                            && !DoubleThreeDetection::creates_double_three(&game_state.board, cell.x, cell.y, game_state.current_player)
                            && game_state.is_move_legal((cell.x, cell.y));
                        
                        if is_valid_move {
                            if is_synthwave {
                                // Use image assets for Synthwave theme
                                let stone_handle = if game_state.current_player == Player::Max {
                                    preloaded_stones.pink_stone.clone()
                                } else {
                                    preloaded_stones.blue_stone.clone()
                                };
                                
                                // Add ImageNode for the stone texture with transparency
                                commands.entity(entity)
                                    .insert(ImageNode {
                                        image: stone_handle,
                                        color: Color::srgba(1.0, 1.0, 1.0, 0.5), // Semi-transparent
                                        ..default()
                                    });
                                
                                // Clear background color for clean image display
                                *background_color = BackgroundColor(Color::NONE);
                            } else {
                                // Remove any ImageNode for color-based themes
                                commands.entity(entity).remove::<ImageNode>();
                                
                                // Use theme colors for other themes but semi-transparent
                                let player_color = if game_state.current_player == Player::Max {
                                    config.colors.primary.clone()
                                } else {
                                    config.colors.secondary.clone()
                                };
                                
                                *background_color = BackgroundColor(Color::srgba(
                                    player_color.r,
                                    player_color.g,
                                    player_color.b,
                                    0.4, // Semi-transparent
                                ));
                            }
                            *ghost_visibility = Visibility::Visible;
                            
                            // Hide the preview dot for this specific cell while showing ghost stone
                            for preview_child in children.iter() {
                                if let Ok(mut preview_visibility) = preview_dots.get_mut(preview_child) {
                                    *preview_visibility = Visibility::Hidden;
                                }
                            }
                        } else {
                            // Hide ghost stone for invalid moves
                            *ghost_visibility = Visibility::Hidden;
                        }
                    }
                    _ => {
                        // Hide ghost stone when not hovered
                        *ghost_visibility = Visibility::Hidden;
                        // Remove ImageNode when hiding to clean up
                        commands.entity(entity).remove::<ImageNode>();
                        
                        // Restore preview dot visibility when not hovering
                        for preview_child in children.iter() {
                            if let Ok(mut preview_visibility) = preview_dots.get_mut(preview_child) {
                                // Check if this position should have a preview dot
                                let should_show_preview = game_state.board.is_empty_position(cell.x, cell.y)
                                    && !DoubleThreeDetection::creates_double_three(&game_state.board, cell.x, cell.y, game_state.current_player)
                                    && game_state.is_move_legal((cell.x, cell.y));
                                
                                if should_show_preview {
                                    *preview_visibility = Visibility::Visible;
                                }
                            }
                        }
                    }
                }
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
    ai_vs_ai_state: Res<AIvsAIState>,
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
                            win_sound.write(PlayWinSound);
                        } else {
                            // AI won (Player::Min)
                            lose_sound.write(PlayLoseSound);
                        }
                    } else {
                        // In multiplayer mode, always play win sound
                        win_sound.write(PlayWinSound);
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
        if settings.ai_vs_ai {
            // AI vs AI mode - both players are AI
            // But only auto-advance if auto-play is enabled
            if ai_vs_ai_state.auto_play {
                info!("AI vs AI mode with auto-play - setting AIThinking status");
                *game_status = GameStatus::AIThinking;
            } else {
                info!("AI vs AI mode without auto-play - awaiting next move button");
                *game_status = GameStatus::AwaitingUserInput;
            }
        } else if game_state.current_player == Player::Max || (game_state.current_player == Player::Min && !settings.versus_ai) {
            info!("Awaiting user click");
            *game_status = GameStatus::AwaitingUserInput;
        } else if settings.versus_ai {
            // AI's turn - set status and update display
            // The actual AI computation will happen in handle_ai_turn system next frame
            info!("AI's turn - setting AIThinking status");
            *game_status = GameStatus::AIThinking;
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
    
    // Spawn the AI computation on the async compute thread pool
    let thread_pool = AsyncComputeTaskPool::get();
    let task = thread_pool.spawn(async move {
        let mut state = game_state_clone;
        info!("AI using Lazy SMP search with {}ms time limit and max depth {}", time_limit_ms, ai_depth);
        lazy_smp_search(&mut state, time_limit_ms as u64, ai_depth, None)
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
    mut menu_state: ResMut<NextState<MenuState>>,
    game_status: Res<GameStatus>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        // Don't allow going back to menu while AI is thinking
        if *game_status != GameStatus::AIThinking {
            app_state.set(AppState::Menu);
            menu_state.set(MenuState::Main);
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

fn handle_undo_move_button(
    button_query: Query<&Interaction, (Changed<Interaction>, With<UndoMoveButton>)>,
    mut game_state: ResMut<GameState>,
    game_status: Res<GameStatus>,
    stone_query: Query<Entity, With<Stone>>,
    mut commands: Commands,
    mut move_played: EventWriter<MovePlayed>,
    board_query: Query<Entity, With<BoardRoot>>,
    config: Res<GameConfig>,
    preloaded_stones: Option<Res<PreloadedStones>>,
) {
    for interaction in button_query.iter() {
        if *interaction == Interaction::Pressed {
            // Don't allow undo while AI is thinking
            if *game_status == GameStatus::AIThinking {
                println!("Cannot undo move while AI is thinking");
                return;
            }

            // Check if there are moves to undo
            if game_state.move_history.is_empty() {
                println!("No moves to undo");
                return;
            }

            println!("Undo Move button clicked - undoing last move...");
            
            // Get the last move
            if let Some(&last_move) = game_state.move_history.last() {
                // Undo the move in the game state first
                game_state.undo_move(last_move);
                
                // Clear all stones and resynchronize with the board state
                for stone_entity in stone_query.iter() {
                    commands.entity(stone_entity).despawn();
                }
                
                // Recreate all stones based on the current board state
                if let (Ok(board_entity), Some(preloaded_stones)) = (board_query.single(), preloaded_stones.as_deref()) {
                    let current_theme = config.get_current_theme();
                    let is_synthwave = current_theme == "Synthwave";
                    
                    for x in 0..game_state.board.size {
                        for y in 0..game_state.board.size {
                            if let Some(player) = game_state.board.get_player(x, y) {
                                let is_first_player = player == Player::Max;
                                
                                commands.entity(board_entity).with_children(|builder| {
                                    if is_synthwave {
                                        // Use image assets for Synthwave theme
                                        let stone_handle = if is_first_player {
                                            preloaded_stones.pink_stone.clone()
                                        } else {
                                            preloaded_stones.blue_stone.clone()
                                        };
                                        
                                        builder.spawn((
                                            BoardUtils::stone_node(x, y, BoardUtils::STONE_SIZE),
                                            ImageNode::new(stone_handle),
                                            Stone(player),
                                            ZIndex(15),
                                            OnGameScreen,
                                            GridCell { x, y },
                                        ));
                                    } else {
                                        // Use theme colors for other themes
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
                                            BoardUtils::stone_node(x, y, BoardUtils::STONE_SIZE),
                                            BackgroundColor(stone_color),
                                            BorderRadius::all(Val::Px(BoardUtils::STONE_SIZE / 2.0)),
                                            Stone(player),
                                            ZIndex(15),
                                            OnGameScreen,
                                            GridCell { x, y },
                                        ));
                                    }
                                });
                            }
                        }
                    }
                }
                
                // Trigger UI updates
                move_played.write(MovePlayed);
                
                println!("Successfully undid move at ({}, {})", last_move.0, last_move.1);
            }
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
        game_settings.minimum_chain_to_win
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
    mut menu_state: ResMut<NextState<MenuState>>,
) {
    for interaction in button_query.iter() {
        if *interaction == Interaction::Pressed {
            info!("Back to Menu button pressed!");
            app_state.set(AppState::Menu);
            menu_state.set(MenuState::Main);
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
    mut menu_state: ResMut<NextState<MenuState>>,
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
                    menu_state.set(MenuState::Main);
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
    ai_vs_ai_state: Res<AIvsAIState>,
    mut text_query: Query<&mut Text, With<CurrentPlayerText>>,
    mut circle_query: Query<&mut BackgroundColor, With<PlayerTurnCircle>>,
) {
    if !game_state.is_changed() && !game_status.is_changed() && !ai_vs_ai_state.is_changed() {
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
        let message = if game_settings.ai_vs_ai {
            // AI vs AI mode
            if *game_status == GameStatus::AIThinking {
                let ai_name = match game_state.current_player {
                    Player::Max => "AI 1",
                    Player::Min => "AI 2",
                };
                format!("{} is thinking...", ai_name)
            } else {
                let ai_name = match game_state.current_player {
                    Player::Max => "AI 1",
                    Player::Min => "AI 2",
                };
                if ai_vs_ai_state.auto_play {
                    format!("{}'s Turn", ai_name)
                } else {
                    format!("{}'s Turn (Press Next Move)", ai_name)
                }
            }
        } else if is_ai_turn {
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
    _existing_bg_query: Query<Entity, With<PersistentGameVideoBackground>>,
    mut existing_visibility_query: Query<&mut Visibility, With<PersistentGameVideoBackground>>,
) {

    
    // Skip in devMode
    if config.dev_mode {
        // Spawn a solid color background in dev mode
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
            BackgroundColor(config.colors.background.clone().into()),
            OnGameScreen,
        ));
        return;
    }

    // Check if we already have a persistent game video background
    if let Ok(mut visibility) = existing_visibility_query.single_mut() {
        *visibility = Visibility::Visible;
        return;
    }
    
    // Spawn a persistent GStreamer video background for the game
    let _entity = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        ZIndex(-2000), // Behind everything including the game board
        BackgroundColor(Color::srgb(1.0, 0.0, 0.0)), // Temporary red background for debugging
        ImageNode {
            image_mode: NodeImageMode::Stretch,
            ..default()
        }, // Will be updated by the video player
        GameGStreamerVideoBackground {
            video_path: "backgrounds/ingame-background/in-game.webm".to_string(),
        },
        GameVideoFilePlayer::default(),
        PersistentGameVideoBackground, // Mark as persistent
        Visibility::Visible, // Initially visible
    ));
}

fn initialize_game_video_players(
    mut video_players: Query<(Entity, &mut GameVideoFilePlayer, &GameGStreamerVideoBackground), (With<PersistentGameVideoBackground>, Without<BackgroundImageMarker>)>,
) {
    // Early return if no uninitialized players
    let has_uninitialized = video_players.iter().any(|(_, player, _)| !player.initialized);
    if !has_uninitialized {
        return;
    }
    
    // Try to initialize GStreamer if not already done - this is safe to call multiple times
    if let Err(_) = gstreamer::init() {
        return;
    }
    
    for (_entity, mut player, video_bg) in video_players.iter_mut() {
        if player.initialized {
            continue;
        }

        let video_path = format!("assets/{}", video_bg.video_path);
        let file_path = std::path::Path::new(&video_path);
        
        if !file_path.exists() {
            continue;
        }

        let uri = format!("file://{}", file_path.canonicalize().unwrap().to_string_lossy());
        
        let pipeline_description = format!(
            "uridecodebin uri={} ! videoconvert ! videoscale method=lanczos add-borders=false ! videorate ! video/x-raw,format=RGBA,framerate=30/1 ! appsink name=appsink sync=true drop=false max-buffers=1",
            uri
        );

        match gstreamer::parse::launch(&pipeline_description) {
            Ok(pipeline) => {
                let appsink = pipeline
                    .clone()
                    .downcast::<gstreamer::Pipeline>()
                    .unwrap()
                    .by_name("appsink")
                    .unwrap()
                    .downcast::<AppSink>()
                    .unwrap();

                player.pipeline = Some(pipeline.clone());
                player.app_sink = Some(appsink);
                player.initialized = true;

                let _ = pipeline.set_state(gstreamer::State::Playing);
            }
            Err(_) => {
                // Failed to create pipeline
            }
        }
    }
}

fn update_game_video_players(
    mut video_players: Query<(Entity, &mut GameVideoFilePlayer, &mut ImageNode), (With<GameGStreamerVideoBackground>, With<PersistentGameVideoBackground>, Without<BackgroundImageMarker>)>,
    mut images: ResMut<Assets<Image>>,
    time: Res<Time>,
) {
    for (_entity, mut player, mut image_node) in video_players.iter_mut() {
        if !player.initialized {
            continue;
        }

        // Stream new frames into buffer (but don't overwhelm it)
        if let Some(app_sink) = player.app_sink.as_ref().cloned() {
            // Fill buffer with available frames (max 3 frames)
            while player.frame_buffer.len() < 3 {
                if let Some(sample) = app_sink.try_pull_sample(gstreamer::ClockTime::from_mseconds(0)) {
                    
                    if let (Some(buffer), Some(caps)) = (sample.buffer(), sample.caps()) {
                        if let Ok(map) = buffer.map_readable() {
                            let data = map.as_slice();
                            let structure = caps.structure(0).unwrap();
                            let width = structure.get::<i32>("width").unwrap() as u32;
                            let height = structure.get::<i32>("height").unwrap() as u32;
                            
                            // Store video dimensions on first frame
                            if player.video_width == 0 {
                                player.video_width = width;
                                player.video_height = height;
                            }
                            
                            // Data is already RGBA from the pipeline
                            player.frame_buffer.push_back(data.to_vec());
                        }
                    }
                } else {
                    break; // No more frames available
                }
            }
        }
        
        // OPTIMIZED: Restore frame rate limiting - now we know the real issue was texture operations
        player.frame_timer += time.delta_secs();
        let should_update_frame = player.frame_timer >= 0.033 && !player.frame_buffer.is_empty(); // 30 FPS limit for smooth video
        
        if should_update_frame {
            player.frame_timer = 0.0; // Reset timer when updating
        }
        
        if should_update_frame { // 30 FPS updates
            
            // Get next frame from buffer
            if let Some(rgba_data) = player.frame_buffer.pop_front() {
                if player.image_handle.is_none() {
                    // Create texture for the first time
                    let bevy_image = Image::new_fill(
                        bevy::render::render_resource::Extent3d {
                            width: player.video_width,
                            height: player.video_height,
                            depth_or_array_layers: 1,
                        },
                        bevy::render::render_resource::TextureDimension::D2,
                        &rgba_data,
                        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                        bevy::render::render_asset::RenderAssetUsages::MAIN_WORLD | bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
                    );
                    
                    let new_handle = images.add(bevy_image);
                    player.image_handle = Some(new_handle.clone());
                    image_node.image = new_handle;
                } else {
                    // Update texture with buffered frame data
                    if let Some(ref handle) = player.image_handle {
                        let updated_image = Image::new_fill(
                            bevy::render::render_resource::Extent3d {
                                width: player.video_width,
                                height: player.video_height,
                                depth_or_array_layers: 1,
                            },
                            bevy::render::render_resource::TextureDimension::D2,
                            &rgba_data,
                            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                            bevy::render::render_asset::RenderAssetUsages::MAIN_WORLD | bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
                        );
                        
                        images.insert(handle, updated_image);
                    }
                }
            }
        }
    }
}

fn handle_game_video_looping(
    mut video_players: Query<&mut GameVideoFilePlayer, (With<GameGStreamerVideoBackground>, With<PersistentGameVideoBackground>)>,
) {
    for player in video_players.iter_mut() {
        if !player.initialized {
            continue;
        }

        let Some(ref pipeline) = player.pipeline else {
            continue;
        };
        
        // Check if the pipeline has reached the end
        if let Some(bus) = pipeline.bus() {
            while let Some(msg) = bus.timed_pop(gstreamer::ClockTime::from_mseconds(0)) {
                match msg.view() {
                    gstreamer::MessageView::Eos(_) => {
                        println!("[GAME VIDEO] Video reached end, restarting...");
                        if let Err(e) = pipeline.seek_simple(
                            gstreamer::SeekFlags::FLUSH | gstreamer::SeekFlags::KEY_UNIT,
                            gstreamer::ClockTime::ZERO,
                        ) {
                            println!("[GAME VIDEO] Failed to seek to beginning: {}", e);
                        }
                    }
                    gstreamer::MessageView::Error(err) => {
                        println!("[GAME VIDEO] Pipeline error: {}", err.error());
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Hide persistent game video background when exiting the game state
fn hide_persistent_game_video_background(
    mut video_query: Query<&mut Visibility, With<PersistentGameVideoBackground>>,
) {
    for mut visibility in video_query.iter_mut() {
        *visibility = Visibility::Hidden;
    }
}

fn show_persistent_game_video_background(
    mut video_query: Query<&mut Visibility, With<PersistentGameVideoBackground>>,
) {
    for mut visibility in video_query.iter_mut() {
        *visibility = Visibility::Visible;
    }
}

/// Preload game video background during menu to avoid flash screens
pub fn preload_game_video_background(
    mut commands: Commands,
    config: Res<GameConfig>,
    existing_bg_query: Query<Entity, With<PersistentGameVideoBackground>>,
) {
    // Skip in devMode or if already exists
    if config.dev_mode || !existing_bg_query.is_empty() {
        return;
    }
    
    // Spawn a hidden persistent GStreamer video background for the game
    let _entity = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        ZIndex(-2000), // Behind everything including the game board
        BackgroundColor(Color::srgb(1.0, 0.0, 0.0)), // Temporary red background for debugging
        ImageNode {
            image_mode: NodeImageMode::Stretch,
            ..default()
        }, // Will be updated by the video player
        GameGStreamerVideoBackground {
            video_path: "backgrounds/ingame-background/in-game.webm".to_string(),
        },
        GameVideoFilePlayer::default(),
        PersistentGameVideoBackground, // Mark as persistent
        Visibility::Hidden, // Initially hidden
    ));
}

// AI vs AI button handlers
fn handle_next_move_button(
    button_query: Query<&Interaction, (Changed<Interaction>, With<NextMoveButton>)>,
    game_settings: Res<GameSettings>,
    mut game_status: ResMut<GameStatus>,
    ai_vs_ai_state: Res<AIvsAIState>,
) {
    for interaction in button_query.iter() {
        if *interaction == Interaction::Pressed && game_settings.ai_vs_ai {
            // Only allow next move if waiting for user input and it's AI vs AI mode
            if *game_status == GameStatus::AwaitingUserInput && !ai_vs_ai_state.auto_play {
                // Trigger AI move by setting the status to AI thinking
                info!("Next Move button clicked in AI vs AI mode");
                *game_status = GameStatus::AIThinking;
            }
        }
    }
}

fn handle_auto_play_toggle(
    mut interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<AutoPlayToggle>)>,
    mut text_query: Query<&mut Text>,
    mut ai_vs_ai_state: ResMut<AIvsAIState>,
    game_settings: Res<GameSettings>,
    config: Res<GameConfig>,
) {
    for (interaction, children) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed && game_settings.ai_vs_ai {
            ai_vs_ai_state.auto_play = !ai_vs_ai_state.auto_play;
            info!("Auto Play toggled: {}", ai_vs_ai_state.auto_play);
            
            // Update button text
            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    text.0 = if ai_vs_ai_state.auto_play {
                        "AUTO PLAY: ON".to_string()
                    } else {
                        "AUTO PLAY: OFF".to_string()
                    };
                }
            }
        }
    }
}

fn handle_ai_vs_ai_auto_play(
    game_settings: Res<GameSettings>,
    ai_vs_ai_state: Res<AIvsAIState>,
    mut game_status: ResMut<GameStatus>,
    time: Res<Time>,
    mut last_move_time: Local<f32>,
) {
    if game_settings.ai_vs_ai && ai_vs_ai_state.auto_play {
        if *game_status == GameStatus::AwaitingUserInput {
            // Wait a bit between moves for visibility
            *last_move_time += time.delta_secs();
            if *last_move_time >= 1.5 { // 1.5 second delay between auto moves
                *last_move_time = 0.0;
                // Trigger next AI move
                *game_status = GameStatus::AIThinking;
                info!("Auto play triggering next AI move");
            }
        }
    } else {
        // Reset timer when auto play is off
        *last_move_time = 0.0;
    }
}




