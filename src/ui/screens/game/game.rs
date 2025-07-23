use std::time::{Duration, Instant};

use bevy::prelude::*;
use crate::{ai::transposition::{TranspositionTable, SharedTranspositionTable}, core::{board::Player, state::GameState}, interface::utils::{find_best_move, find_best_move_parallel, find_best_move_parallel_enhanced}, ui::{app::{AppState, GameSettings}, screens::{game::{board::{BoardRoot, BoardUtils, PreviewDot}, settings::{spawn_settings_panel, SettingsChanged, SettingsBoardSizeButton, SettingsChainToWinButton, SettingsCaptureToWinButton, SettingsAIDepthButton, SettingsTimeLimitButton, SettingsTimeLimitToggleButton, BoardSizeText, ChainToWinText, CaptureToWinText, AIDepthText, TimeLimitText}}, utils::despawn_screen}}};

// Game status resource
#[derive(Resource, Default)]
pub enum GameStatus {
    #[default]
    AwaitingUserInput,
    Paused,
    GameOver,
}

#[derive(Component)]
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
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridCell {
    pub x: usize,
    pub y: usize,
}

pub fn game_plugin(app: &mut App) {
    app.init_resource::<GameStatus>()
        .init_resource::<AITimeTaken>()
        .init_resource::<GameStats>()
        .init_resource::<EscapeMenuState>()
        .add_event::<GameEnded>()
        .add_event::<StonePlacement>()
        .add_event::<MovePlayed>()
        .add_event::<UpdateAITimeDisplay>()
        .add_event::<UndoMove>()
        .add_event::<UpdateGameStats>()
        .add_event::<RestartGame>()
        .add_event::<EscapeMenuToggled>()
        .add_event::<ExitToMenu>()
        .add_event::<SettingsChanged>()
        .add_systems(OnEnter(AppState::Game), (setup_game_ui, update_available_placement, initialize_game_stats).chain())
        .add_systems(
            Update,
            (
                handle_player_placement,
                handle_undo_button,
                handle_restart_button,
                handle_escape_key,
                handle_escape_menu_buttons,
                handle_settings_buttons,
                refresh_settings_panel.run_if(on_event::<SettingsChanged>),
                place_stone.run_if(on_event::<StonePlacement>),
                process_next_round.run_if(on_event::<MovePlayed>),
                (
                    handle_undo_move.run_if(on_event::<UndoMove>),
                    restart_game.run_if(on_event::<RestartGame>),
                ).before(update_available_placement),
                update_available_placement,
                update_game_stats_display.run_if(on_event::<UpdateGameStats>),
                manage_escape_menu.run_if(on_event::<EscapeMenuToggled>),
                handle_exit_to_menu.run_if(on_event::<ExitToMenu>),
                enhanced_button_interaction_system,
                update_ai_time_display.run_if(on_event::<UpdateAITimeDisplay>),
            ).run_if(in_state(AppState::Game)),
        )
        .add_systems(OnExit(AppState::Game), despawn_screen::<OnGameScreen>);
}

