use bevy::prelude::*;
use crate::ui::{app::GameSettings, screens::game::game::{GridCell, OnGameScreen}};

#[derive(Component)]
pub struct BoardRoot;

#[derive(Component)]
pub struct PreviewDot;

pub struct BoardUtils;

impl BoardUtils {
    pub const CELL_SIZE: f32 = 32.0;
    pub const LINE_THICKNESS: f32 = 2.0;
    pub const STONE_SIZE: f32 = 28.0; // Increased from 24.0 to 28.0 for bigger stones
    pub const PREVIEW_SIZE: f32 = 8.0; // Reduced from 16.0 to 8.0 - much smaller available spots
    
    /// Calculate the spacing between major grid lines based on board size
    /// This ensures the board looks balanced regardless of size
    fn calculate_major_line_spacing(board_size: usize) -> usize {
        match board_size {
            15 => 5,  // 15x15: major lines every 5 (0, 5, 10, 15)
            16 => 5,  // 16x16: major lines every 5 (0, 5, 10, 15)
            17 => 4,  // 17x17: major lines every 4 (0, 4, 8, 12, 16)
            18 => 6,  // 18x18: major lines every 6 (0, 6, 12, 18)
            19 => 6,  // 19x19: major lines every 6 (0, 6, 12, 18)
            _ => {
                // For other sizes, use a smart algorithm
                // Aim for 3-4 segments, so divide by 3 or 4
                if board_size % 3 == 0 {
                    board_size / 3
                } else if board_size % 4 == 0 {
                    board_size / 4
                } else {
                    // Default to roughly dividing into thirds
                    (board_size + 2) / 3
                }
            }
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
    
    pub fn spawn_board(builder: &mut ChildSpawnerCommands, game_settings: &GameSettings) {
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
                BackgroundColor(Color::srgba(1.0, 0.0, 1.0, 0.1)), // Magenta outer glow
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
                        BackgroundColor(Color::srgba(0.02, 0.0, 0.08, 0.9)), // Even darker with slight transparency
                        BorderRadius::all(Val::Px(8.0)),
                        BoardRoot,
                    ))
                    .with_children(|builder| {
                        Self::draw_board(builder, game_settings.board_size);
                        Self::insert_intersection_hitboxes(builder, game_settings.board_size);
                    });
            });
        
        info!("Board initialized with size {}x{}", game_settings.board_size, game_settings.board_size);
    }
    
    fn draw_board(builder: &mut ChildSpawnerCommands, board_size: usize) {
        info!("Drawing board grid lines...");
        
        // Calculate dynamic major line spacing based on board size
        let major_line_spacing = Self::calculate_major_line_spacing(board_size);
        info!("Using major line spacing of {} for board size {}", major_line_spacing, board_size);
        
        // Draw clean vertical lines
        for i in 0..board_size {
            let is_major_line = i % major_line_spacing == 0 || i == board_size - 1;
            let line_thickness = if is_major_line { 
                Self::LINE_THICKNESS * 1.5
            } else { 
                Self::LINE_THICKNESS * 0.7
            };
            
            let base_color = if is_major_line {
                Color::srgba(0.7, 0.08, 0.7, 0.7) // Dimmed magenta for major lines with transparency
            } else {
                Color::srgba(0.08, 0.4, 0.6, 0.4) // More subtle cyan for regular lines with transparency
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
                    BackgroundColor(Color::srgba(0.7, 0.08, 0.7, 0.15)), // Reduced glow effect
                    ZIndex(-1),
                ));
            }
        }
        
        // Draw clean horizontal lines
        for i in 0..board_size {
            let is_major_line = i % major_line_spacing == 0 || i == board_size - 1;
            let line_thickness = if is_major_line { 
                Self::LINE_THICKNESS * 1.5
            } else { 
                Self::LINE_THICKNESS * 0.7
            };
            
            let base_color = if is_major_line {
                Color::srgba(0.7, 0.08, 0.7, 0.7) // Dimmed magenta for major lines with transparency
            } else {
                Color::srgba(0.08, 0.4, 0.6, 0.4) // More subtle cyan for regular lines with transparency
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
                    BackgroundColor(Color::srgba(0.7, 0.08, 0.7, 0.15)), // Reduced glow effect
                    ZIndex(-1),
                ));
            }
        }
        
        // Simplified intersection points - only at major line crossings
        for y in (0..board_size).step_by(major_line_spacing) {
            for x in (0..board_size).step_by(major_line_spacing) {
                builder.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(x as f32 * Self::CELL_SIZE + Self::CELL_SIZE / 2.0 - 1.5),
                        top: Val::Px(y as f32 * Self::CELL_SIZE + Self::CELL_SIZE / 2.0 - 1.5),
                        width: Val::Px(3.0),
                        height: Val::Px(3.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(1.0, 0.8, 1.0)), // Bright white-magenta
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
                BackgroundColor(Color::srgb(1.0, 0.8, 1.0)), // Bright white-magenta
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
                BackgroundColor(Color::srgb(1.0, 0.2, 0.8)), // Accent color
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
                BackgroundColor(Color::srgba(1.0, 0.2, 0.8, 0.2)), // Much more subtle
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