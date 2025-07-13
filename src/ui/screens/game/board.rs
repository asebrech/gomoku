use bevy::prelude::*;
use crate::ui::{app::GameSettings, screens::game::game::{GridCell, OnGameScreen, PreviewDot}};

#[derive(Component)]
pub struct BoardRoot;

pub struct BoardUtils;

impl BoardUtils {
    pub const CELL_SIZE: f32 = 32.0;
    pub const LINE_THICKNESS: f32 = 2.0;
    pub const STONE_SIZE: f32 = 24.0;
    pub const PREVIEW_SIZE: f32 = 16.0;
    
    pub fn spawn_board(builder: &mut ChildSpawnerCommands, game_settings: &GameSettings) {
        builder
            .spawn((
                Node {
                    display: Display::Grid,
                    width: Val::Px((game_settings.board_size as f32) * Self::CELL_SIZE),
                    height: Val::Px((game_settings.board_size as f32) * Self::CELL_SIZE),
                    position_type: PositionType::Relative,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.95, 0.85, 0.7)),
                OnGameScreen,
                BoardRoot,
            ))
            .with_children(|builder| {
                Self::draw_board(builder, game_settings.board_size);
                Self::insert_intersection_hitboxes(builder, game_settings.board_size);
            });
        
        info!("Board initialized with size {}x{}", game_settings.board_size, game_settings.board_size);
    }
    
    fn draw_board(builder: &mut ChildSpawnerCommands, board_size: usize) {
        info!("Drawing board grid lines...");
        
        // Vertical lines
        for i in 0..board_size {
            builder.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(i as f32 * Self::CELL_SIZE + Self::CELL_SIZE / 2.0 - Self::LINE_THICKNESS / 2.0),
                    top: Val::Px(0.0),
                    width: Val::Px(Self::LINE_THICKNESS),
                    height: Val::Px(Self::CELL_SIZE * board_size as f32),
                    ..default()
                },
                BackgroundColor(Color::BLACK),
            ));
        }
        
        // Horizontal lines
        for i in 0..board_size {
            builder.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(i as f32 * Self::CELL_SIZE + Self::CELL_SIZE / 2.0 - Self::LINE_THICKNESS / 2.0),
                    width: Val::Px(Self::CELL_SIZE * board_size as f32),
                    height: Val::Px(Self::LINE_THICKNESS),
                    ..default()
                },
                BackgroundColor(Color::BLACK),
            ));
        }
    }
    
    fn insert_intersection_hitboxes(builder: &mut ChildSpawnerCommands, board_size: usize) {
        info!("Creating interactive hitboxes...");
        
        for y in 0..board_size {
            for x in 0..board_size {
                builder
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(x as f32 * Self::CELL_SIZE),
                            top: Val::Px(y as f32 * Self::CELL_SIZE),
                            width: Val::Px(Self::CELL_SIZE),
                            height: Val::Px(Self::CELL_SIZE),
                            ..default()
                        },
                        ZIndex(10),
                        Interaction::default(),
                        Visibility::Visible,
                        GridCell { x, y },
                    ))
                    .with_children(|builder| {
                        // Preview dot for showing possible moves
                        builder.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px((Self::CELL_SIZE - Self::PREVIEW_SIZE) / 2.0),
                                top: Val::Px((Self::CELL_SIZE - Self::PREVIEW_SIZE) / 2.0),
                                width: Val::Px(Self::PREVIEW_SIZE),
                                height: Val::Px(Self::PREVIEW_SIZE),
                                ..default()
                            },
                            BorderRadius::all(Val::Percent(50.0)),
                            ZIndex(9),
                            Visibility::Hidden,
                            BackgroundColor(Color::NONE),
                            PreviewDot,
                        ));
                    });
            }
        }
    }
    
    pub fn stone_node(x: usize, y: usize, size: f32) -> Node {
        let offset = (Self::CELL_SIZE - size) / 2.0;
        
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(x as f32 * Self::CELL_SIZE + offset),
            top: Val::Px(y as f32 * Self::CELL_SIZE + offset),
            width: Val::Px(size),
            height: Val::Px(size),
            ..default()
        }
    }
}