fn setup_game_ui(mut commands: Commands, game_settings: Res<GameSettings>) {
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

fn initialize_game_stats(
    mut update_stats: EventWriter<UpdateGameStats>,
    mut update_ai_time: EventWriter<UpdateAITimeDisplay>,
) {
    // Trigger initial stats display
    update_stats.write(UpdateGameStats);
    update_ai_time.write(UpdateAITimeDisplay);
}

pub fn update_available_placement(
    mut commands: Commands,
    mut ev_board_update: EventReader<MovePlayed>,
    mut ev_undo_move: EventReader<UndoMove>,
    mut ev_restart_game: EventReader<RestartGame>,
    game_state: Res<GameState>,
    parents: Query<(Entity, &Children, &GridCell), With<GridCell>>,
    mut dots: Query<(&mut BackgroundColor, &mut Visibility), With<PreviewDot>>,
) {
    // Check if there are any events that should trigger an update, or if this is the first run
    let has_move_events = !ev_board_update.is_empty();
    let has_undo_events = !ev_undo_move.is_empty();
    let has_restart_events = !ev_restart_game.is_empty();
    let is_first_run = dots.iter().all(|(_, visibility)| *visibility == Visibility::Hidden);
    
    let should_update = has_move_events || has_undo_events || has_restart_events || is_first_run;
    
    // Consume events
    for _ in ev_board_update.read() {}
    for _ in ev_undo_move.read() {}
    for _ in ev_restart_game.read() {}
    
    if !should_update {
        return;
    }

    let possible_moves = game_state.get_possible_moves();
    info!("Updating stone preview... Current player: {:?}, Possible moves count: {}", game_state.current_player, possible_moves.len());
    for (entity, children, cell) in parents.iter() {
        if possible_moves.contains(&(cell.x, cell.y)) {
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
    board_query: Query<Entity, With<BoardRoot>>,
    mut game_state: ResMut<GameState>,
    mut ev_stone_placement: EventReader<StonePlacement>,
    mut move_played: EventWriter<MovePlayed>,
    mut game_stats: ResMut<GameStats>,
    mut update_stats: EventWriter<UpdateGameStats>,
    stones: Query<(Entity, &GridCell, &Stone)>,
) {
    for ev in ev_stone_placement.read() {
        info!("Stone placed at x: {}, y: {}", ev.x, ev.y);
        
        // Track move in history
        game_stats.move_history.push((ev.x, ev.y));
        game_stats.total_moves += 1;
        
        game_state.make_move((ev.x, ev.y));

        let player = game_state.current_player;
        let color = match player {
            Player::Min => Color::BLACK,
            Player::Max => Color::WHITE,
        };

        // Update capture stats
        game_stats.player_captures = game_state.max_captures;
        game_stats.ai_captures = game_state.min_captures;

        // Despawn captured stones
        for (stone_entity, stone_cell, _) in stones.iter() {
            if game_state.board.is_empty_position(stone_cell.x, stone_cell.y) {
                info!("Despawning captured stone at x: {}, y: {}", stone_cell.x, stone_cell.y);
                commands.entity(stone_entity).despawn();
            }
        }

        // Spawn new stone
        if let Ok(board_entity) = board_query.single() {
            commands.entity(board_entity).with_children(|builder| {
                builder.spawn((
                    BoardUtils::stone_node(ev.x, ev.y, BoardUtils::STONE_SIZE),
                    BackgroundColor(color),
                    Stone(player),
                    BorderRadius::all(Val::Percent(50.0)),
                    ZIndex(20),
                    OnGameScreen,
                    GridCell { x: ev.x, y: ev.y },
                ));
            });
        }
        
        // Trigger stats update
        update_stats.write(UpdateGameStats);
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

pub fn handle_undo_button(
    mut undo_events: EventWriter<UndoMove>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<UndoButton>)>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            undo_events.write(UndoMove);
            info!("Undo button pressed");
        }
    }
}

pub fn handle_restart_button(
    mut restart_events: EventWriter<RestartGame>,
    restart_interaction_query: Query<&Interaction, (Changed<Interaction>, With<RestartButton>)>,
    game_over_restart_query: Query<&Interaction, (Changed<Interaction>, With<GameOverRestartButton>)>,
) {
    // Handle settings panel restart button
    for interaction in &restart_interaction_query {
        if *interaction == Interaction::Pressed {
            restart_events.write(RestartGame);
            info!("Settings restart button pressed");
        }
    }
    
    // Handle game over restart button
    for interaction in &game_over_restart_query {
        if *interaction == Interaction::Pressed {
            restart_events.write(RestartGame);
            info!("Game over restart button pressed");
        }
    }
}

pub fn handle_escape_key(
    mut escape_menu_events: EventWriter<EscapeMenuToggled>,
    mut escape_menu_state: ResMut<EscapeMenuState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    game_status: Res<GameStatus>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        // Only allow escape menu if not in game over state
        if !matches!(*game_status, GameStatus::GameOver) {
            escape_menu_state.is_open = !escape_menu_state.is_open;
            escape_menu_state.showing_info = false; // Close info if open
            escape_menu_events.write(EscapeMenuToggled);
            
            info!("Escape menu toggled: {}", if escape_menu_state.is_open { "Open" } else { "Closed" });
        }
    }
}

pub fn manage_escape_menu(
    mut commands: Commands,
    mut escape_menu_events: EventReader<EscapeMenuToggled>,
    mut game_status: ResMut<GameStatus>,
    escape_menu_state: Res<EscapeMenuState>,
    overlay_query: Query<Entity, With<EscapeMenuOverlay>>,
) {
    for _ in escape_menu_events.read() {
        if escape_menu_state.is_open {
            // Pause the game when escape menu opens
            *game_status = GameStatus::Paused;
            
            // Create escape menu overlay
            spawn_escape_menu(&mut commands);
            info!("Escape menu opened - Game paused");
        } else {
            // Resume the game when escape menu closes
            *game_status = GameStatus::AwaitingUserInput;
            
            // Remove escape menu overlay
            for entity in overlay_query.iter() {
                commands.entity(entity).despawn();
            }
            info!("Escape menu closed - Game resumed");
        }
    }
}

pub fn handle_escape_menu_buttons(
    interaction_query: Query<
        (&Interaction, Option<&ResumeButton>, Option<&InfoButton>, Option<&ExitButton>, Option<&RestartButton>),
        (Changed<Interaction>, With<Button>)
    >,
    mut escape_menu_events: EventWriter<EscapeMenuToggled>,
    mut restart_events: EventWriter<RestartGame>,
    mut exit_events: EventWriter<ExitToMenu>,
    mut escape_menu_state: ResMut<EscapeMenuState>,
    mut commands: Commands,
    info_panel_query: Query<Entity, With<InfoPanel>>,
) {
    for (interaction, resume_btn, info_btn, exit_btn, restart_btn) in &interaction_query {
        if *interaction == Interaction::Pressed {
            if resume_btn.is_some() {
                // Resume game
                escape_menu_state.is_open = false;
                escape_menu_events.write(EscapeMenuToggled);
                info!("Resume button pressed");
            } else if restart_btn.is_some() {
                // Restart game
                restart_events.write(RestartGame);
                escape_menu_state.is_open = false;
                escape_menu_events.write(EscapeMenuToggled);
                info!("Restart button pressed from escape menu");
            } else if info_btn.is_some() {
                // Toggle info panel
                if escape_menu_state.showing_info {
                    // Close info panel
                    for entity in info_panel_query.iter() {
                        commands.entity(entity).despawn();
                    }
                    escape_menu_state.showing_info = false;
                } else {
                    // Show info panel
                    spawn_info_panel(&mut commands);
                    escape_menu_state.showing_info = true;
                }
                info!("Info button pressed");
            } else if exit_btn.is_some() {
                // Exit to menu
                exit_events.write(ExitToMenu);
                info!("Exit button pressed");
            }
        }
    }
}

pub fn handle_exit_to_menu(
    mut exit_events: EventReader<ExitToMenu>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    for _ in exit_events.read() {
        info!("Exiting to main menu");
        app_state.set(AppState::Menu);
    }
}

pub fn enhanced_button_interaction_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&EscapeMenuButton>, Option<&ResumeButton>, Option<&RestartButton>, Option<&InfoButton>, Option<&ExitButton>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut background_color, is_escape_menu_btn, resume_btn, restart_btn, info_btn, exit_btn) in &mut interaction_query {
        if is_escape_menu_btn.is_some() {
            // Determine the original color based on button type
            let original_color = if resume_btn.is_some() {
                Color::srgb(0.2, 0.6, 0.2) // Green for Resume
            } else if restart_btn.is_some() {
                Color::srgb(0.6, 0.4, 0.2) // Orange for Restart
            } else if info_btn.is_some() {
                Color::srgb(0.2, 0.4, 0.6) // Blue for Info
            } else if exit_btn.is_some() {
                Color::srgb(0.6, 0.2, 0.2) // Red for Exit
            } else {
                Color::srgb(0.3, 0.3, 0.3) // Default gray
            };

            // Apply interaction effects
            match *interaction {
                Interaction::Pressed => {
                    // Darken the original color for pressed state
                    let [r, g, b, a] = original_color.to_srgba().to_f32_array();
                    *background_color = BackgroundColor(Color::srgba(r * 0.7, g * 0.7, b * 0.7, a));
                },
                Interaction::Hovered => {
                    // Brighten the original color for hover state
                    let [r, g, b, a] = original_color.to_srgba().to_f32_array();
                    *background_color = BackgroundColor(Color::srgba((r * 1.2).min(1.0), (g * 1.2).min(1.0), (b * 1.2).min(1.0), a));
                },
                Interaction::None => {
                    // Restore to original color
                    *background_color = BackgroundColor(original_color);
                }
            };
        }
    }
}

