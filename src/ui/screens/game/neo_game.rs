use bevy::prelude::*;

use crate::{core::state::GameState, ui::{app::{AppState, GameSettings}, screens::{game::{board::{BoardUtils, PreviewDot}, game::{AvailableArea, GridCell, OnGameScreen}, settings::spawn_settings_panel}, utils::despawn_screen}}};

#[derive(Event)]
pub struct BoardUpdate;





pub fn game_plugin(app: &mut App) {
    app
		.add_event::<GameEnded>()
		.add_event::<StonePlacement>()
		.add_event::<BoardUpdate>()
		.add_event::<MovePlayed>()
		.add_systems(OnEnter(AppState::Game), (setup_game_ui, update_available_placement).chain())
		.add_systems(
            Update,
            handle_player_click
				.run_if(in_state(AppState::Game))
        )
		.add_systems(
            Update,
            update_available_placement
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

fn update_available_placement(
    mut commands: Commands,
	mut ev_board_update: EventReader<BoardUpdate>,
    game_state: Res<GameState>,
    parents: Query<(Entity, &Children, &GridCell), With<GridCell>>,
    mut dots: Query<(&mut BackgroundColor, &mut Visibility, Entity), With<PreviewDot>>
) {
    let possible_moves = game_state.get_possible_moves();
	info!("Updating stone preview...");	
    // Consume events
	for _ in ev_board_update.read() {}
	
	for (entity, children, cell) in parents.iter() {
		if possible_moves.contains(&(cell.x, cell.y)) {
			for &child in children {
				if let Ok((mut bg, mut visibility, child_entity)) = dots.get_mut(child) {
					*bg = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.4));
					*visibility = Visibility::Visible;
					commands.entity(child_entity).insert(PreviewDot);
					commands.entity(entity).insert(AvailableArea);
				}
			}
		} else {
			for &child in children {
				if let Ok((mut bg, mut visibility, child_entity)) = dots.get_mut(child) {
					*bg = BackgroundColor(Color::NONE);
					*visibility = Visibility::Hidden;
					commands.entity(child_entity).remove::<PreviewDot>();
					commands.entity(entity).remove::<AvailableArea>();
				}
			}
		}
	}
}