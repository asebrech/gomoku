use bevy::prelude::*;
use crate::{core::{board::Player, state::GameState}, interface::utils::find_best_move, ui::{app::{AppState, GameSettings}, screens::{game::{self, board::{BoardRoot, BoardUtils}}, utils::despawn_screen}}};

#[derive(Component)]
pub struct OnGameScreen;
#[derive(Component)]
pub struct Stone(Player);
#[derive(Component)]
pub struct AvailableArea;

#[derive(Event)]
pub struct StonePlacement {
	x: usize,
	y: usize
}

#[derive(Event)]
pub struct MovePlayed;

#[derive(Event)]
pub struct GameEnded {
	winner: Option<Player>
}

#[derive(Component)]
pub struct PreviewStone;

#[derive(Component)]
pub struct PreviewDot;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridCell {
    pub x: usize,
    pub y: usize,
}

#[derive(Component)]
pub struct GameSettingsPanel;

pub fn game_plugin(app: &mut App) {
    app
		.add_event::<GameEnded>()
		.add_event::<StonePlacement>()
		.add_event::<MovePlayed>()
		.add_systems(OnEnter(AppState::Game), (setup_game_ui, update_preview_stone).chain())
		.add_systems(
            Update,
            handle_player_placement
				.run_if(in_state(AppState::Game))
        )
		.add_systems(
            Update,
            update_preview_stone
                .run_if(in_state(AppState::Game))
				.run_if(on_event::<StonePlacement>),
        )
		.add_systems(
            Update,
            place_stone
                .run_if(in_state(AppState::Game))
				.run_if(on_event::<StonePlacement>),
        )
		.add_systems(
            Update,
            process_next_round
                .run_if(in_state(AppState::Game))
                .run_if(on_event::<MovePlayed>), // Run only on MovePlayed event
        )
        .add_systems(OnExit(AppState::Game), despawn_screen::<OnGameScreen>);
}

fn setup_game_ui(mut commands: Commands, game_settings: Res<GameSettings>) {
    // Main container for the entire game UI
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
            // Board container
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

            // Settings panel
            spawn_settings_panel(builder, &game_settings);
        });
}

fn spawn_settings_panel(builder: &mut ChildSpawnerCommands, game_settings: &GameSettings) {
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

pub fn update_preview_stone(
    mut commands: Commands,
	mut ev_placed_stone: EventReader<StonePlacement>,
    game_state: Res<GameState>,
    parents: Query<(Entity, &Children, &GridCell), With<GridCell>>,
    mut dots: Query<(&mut BackgroundColor, &mut Visibility, Entity), With<PreviewDot>>,
) {
    let possible_moves = game_state.get_possible_moves();
	info!("Updating stone preview...");	
    // Consume events
	for _ in ev_placed_stone.read() {}
	
	for (entity, children, cell) in parents.iter() {
		if possible_moves.contains(&(cell.x, cell.y)) {
			for &child in children {
				if let Ok((mut bg, mut visibility, child_entity)) = dots.get_mut(child) {
					*bg = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.4));
					*visibility = Visibility::Visible;
					commands.entity(child_entity).insert(PreviewStone);
					commands.entity(entity).insert(AvailableArea);
				}
			}
		} else {
			for &child in children {
				if let Ok((mut bg, mut visibility, child_entity)) = dots.get_mut(child) {
					*bg = BackgroundColor(Color::NONE);
					*visibility = Visibility::Hidden;
					commands.entity(child_entity).remove::<PreviewStone>();
					commands.entity(entity).remove::<AvailableArea>();
				}
			}
		}
	}
}

pub fn process_next_round(mut move_played: EventReader<MovePlayed>, mut stone_placed: EventWriter<StonePlacement>, mut game_event: EventWriter<GameEnded>, settings: Res<GameSettings>, mut game_state: ResMut<GameState>) {
    for _ in move_played.read() {
        if game_state.is_terminal() {
			let winner =  game_state.check_winner();
			game_event.write(GameEnded { winner });
			
            if winner.is_some()  {
				println!("End game ! Winner is {:?}", winner);
            } else {
				println!("An error occured, game is terminal but no winner is found, the game is a draw.");
			}
			return;
        }
		if game_state.current_player == Player::Max {
			//request input, we are player
			info!("Awaiting user click");
		} else {
			//request ai input
			let placement = find_best_move(&mut game_state, settings.ai_depth);
			if placement.is_some() {
				let p = placement.unwrap();
				stone_placed.write(StonePlacement { x: p.0, y: p.1 });
			} else {
				game_event.write(GameEnded { winner: Some(Player::Max) });
				println!("Error occured while finding best move");
				return;
			}
		}
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
) {
    if buttons.just_pressed(MouseButton::Left) {
        for (interaction, cell) in interaction_query.iter_mut() {
            if *interaction == Interaction::Pressed
                && game_state.board.get_player(cell.x, cell.y).is_none()
            {
				println!("Writing stone placement");
				stone_placement.write(StonePlacement {
					x: cell.x,
					y: cell.y
				});
            }
        }
    }
}

pub fn place_stone(mut commands: Commands,
	available_spot: Query<
        (Entity, &GridCell),
        (With<AvailableArea>, Without<Stone>),
    >,
	board_query: Query<Entity, With<BoardRoot>>,
	mut game_state: ResMut<GameState>,
	mut ev_stone_placed: EventReader<StonePlacement>,
	mut move_played: EventWriter<MovePlayed>) {
		for ev in ev_stone_placed.read() {
			info!("Stone placed at x: {}, y: {}", ev.x, ev.y);
			game_state.make_move((ev.x, ev.y));
			let player = game_state.current_player;
			let color = match player {
				Player::Min => Color::BLACK,
				Player::Max => Color::WHITE,
			};

			// ✅ Spawn inside the board
			if let Ok(board_entity) = board_query.single() {
				commands.entity(board_entity).with_children(|builder| {
					builder.spawn((
						BoardUtils::stone_node(ev.x, ev.y, BoardUtils::STONE_SIZE),
						BackgroundColor(color),
						Stone(player),
						BorderRadius::all(Val::Percent(50.0)),
						ZIndex(20),
						OnGameScreen,
					));
				});
			}
			let curr_entity = available_spot.iter().find(|&x| x.1.x == ev.x && x.1.y == ev.y);
			if let Some((entity, _)) = curr_entity {
				println!("Placed stone");
				commands.entity(entity)
					.insert(Stone(player))
					.remove::<AvailableArea>()
					.insert(BackgroundColor(Color::NONE));
			} else {
				println!("Error placing stone")
			}
			move_played.write(MovePlayed);
		}
}