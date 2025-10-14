use bevy::prelude::*;
use crate::ui::{app::GameSettings, config::GameConfig, screens::game::game::{GridCell, OnGameScreen}};

#[derive(Component)]
pub struct BoardRoot;

#[derive(Component)]
pub struct PreviewDot;

pub struct BoardUtils;

impl BoardUtils {
    pub const CELL_SIZE: f32 = 32.0;
    pub const LINE_THICKNESS: f32 = 2.0;
    pub const STONE_SIZE: f32 = 26.0; // Slightly smaller circular stones
    pub const PREVIEW_SIZE: f32 = 8.0; // Reduced from 16.0 to 8.0 - much smaller available spots
    
    fn calculate_major_line_positions(board_size: usize) -> Vec<usize> {
        match board_size {
            5 => vec![0, 2, 4],
            6 => vec![0, 5],
            7 => vec![0, 3, 6],
            8 => vec![0, 2, 5, 7],
            9 => vec![0, 4, 8],
            10 => vec![0, 3, 6, 9],
            11 => vec![0, 3, 7, 10],
            12 => vec![0, 3, 8, 11],
            13 => vec![0, 4, 8, 12],
            14 => vec![0, 4, 9, 13],
            15 => vec![0, 4, 10, 14],
            16 => vec![0, 5, 10, 15],
            17 => vec![0, 5, 11, 16],
            18 => vec![0, 5, 12, 17],
            19 => vec![0, 6, 12, 18],
            20 => vec![0, 6, 13, 19],
            _ => vec![0, board_size - 1],
        }
    }
    
    /// Calculate star point positions dynamically based on board size
    /// Star points are decorative markers that help orient the board
    fn calculate_star_points(board_size: usize) -> Vec<(usize, usize)> {
        match board_size {
            // Traditional patterns for standard sizes
            19 => vec![
                (3, 3), (3, 9), (3, 15),
                (9, 3), (9, 9), (9, 15),
                (15, 3), (15, 9), (15, 15)
            ],
            15 => vec![
                (3, 3), (3, 11),
                (7, 7),
                (11, 3), (11, 11)
            ],
            13 => vec![
                (3, 3), (3, 9),
                (6, 6),
                (9, 3), (9, 9)
            ],
            // Dynamic calculation for other sizes
            _ => {
                let mut positions = Vec::new();
                
                // Only add star points if board is large enough
                if board_size >= 9 {
                    let edge_offset = if board_size <= 11 { 2 } else { 3 };
                    let center = board_size / 2;
                    
                    // Add 4 corner star points
                    positions.push((edge_offset, edge_offset));
                    positions.push((edge_offset, board_size - 1 - edge_offset));
                    positions.push((board_size - 1 - edge_offset, edge_offset));
                    positions.push((board_size - 1 - edge_offset, board_size - 1 - edge_offset));
                    
                    // Add center point if board size is odd
                    if board_size % 2 == 1 {
                        positions.push((center, center));
                    }
                    
                    // Add edge midpoints for larger boards
                    if board_size >= 15 {
                        positions.push((edge_offset, center));
                        positions.push((center, edge_offset));
                        positions.push((board_size - 1 - edge_offset, center));
                        positions.push((center, board_size - 1 - edge_offset));
                    }
                }
                
                positions
            }
        }
    }
    
    pub fn spawn_board(builder: &mut ChildSpawnerCommands, game_settings: &GameSettings, config: &GameConfig) {
        let colors = &config.colors;
        let total_size = (game_settings.board_size as f32) * Self::CELL_SIZE;
        
        // Outer glow container
        builder
            .spawn((
                Node {
                    display: Display::Grid,
                    width: Val::Px(total_size + 20.0),
                    height: Val::Px(total_size + 20.0),
                    position_type: PositionType::Relative,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(colors.accent.r, colors.accent.g, colors.accent.b, 0.1)),
                BorderRadius::all(Val::Px(15.0)),
                OnGameScreen,
            ))
            .with_children(|builder| {
                // Main board
                builder
                    .spawn((
                        Node {
                            display: Display::Grid,
                            width: Val::Px(total_size),
                            height: Val::Px(total_size),
                            position_type: PositionType::Relative,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(colors.background.r * 0.1, colors.background.g * 0.1, colors.background.b * 0.1, 0.9)),
                        BorderRadius::all(Val::Px(8.0)),
                        BoardRoot,
                    ))
                    .with_children(|builder| {
                        Self::draw_board(builder, game_settings.board_size, colors);
                        Self::insert_intersection_hitboxes(builder, game_settings.board_size);
                    });
            });
        
        info!("Board initialized with size {}x{}", game_settings.board_size, game_settings.board_size);
    }
    