pub fn process_next_round(
    mut commands: Commands,
    mut move_played: EventReader<MovePlayed>,
    mut stone_placement: EventWriter<StonePlacement>,
    mut game_event: EventWriter<GameEnded>,
    settings: Res<GameSettings>,
    mut game_state: ResMut<GameState>,
    mut game_status: ResMut<GameStatus>,
    mut ai_time: ResMut<AITimeTaken>,
    mut update_ai_time: EventWriter<UpdateAITimeDisplay>,
    mut tt: ResMut<TranspositionTable>,
    shared_tt: Res<SharedTranspositionTable>,
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
            
            // Spawn restart button when game ends
            spawn_restart_button(&mut commands);
            
            return;
        }

        // Handle next player's turn
        if game_state.current_player == Player::Max || (game_state.current_player == Player::Min && !settings.versus_ai) {
            info!("Awaiting user click");
            *game_status = GameStatus::AwaitingUserInput;
        } else if settings.versus_ai {
            // AI's turn
            let start_time = Instant::now();
            
            // Only call find_best_move if game is not terminal
            if !game_state.is_terminal() {
                let placement = if let Some(time_limit_ms) = settings.time_limit {
                    // Use time-based iterative deepening
                    let time_limit = Duration::from_millis(time_limit_ms as u64);
                    info!("AI using time-based search with {}ms limit", time_limit_ms);
                    
                    // Use enhanced parallel search for deeper depths (8+), parallel for medium (5-7), standard for shallow (1-4)
                    if settings.ai_depth >= 8 {
                        info!("Using enhanced parallel search for depth {}", settings.ai_depth);
                        find_best_move_parallel_enhanced(&mut game_state, settings.ai_depth, Some(time_limit), &shared_tt)
                    } else if settings.ai_depth > 4 {
                        find_best_move_parallel(&mut game_state, settings.ai_depth, Some(time_limit), &shared_tt)
                    } else {
                        find_best_move(&mut game_state, settings.ai_depth, Some(time_limit), &mut tt)
                    }
                } else {
                    // Use depth-based iterative deepening
                    info!("AI using depth-based search to depth {}", settings.ai_depth);
                    
                    // Use enhanced parallel search for deeper depths (8+), parallel for medium (5-7), standard for shallow (1-4)
                    if settings.ai_depth >= 8 {
                        info!("Using enhanced parallel search for depth {}", settings.ai_depth);
                        find_best_move_parallel_enhanced(&mut game_state, settings.ai_depth, None, &shared_tt)
                    } else if settings.ai_depth > 4 {
                        find_best_move_parallel(&mut game_state, settings.ai_depth, None, &shared_tt)
                    } else {
                        find_best_move(&mut game_state, settings.ai_depth, None, &mut tt)
                    }
                };
                let elapsed_time = start_time.elapsed().as_millis();
                
                // Update AI timing statistics
                ai_time.millis = elapsed_time;
                ai_time.total_time += elapsed_time;
                ai_time.move_count += 1;
                ai_time.average_time = ai_time.total_time as f64 / ai_time.move_count as f64;
                
                update_ai_time.write(UpdateAITimeDisplay);

                if let Some((x, y)) = placement {
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
    mut query: Query<&mut Text, (With<AITimeText>, Without<AIAverageTimeText>)>,
    mut avg_query: Query<&mut Text, (With<AIAverageTimeText>, Without<AITimeText>)>,
    ai_time: Res<AITimeTaken>,
    mut events: EventReader<UpdateAITimeDisplay>,
) {
    for _ in events.read() {
        info!("Updating AI time display: {:.0}ms, Average: {:.1}ms", ai_time.millis, ai_time.average_time);
        
        // Update current AI time
        for mut text in query.iter_mut() {
			text.0 = format!("{:.0}ms", ai_time.millis);
        }
        
        // Update average AI time
        for mut text in avg_query.iter_mut() {
            text.0 = format!("{:.1}ms", ai_time.average_time);
        }
    }
}

pub fn handle_undo_move(
    mut commands: Commands,
    mut undo_events: EventReader<UndoMove>,
    mut game_state: ResMut<GameState>,
    mut game_stats: ResMut<GameStats>,
    mut update_stats: EventWriter<UpdateGameStats>,
    stones: Query<(Entity, &GridCell, &Stone)>,
    game_status: Res<GameStatus>,
    board_query: Query<Entity, With<BoardRoot>>,
) {
    for _ in undo_events.read() {
        // Only allow undo during user input phase or when paused
        if !matches!(*game_status, GameStatus::AwaitingUserInput | GameStatus::Paused) {
            return;
        }

        // Check if there are moves to undo
        if game_stats.move_history.is_empty() {
            info!("No moves to undo");
            return;
        }

        if let Some(last_move) = game_stats.move_history.pop() {
            info!("Undoing move at ({}, {})", last_move.0, last_move.1);
            
            // Store board state before undo to know which stones to restore
            let mut stones_to_restore = Vec::new();
            
            // Check which positions will be restored after undo
            if let Some(last_captures) = game_state.capture_history.last() {
                for &(x, y) in last_captures {
                    stones_to_restore.push((x, y));
                }
            }
            
            // Update game stats
            game_stats.total_moves = game_stats.total_moves.saturating_sub(1);
            
            // Undo the move in the game state
            game_state.undo_move(last_move);
            
            // Debug: Print current player after undo
            info!("After undo: current player = {:?}, total moves = {}", game_state.current_player, game_stats.total_moves);
            
            // Update capture counts after undo
            game_stats.player_captures = game_state.max_captures;
            game_stats.ai_captures = game_state.min_captures;
            
            // Remove the stone entity from UI
            for (stone_entity, stone_cell, _) in stones.iter() {
                if stone_cell.x == last_move.0 && stone_cell.y == last_move.1 {
                    commands.entity(stone_entity).despawn();
                    break;
                }
            }
            
            // Restore captured stones in UI
            if let Ok(board_entity) = board_query.single() {
                for (x, y) in stones_to_restore {
                    // Get the player who owns this restored stone
                    if let Some(player) = game_state.board.get_player(x, y) {
                        let color = match player {
                            Player::Min => Color::BLACK,
                            Player::Max => Color::WHITE,
                        };
                        
                        // Spawn the restored stone
                        commands.entity(board_entity).with_children(|builder| {
                            builder.spawn((
                                BoardUtils::stone_node(x, y, BoardUtils::STONE_SIZE),
                                BackgroundColor(color),
                                Stone(player),
                                BorderRadius::all(Val::Percent(50.0)),
                                ZIndex(20),
                                OnGameScreen,
                                GridCell { x, y },
                            ));
                        });
                    }
                }
            }
            
            // Trigger stats update
            update_stats.write(UpdateGameStats);
            
            info!("Move undone successfully. Total moves: {}", game_stats.total_moves);
        }
    }
}

pub fn update_game_stats_display(
    mut events: EventReader<UpdateGameStats>,
    game_stats: Res<GameStats>,
    mut total_moves_query: Query<&mut Text, (With<TotalMovesText>, Without<PlayerCapturesText>, Without<AICapturesText>)>,
    mut player_captures_query: Query<&mut Text, (With<PlayerCapturesText>, Without<TotalMovesText>, Without<AICapturesText>)>,
    mut ai_captures_query: Query<&mut Text, (With<AICapturesText>, Without<TotalMovesText>, Without<PlayerCapturesText>)>,
) {
    for _ in events.read() {
        // Update total moves
        for mut text in total_moves_query.iter_mut() {
            text.0 = game_stats.total_moves.to_string();
        }
        
        // Update player captures
        for mut text in player_captures_query.iter_mut() {
            text.0 = game_stats.player_captures.to_string();
        }
        
        // Update AI captures
        for mut text in ai_captures_query.iter_mut() {
            text.0 = game_stats.ai_captures.to_string();
        }
        
        info!("Updated game stats - Moves: {}, Player captures: {}, AI captures: {}", 
              game_stats.total_moves, game_stats.player_captures, game_stats.ai_captures);
    }
}

pub fn restart_game(
    mut restart_events: EventReader<RestartGame>,
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut game_status: ResMut<GameStatus>,
    mut game_stats: ResMut<GameStats>,
    mut ai_time: ResMut<AITimeTaken>,
    mut update_stats: EventWriter<UpdateGameStats>,
    mut update_ai_time: EventWriter<UpdateAITimeDisplay>,
    settings: Res<GameSettings>,
    stones: Query<Entity, With<Stone>>,
    game_over_restart_buttons: Query<Entity, With<GameOverRestartButton>>,
) {
    for _ in restart_events.read() {
        info!("Restarting game...");
        
        // Reset game state
        *game_state = GameState::new(settings.board_size, settings.minimum_chain_to_win);
        *game_status = GameStatus::AwaitingUserInput;
        
        // Reset game statistics
        game_stats.total_moves = 0;
        game_stats.player_captures = 0;
        game_stats.ai_captures = 0;
        game_stats.move_history.clear();
        
        // Reset AI timing statistics
        ai_time.millis = 0;
        ai_time.total_time = 0;
        ai_time.move_count = 0;
        ai_time.average_time = 0.0;
        
        // Clean up existing stones
        for stone_entity in stones.iter() {
            commands.entity(stone_entity).despawn();
        }
        
        // Remove only the game over restart buttons (keep the settings panel ones)
        for button_entity in game_over_restart_buttons.iter() {
            commands.entity(button_entity).despawn();
        }
        
        // Trigger updates
        update_stats.write(UpdateGameStats);
        update_ai_time.write(UpdateAITimeDisplay);
        
        info!("Game restarted successfully");
    }
}

fn spawn_restart_button(commands: &mut Commands) {
    commands.spawn((
        Button,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(50.0),
            left: Val::Percent(50.0),
            width: Val::Px(200.0),
            height: Val::Px(60.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            margin: UiRect::left(Val::Px(-100.0)), // Center the button
            ..default()
        },
        BackgroundColor(Color::srgb(0.2, 0.7, 0.2)),
        BorderRadius::all(Val::Px(8.0)),
        ZIndex(200),
        GameOverRestartButton,
        OnGameScreen,
    )).with_children(|builder| {
        builder.spawn((
            Text::new("Restart Game"),
            TextFont { font_size: 20.0, ..default() },
            TextColor(Color::WHITE),
        ));
    });
}

#[derive(Component)]
pub struct AITimeText;

#[derive(Component)]
pub struct AIAverageTimeText;

// Resource to store AI computation time
#[derive(Resource, Default)]
pub struct AITimeTaken {
    pub millis: u128,
    pub total_time: u128,
    pub move_count: usize,
    pub average_time: f64,
}

// Event to trigger AI time display update
#[derive(Event)]
pub struct UpdateAITimeDisplay;

// Event to trigger undo move
#[derive(Event)]
pub struct UndoMove;

// Event to update game stats display
#[derive(Event)]
pub struct UpdateGameStats;

// Event to restart the game
#[derive(Event)]
pub struct RestartGame;

// Event to toggle escape menu
#[derive(Event)]
pub struct EscapeMenuToggled;

// Event to exit to menu
#[derive(Event)]
pub struct ExitToMenu;

// Resource to track game statistics
#[derive(Resource, Default)]
pub struct GameStats {
    pub total_moves: usize,
    pub player_captures: usize,
    pub ai_captures: usize,
    pub move_history: Vec<(usize, usize)>, // Track moves for undo
}

// Resource to track escape menu state
#[derive(Resource, Default)]
pub struct EscapeMenuState {
    pub is_open: bool,
    pub showing_info: bool,
}

// Components for UI elements
#[derive(Component)]
pub struct UndoButton;

#[derive(Component)]
pub struct PlayerCapturesText;

#[derive(Component)]
pub struct AICapturesText;

#[derive(Component)]
pub struct TotalMovesText;

#[derive(Component)]
pub struct RestartButton;

#[derive(Component)]
pub struct GameOverRestartButton;

// Escape menu components
#[derive(Component)]
pub struct EscapeMenuOverlay;

#[derive(Component)]
pub struct ResumeButton;

#[derive(Component)]
pub struct ExitButton;

#[derive(Component)]
pub struct InfoButton;

#[derive(Component)]
pub struct InfoPanel;

#[derive(Component)]
pub struct EscapeMenuButton;

fn spawn_escape_menu(commands: &mut Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)), // Semi-transparent dark overlay
        ZIndex(150),
        EscapeMenuOverlay,
        OnGameScreen,
    )).with_children(|builder| {
        // Main menu panel
        builder.spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                width: Val::Px(400.0),
                min_height: Val::Px(500.0),
                padding: UiRect::all(Val::Px(30.0)),
                row_gap: Val::Px(20.0),
                border: UiRect::all(Val::Px(3.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            BorderColor(Color::srgb(0.4, 0.4, 0.4)),
            BorderRadius::all(Val::Px(15.0)),
        )).with_children(|builder| {
            // Title
            builder.spawn((
                Text::new("GAME PAUSED"),
                TextFont { font_size: 32.0, ..default() },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Resume Game Button
            builder
                .spawn((
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::vertical(Val::Px(5.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.6, 0.2)),
                    BorderRadius::all(Val::Px(8.0)),
                    ResumeButton,
                    EscapeMenuButton,
                ))
                .with_children(|builder| {
                    builder.spawn((
                        Text::new("Resume Game"),
                        TextFont { font_size: 18.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });
            
            // Restart Game Button  
            builder
                .spawn((
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::vertical(Val::Px(5.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.6, 0.4, 0.2)),
                    BorderRadius::all(Val::Px(8.0)),
                    RestartButton,
                    EscapeMenuButton,
                ))
                .with_children(|builder| {
                    builder.spawn((
                        Text::new("Restart Game"),
                        TextFont { font_size: 18.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });
            
            // How to Play Button
            builder
                .spawn((
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::vertical(Val::Px(5.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.4, 0.6)),
                    BorderRadius::all(Val::Px(8.0)),
                    InfoButton,
                    EscapeMenuButton,
                ))
                .with_children(|builder| {
                    builder.spawn((
                        Text::new("How to Play"),
                        TextFont { font_size: 18.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });
            
            // Exit to Menu Button
            builder
                .spawn((
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::vertical(Val::Px(5.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.6, 0.2, 0.2)),
                    BorderRadius::all(Val::Px(8.0)),
                    ExitButton,
                    EscapeMenuButton,
                ))
                .with_children(|builder| {
                    builder.spawn((
                        Text::new("Exit to Menu"),
                        TextFont { font_size: 18.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });
            
            // Controls info
            builder.spawn((
                Text::new("Press ESC to close this menu"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
            ));
            });
        });
}

fn handle_settings_buttons(
    mut game_settings: ResMut<GameSettings>,
    mut button_query: Query<
        (&Interaction, Option<&SettingsBoardSizeButton>, Option<&SettingsChainToWinButton>, 
         Option<&SettingsCaptureToWinButton>, Option<&SettingsAIDepthButton>, 
         Option<&SettingsTimeLimitButton>, Option<&SettingsTimeLimitToggleButton>),
        (Changed<Interaction>, With<Button>)
    >,
    mut settings_changed: EventWriter<SettingsChanged>,
) {
    for (interaction, board_size, chain_to_win, capture_to_win, ai_depth, time_limit, time_limit_toggle) in button_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            let mut settings_updated = false;

            // Board Size Button
            if let Some(button) = board_size {
                if button.increment {
                    if game_settings.board_size < 25 {
                        game_settings.board_size += 1;
                        settings_updated = true;
                    }
                } else {
                    if game_settings.board_size > 3 {
                        game_settings.board_size -= 1;
                        settings_updated = true;
                    }
                }
            }

            // Chain to Win Button
            if let Some(button) = chain_to_win {
                if button.increment {
                    if game_settings.minimum_chain_to_win < 8 {
                        game_settings.minimum_chain_to_win += 1;
                        settings_updated = true;
                    }
                } else {
                    if game_settings.minimum_chain_to_win > 3 {
                        game_settings.minimum_chain_to_win -= 1;
                        settings_updated = true;
                    }
                }
            }

            // Capture to Win Button
            if let Some(button) = capture_to_win {
                if button.increment {
                    if game_settings.total_capture_to_win < 20 {
                        game_settings.total_capture_to_win += 1;
                        settings_updated = true;
                    }
                } else {
                    if game_settings.total_capture_to_win > 1 {
                        game_settings.total_capture_to_win -= 1;
                        settings_updated = true;
                    }
                }
            }

            // AI Depth Button
            if let Some(button) = ai_depth {
                if button.increment {
                    if game_settings.ai_depth < 6 {
                        game_settings.ai_depth += 1;
                        settings_updated = true;
                    }
                } else {
                    if game_settings.ai_depth > 1 {
                        game_settings.ai_depth -= 1;
                        settings_updated = true;
                    }
                }
            }

            // Time Limit Button
            if let Some(button) = time_limit {
                if let Some(current_limit) = game_settings.time_limit {
                    if button.increment {
                        game_settings.time_limit = Some(current_limit + 1000); // Increase by 1 second
                        settings_updated = true;
                    } else if current_limit > 1000 {
                        game_settings.time_limit = Some(current_limit - 1000); // Decrease by 1 second
                        settings_updated = true;
                    }
                }
            }

            // Time Limit Toggle Button
            if time_limit_toggle.is_some() {
                if game_settings.time_limit.is_some() {
                    game_settings.time_limit = None; // Disable time limit
                } else {
                    game_settings.time_limit = Some(5000); // Enable with 5 seconds default
                }
                settings_updated = true;
            }

            if settings_updated {
                settings_changed.write(SettingsChanged);
            }
        }
    }
}

fn refresh_settings_panel(
    game_settings: Res<GameSettings>,
    mut board_size_query: Query<&mut Text, (With<BoardSizeText>, Without<ChainToWinText>, Without<CaptureToWinText>, Without<AIDepthText>, Without<TimeLimitText>)>,
    mut chain_to_win_query: Query<&mut Text, (With<ChainToWinText>, Without<BoardSizeText>, Without<CaptureToWinText>, Without<AIDepthText>, Without<TimeLimitText>)>,
    mut capture_to_win_query: Query<&mut Text, (With<CaptureToWinText>, Without<BoardSizeText>, Without<ChainToWinText>, Without<AIDepthText>, Without<TimeLimitText>)>,
    mut ai_depth_query: Query<&mut Text, (With<AIDepthText>, Without<BoardSizeText>, Without<ChainToWinText>, Without<CaptureToWinText>, Without<TimeLimitText>)>,
    mut time_limit_query: Query<&mut Text, (With<TimeLimitText>, Without<BoardSizeText>, Without<ChainToWinText>, Without<CaptureToWinText>, Without<AIDepthText>)>,
) {
    info!("Refreshing settings UI display");
    
    // Update Board Size
    for mut text in board_size_query.iter_mut() {
        text.0 = format!("{}x{}", game_settings.board_size, game_settings.board_size);
    }
    
    // Update Chain to Win
    for mut text in chain_to_win_query.iter_mut() {
        text.0 = game_settings.minimum_chain_to_win.to_string();
    }
    
    // Update Capture to Win
    for mut text in capture_to_win_query.iter_mut() {
        text.0 = game_settings.total_capture_to_win.to_string();
    }
    
    // Update AI Depth (only if in AI mode)
    if game_settings.versus_ai {
        for mut text in ai_depth_query.iter_mut() {
            text.0 = game_settings.ai_depth.to_string();
        }
        
        // Update Time Limit
        for mut text in time_limit_query.iter_mut() {
            if let Some(time_limit) = game_settings.time_limit {
                text.0 = time_limit.to_string();
            } else {
                text.0 = "Click to enable".to_string();
            }
        }
    }
    
    info!("Settings UI updated - Board: {}x{}, Chain: {}, Captures: {}, AI Depth: {}, VS AI: {}", 
          game_settings.board_size, game_settings.board_size, 
          game_settings.minimum_chain_to_win, game_settings.total_capture_to_win,
          game_settings.ai_depth, game_settings.versus_ai);
}fn spawn_info_panel(commands: &mut Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        ZIndex(160),
        InfoPanel,
        OnGameScreen,
    )).with_children(|builder| {
        builder.spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                width: Val::Px(600.0),
                max_height: Val::Percent(80.0),
                padding: UiRect::all(Val::Px(30.0)),
                row_gap: Val::Px(15.0),
                border: UiRect::all(Val::Px(2.0)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(Color::srgb(0.05, 0.05, 0.05)),
            BorderColor(Color::srgb(0.5, 0.5, 0.5)),
            BorderRadius::all(Val::Px(10.0)),
        )).with_children(|builder| {
            // Title
            builder.spawn((
                Text::new("How to Play Gomoku"),
                TextFont { font_size: 28.0, ..default() },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(15.0)),
                    ..default()
                },
            ));

            // Objective section
            builder.spawn((
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(5.0),
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            )).with_children(|builder| {
                builder.spawn((
                    Text::new("🎯 Objective"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.8, 0.8, 1.0)),
                ));
                builder.spawn((
                    Text::new("Be the first to get 5 stones in a row (horizontally, vertically, or diagonally)"),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    Node {
                        margin: UiRect::left(Val::Px(10.0)),
                        ..default()
                    },
                ));
            });

            // Players section
            builder.spawn((
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(5.0),
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            )).with_children(|builder| {
                builder.spawn((
                    Text::new("⚫ Players"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.8, 0.8, 1.0)),
                ));
                builder.spawn((
                    Text::new("You play as WHITE stones, AI plays as BLACK stones"),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    Node {
                        margin: UiRect::left(Val::Px(10.0)),
                        ..default()
                    },
                ));
            });

            // How to Play section
            builder.spawn((
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(5.0),
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            )).with_children(|builder| {
                builder.spawn((
                    Text::new("🎮 How to Play"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.8, 0.8, 1.0)),
                ));
                builder.spawn((
                    Text::new("• Click on any white dot to place your stone\n• Stones must be placed in valid positions\n• You can capture opponent stones by surrounding them"),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    Node {
                        margin: UiRect::left(Val::Px(10.0)),
                        ..default()
                    },
                ));
            });

            // Controls section
            builder.spawn((
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(5.0),
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            )).with_children(|builder| {
                builder.spawn((
                    Text::new("⌨️ Controls"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.8, 0.8, 1.0)),
                ));
                builder.spawn((
                    Text::new("• ESC: Open/close menu\n• Left Click: Place stone"),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    Node {
                        margin: UiRect::left(Val::Px(10.0)),
                        ..default()
                    },
                ));
            });

            // Close button
            builder.spawn((
                Button,
                Node {
                    width: Val::Px(120.0),
                    height: Val::Px(40.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(20.0)),
                    align_self: AlignSelf::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.3, 0.3, 0.7)),
                BorderRadius::all(Val::Px(5.0)),
                InfoButton, // Reuse the same button component to toggle
            )).with_children(|builder| {
                builder.spawn((
                    Text::new("Close"),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::WHITE),
                ));
            });
        });
    });
}