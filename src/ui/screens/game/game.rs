use std::time::Instant;

use bevy::prelude::*;
use crate::{core::{board::Player, state::GameState}, interface::utils::find_best_move, ui::{app::{AppState, GameSettings}, screens::{game::{self, board::{BoardRoot, BoardUtils, PreviewDot}, settings::spawn_settings_panel}, utils::despawn_screen}}};

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
    game_state: Res<GameState>,
    parents: Query<(Entity, &Children, &GridCell), With<GridCell>>,
    mut dots: Query<(&mut BackgroundColor, &mut Visibility), With<PreviewDot>>,
) {
    // Consume events
    for _ in ev_board_update.read() {}

    let possible_moves = game_state.get_possible_moves();
	println!("{:?}", possible_moves);
    info!("Updating stone preview... {} cells", parents.iter().len());
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

		println!("Move played !");

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
                println!("Writing stone placement");
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
        if game_state.is_terminal() {
            let winner = game_state.check_winner();
            game_event.write(GameEnded { winner });
            *game_status = GameStatus::GameOver;
            if winner.is_some() {
                println!("End game ! Winner is {:?}", winner);
            } else {
                println!("An error occurred, game is terminal but no winner is found, the game is a draw.");
            }
            return;
        }
        if game_state.current_player == Player::Max {
            info!("Awaiting user click");
            *game_status = GameStatus::AwaitingUserInput;
        } else {
            let start_time = Instant::now();
            let placement = find_best_move(&mut game_state, settings.ai_depth);
            let elapsed_time = start_time.elapsed().as_millis();
            ai_time.millis = elapsed_time;
            info!("AI took {:.0}ms to compute move", elapsed_time);
            update_ai_time.write(UpdateAITimeDisplay);

            if let Some((x, y)) = placement {
                stone_placement.write(StonePlacement { x, y });
                *game_status = GameStatus::AwaitingUserInput;
            } else {
                game_event.write(GameEnded { winner: Some(Player::Max) });
                println!("Error occurred while finding best move");
                *game_status = GameStatus::GameOver;
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
			text.0 = format!("AI Time: {:.0}ms", ai_time.millis);
			print!("{}", format!("AI Time: {:.0}ms", ai_time.millis));
        }
    }
}

#[derive(Component)]
pub struct AITimeText;

// Resource to store AI computation time
#[derive(Resource, Default)]
pub struct AITimeTaken {
    pub millis: u128,
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