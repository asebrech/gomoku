use bevy::
    prelude::*
;
use gomoku::{core::{board::Player, state::GameState}, interface::utils::find_best_move};

use crate::ui::{app::{AppState, GameSettings}, screens::utils::despawn_screen};

// This plugin will contain the game, displaying a Gomoku board with variable size
pub fn game_plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Game), game_setup)
        //.add_systems(Update, game.run_if(in_state(AppState::Game)))
        .add_systems(OnExit(AppState::Game), despawn_screen::<OnGameScreen>);
}

// Tag component used to tag entities added on the game screen
#[derive(Component)]
struct OnGameScreen;

const CELL_SIZE: f32 = 32.0;
const LINE_THICKNESS: f32 = 2.0;

#[derive(Component)]
pub struct StonePreview;

#[derive(Component)]
pub struct PlacedStone;

#[derive(Component)]
pub  struct AvailableArea;

#[derive(Component)]
pub struct BoardRoot;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridCell {
    x: usize,
    y: usize,
}



fn game_setup(
    mut commands: Commands,
    game_settings: Res<GameSettings>
) {
    // Board background
    commands
        .spawn((
            Node {
                    display: Display::Grid,
                    width: Val::Px((game_settings.board_size as f32) * CELL_SIZE),
                    height: Val::Px((game_settings.board_size as f32) * CELL_SIZE),
                    align_self: AlignSelf::Center,
                    justify_self: JustifySelf::Center,
                    position_type: PositionType::Relative,
                    ..default()
            },
            BackgroundColor(Color::srgb(0.95, 0.85, 0.7)), // Go board wood color
            OnGameScreen,
			BoardRoot,
        ))
        .with_children(|builder| {
            // Vertical lines
            for i in 0..game_settings.board_size {
                builder.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                            left: Val::Px(i as f32 * CELL_SIZE + CELL_SIZE / 2.0 - LINE_THICKNESS / 2.0),
                            top: Val::Px(0.0),
                            width: Val::Px(LINE_THICKNESS),
                            height: Val::Px(CELL_SIZE * (game_settings.board_size as f32)),
                            ..default()
                    },
                    BackgroundColor(Color::BLACK),
                ));
            }
 
            // Horizontal lines
            for i in 0..game_settings.board_size {
                builder.spawn((
                    Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            top: Val::Px(i as f32 * CELL_SIZE + CELL_SIZE / 2.0 - LINE_THICKNESS / 2.0),
                            width: Val::Px(CELL_SIZE * (game_settings.board_size as f32)),
                            height: Val::Px(LINE_THICKNESS),
                            ..default()
                    },
                    BackgroundColor(Color::BLACK),
                ));
            }

			// Add hoverable intersections
			for y in 0..game_settings.board_size {
				for x in 0..game_settings.board_size {
					builder.spawn((
						Node {
							position_type: PositionType::Absolute,
							left: Val::Px(x as f32 * CELL_SIZE),
							top: Val::Px(y as f32 * CELL_SIZE),
							width: Val::Px(CELL_SIZE),
							height: Val::Px(CELL_SIZE),
							..default()
						},
						ZIndex(10),
						Interaction::default(),
						BackgroundColor(Color::NONE),
						GridCell { x, y }
					));
				}
			}

			// Spawn the preview stone
			builder.spawn((
				Node {
						position_type: PositionType::Absolute,
						width: Val::Px(16.0),
						height: Val::Px(16.0),
						..default()
				},
				ZIndex(20),
				BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)), // semi-transparent black
				Visibility::Hidden,
				StonePreview,
				OnGameScreen,
			));

        });
}

pub fn update_preview_stone(
    mut preview_query: Query<(&mut Node, &mut Visibility), With<StonePreview>>,
    interaction_query: Query<(&Interaction, &GridCell)>,
    game_state: Res<GameState>,
) {
    for (mut node, mut visibility) in preview_query.iter_mut() {
        let mut found_hover = false;
        for (interaction, grid_cell) in interaction_query.iter() {
			if game_state.board.get_player(grid_cell.x, grid_cell.y).is_none() && game_state.get_possible_moves().contains(&(grid_cell.x, grid_cell.y)) {
				node.left = Val::Px(grid_cell.x as f32 * CELL_SIZE + (CELL_SIZE - 16.0) / 2.0);
				node.top = Val::Px(grid_cell.y as f32 * CELL_SIZE + (CELL_SIZE - 16.0) / 2.0);
				*visibility = Visibility::Visible;
			} else {
				*visibility = Visibility::Hidden;
			}
			found_hover = true;
			break;
        }

        if !found_hover {
            *visibility = Visibility::Hidden;
        }
    }
}


pub fn handle_click_to_place(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    interaction_query: Query<(&Interaction, &GridCell)>,
    mut game_state: ResMut<GameState>,
    board_query: Query<Entity, With<BoardRoot>>,
	game_settings: Res<GameSettings>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        for (interaction, cell) in interaction_query.iter() {
            if *interaction == Interaction::Pressed && game_state.board.get_player(cell.x, cell.y).is_none() {
                println!("Placing stone at: {}, {}", cell.x, cell.y);
			println!("board player {:?}", game_state.current_player);
				game_state.make_move((cell.x, cell.y));
				println!("board player {:?}", game_state.current_player);
                let board_entity = match board_query.single() {
                    Ok(entity) => entity,
                    Err(err) => {
                        eprintln!("Board entity not found: {:?}", err);
                        return;
                    }
                };

				//insert into game state here
                commands.entity(board_entity).with_children(|builder| {
                    spawn_piece(builder, cell.x, cell.y,  if game_state.current_player == Player::Min { Color::BLACK } else {Color::WHITE  });
                });

				if game_state.is_terminal() {
					println!("Finished !");
					return;
				}
				let ai_pos = find_best_move(&mut game_state, game_settings.ai_depth);
				if ai_pos.is_some() {
					let pos = ai_pos.unwrap();
					game_state.make_move(pos);
									//insert into game state here
					commands.entity(board_entity).with_children(|builder| {
						spawn_piece(builder, pos.0, pos.1,  if game_state.current_player == Player::Min { Color::BLACK } else {Color::WHITE  });
					});

				} else {
					println!("Couldnt find AI best move");
				}
	
            }
        }
    }
	if buttons.just_pressed(MouseButton::Right) {
		//game_state.undo_move(move_);
	}
}


pub fn spawn_piece(builder: &mut ChildSpawnerCommands, x: usize, y: usize, color: Color) {
    let size = 16.0;
    let offset = (CELL_SIZE - size) / 2.0;

    builder.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(x as f32 * CELL_SIZE + offset),
            top: Val::Px(y as f32 * CELL_SIZE + offset),
            width: Val::Px(size),
            height: Val::Px(size),
            ..default()
        },
        BackgroundColor(color),
        PlacedStone,
        OnGameScreen,
    ));
}


