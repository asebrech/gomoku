use std::time::Instant;

use bevy::prelude::*;
use crate::{core::{board::Player, state::GameState}, interface::utils::find_best_move_timed, ui::{app::{AppState, GameSettings}, screens::{game::{board::{BoardRoot, BoardUtils, PreviewDot}, settings::spawn_settings_panel}, utils::despawn_screen}}};

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
        .add_event::<GameEnded>()
        .add_event::<StonePlacement>()
        .add_event::<MovePlayed>()
        .add_event::<UpdateAITimeDisplay>()
        .add_systems(OnEnter(AppState::Game), (setup_game_ui, update_available_placement).chain())
        .add_systems(
            Update,
            (
                handle_player_placement,
                place_stone.run_if(on_event::<StonePlacement>),
                process_next_round.run_if(on_event::<MovePlayed>),
                update_available_placement.run_if(on_event::<MovePlayed>),
                toggle_pause,
                update_ai_time_display.run_if(on_event::<UpdateAITimeDisplay>),
                update_ai_depth_display.run_if(on_event::<UpdateAITimeDisplay>),
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

pub fn update_available_placement(
    mut commands: Commands,
    mut ev_board_update: EventReader<MovePlayed>,
    mut game_state: ResMut<GameState>,
    parents: Query<(Entity, &Children, &GridCell), With<GridCell>>,
    mut dots: Query<(&mut BackgroundColor, &mut Visibility), With<PreviewDot>>,
) {
    // Consume events
    for _ in ev_board_update.read() {}

    let possible_moves = game_state.get_possible_moves();
    info!("Updating stone preview...");
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
    stones: Query<(Entity, &GridCell, &Stone)>,
) {
    for ev in ev_stone_placement.read() {
        info!("Stone placed at x: {}, y: {}", ev.x, ev.y);
        game_state.make_move((ev.x, ev.y));

        let player = game_state.current_player;
        let color = match player {
            Player::Min => Color::BLACK,
            Player::Max => Color::WHITE,
        };

        // Despawn captured stones
        //let captured_positions = game_state.get_captured_positions().unwrap_or_default();
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
    mut update_ai_time: EventWriter<UpdateAITimeDisplay>,
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
            let start_time = Instant::now();
            
            // Only call AI search if game is not terminal
            if !game_state.is_terminal() {
                // Use the new advanced search with time limit from settings if available
                let (placement, depth_reached) = if let Some(time_limit_seconds) = settings.time_limit {
                    find_best_move_timed(&mut game_state, (time_limit_seconds * 1000) as u64)
                } else {
                    // Use 400ms for competitive play - fast but thorough
                    find_best_move_timed(&mut game_state, 400)
                };
                
                let elapsed_time = start_time.elapsed().as_millis();
                ai_time.millis = elapsed_time;
                ai_time.depth_reached = depth_reached;
                update_ai_time.write(UpdateAITimeDisplay);
                
                info!("AI search completed: depth {}, time {}ms", depth_reached, elapsed_time);

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
    mut query: Query<&mut Text, With<AITimeText>>,
    ai_time: Res<AITimeTaken>,
    mut events: EventReader<UpdateAITimeDisplay>,
) {
    for _ in events.read() {
        info!("Updating AI time display: {:.0}ms", ai_time.millis);
        for mut text in query.iter_mut() {
			text.0 = format!("{:.0}ms", ai_time.millis);
        }
    }
}

pub fn update_ai_depth_display(
    mut query: Query<&mut Text, With<AIDepthText>>,
    ai_time: Res<AITimeTaken>,
    mut events: EventReader<UpdateAITimeDisplay>,
) {
    for _ in events.read() {
        info!("Updating AI depth display: {}", ai_time.depth_reached);
        for mut text in query.iter_mut() {
            text.0 = ai_time.depth_reached.to_string();
        }
    }
}

#[derive(Component)]
pub struct AITimeText;

#[derive(Component)]
pub struct AIDepthText;

// Resource to store AI computation time and depth reached
#[derive(Resource, Default)]
pub struct AITimeTaken {
    pub millis: u128,
    pub depth_reached: i32,
}

// Event to trigger AI time display update
#[derive(Event)]
pub struct UpdateAITimeDisplay;


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