    fn draw_board(builder: &mut ChildSpawnerCommands, board_size: usize, colors: &crate::ui::config::ColorConfig) {
        info!("Drawing board grid lines...");
        
        // Calculate major line positions for balanced spacing
        let major_positions = Self::calculate_major_line_positions(board_size);
        info!("Major line positions for board size {}: {:?}", board_size, major_positions);
        
        // Draw clean vertical lines
        for i in 0..board_size {
            let is_major_line = major_positions.contains(&i);
            
            let line_thickness = if is_major_line { 
                Self::LINE_THICKNESS * 1.5
            } else { 
                Self::LINE_THICKNESS * 0.7
            };
            
            let base_color = if is_major_line {
                // Major lines: dim uniformly to preserve theme color ratios
                Color::srgba(colors.primary.r * 0.7, colors.primary.g * 0.7, colors.primary.b * 0.7, 0.5)
            } else {
                // Regular lines: dim uniformly but more transparent
                Color::srgba(colors.secondary.r * 0.6, colors.secondary.g * 0.6, colors.secondary.b * 0.6, 0.3)
            };
            
            // Main line
            builder.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(i as f32 * Self::CELL_SIZE + Self::CELL_SIZE / 2.0 - line_thickness / 2.0),
                    top: Val::Px(0.0),
                    width: Val::Px(line_thickness),
                    height: Val::Px(Self::CELL_SIZE * board_size as f32),
                    ..default()
                },
                BackgroundColor(base_color),
            ));
            
            // Add glow effect for major lines only
            if is_major_line {
                builder.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(i as f32 * Self::CELL_SIZE + Self::CELL_SIZE / 2.0 - (line_thickness + 2.0) / 2.0),
                        top: Val::Px(-1.0),
                        width: Val::Px(line_thickness + 2.0),
                        height: Val::Px(Self::CELL_SIZE * board_size as f32 + 2.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(colors.primary.r * 0.7, colors.primary.g * 0.7, colors.primary.b * 0.7, 0.08)),
                    ZIndex(-1),
                ));
            }
        }
        
        // Draw clean horizontal lines
        for i in 0..board_size {
            let is_major_line = major_positions.contains(&i);
            
            let line_thickness = if is_major_line { 
                Self::LINE_THICKNESS * 1.5
            } else { 
                Self::LINE_THICKNESS * 0.7
            };
            
            let base_color = if is_major_line {
                // Major lines: dim uniformly to preserve theme color ratios
                Color::srgba(colors.primary.r * 0.7, colors.primary.g * 0.7, colors.primary.b * 0.7, 0.5)
            } else {
                // Regular lines: dim uniformly but more transparent
                Color::srgba(colors.secondary.r * 0.6, colors.secondary.g * 0.6, colors.secondary.b * 0.6, 0.3)
            };
            
            // Main line
            builder.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(i as f32 * Self::CELL_SIZE + Self::CELL_SIZE / 2.0 - line_thickness / 2.0),
                    width: Val::Px(Self::CELL_SIZE * board_size as f32),
                    height: Val::Px(line_thickness),
                    ..default()
                },
                BackgroundColor(base_color),
            ));
            
            // Add glow effect for major lines only
            if is_major_line {
                builder.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(-1.0),
                        top: Val::Px(i as f32 * Self::CELL_SIZE + Self::CELL_SIZE / 2.0 - (line_thickness + 2.0) / 2.0),
                        width: Val::Px(Self::CELL_SIZE * board_size as f32 + 2.0),
                        height: Val::Px(line_thickness + 2.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(colors.primary.r * 0.7, colors.primary.g * 0.7, colors.primary.b * 0.7, 0.08)),
                    ZIndex(-1),
                ));
            }
        }
        
        // Simplified intersection points - only at major line crossings
        for &y in &major_positions {
            for &x in &major_positions {
                builder.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(x as f32 * Self::CELL_SIZE + Self::CELL_SIZE / 2.0 - 1.5),
                        top: Val::Px(y as f32 * Self::CELL_SIZE + Self::CELL_SIZE / 2.0 - 1.5),
                        width: Val::Px(3.0),
                        height: Val::Px(3.0),
                        ..default()
                    },
                    BackgroundColor(colors.accent.clone().into()),
                    BorderRadius::all(Val::Percent(50.0)),
                    ZIndex(2),
                ));
            }
        }
        
        // Add corner points
        let corners = vec![(0, 0), (0, board_size - 1), (board_size - 1, 0), (board_size - 1, board_size - 1)];
        for (x, y) in corners {
            builder.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(x as f32 * Self::CELL_SIZE + Self::CELL_SIZE / 2.0 - 1.5),
                    top: Val::Px(y as f32 * Self::CELL_SIZE + Self::CELL_SIZE / 2.0 - 1.5),
                    width: Val::Px(3.0),
                    height: Val::Px(3.0),
                    ..default()
                },
                BackgroundColor(colors.accent.clone().into()),
                BorderRadius::all(Val::Percent(50.0)),
                ZIndex(2),
            ));
        }
        
        // Add star points for traditional Go board look - dynamically calculated
        let star_positions = Self::calculate_star_points(board_size);
        
        for (x, y) in star_positions {
            builder.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(x as f32 * Self::CELL_SIZE + Self::CELL_SIZE / 2.0 - 3.0),
                    top: Val::Px(y as f32 * Self::CELL_SIZE + Self::CELL_SIZE / 2.0 - 3.0),
                    width: Val::Px(6.0),
                    height: Val::Px(6.0),
                    ..default()
                },
                BackgroundColor(colors.accent.clone().into()),
                BorderRadius::all(Val::Percent(50.0)),
            ));
            
            // Star point glow - more subtle
            builder.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(x as f32 * Self::CELL_SIZE + Self::CELL_SIZE / 2.0 - 4.0),
                    top: Val::Px(y as f32 * Self::CELL_SIZE + Self::CELL_SIZE / 2.0 - 4.0),
                    width: Val::Px(8.0),
                    height: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(colors.accent.r, colors.accent.g, colors.accent.b, 0.2)),
                BorderRadius::all(Val::Percent(50.0)),
                ZIndex(-1),
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