use bevy::prelude::*;
use crate::{
    ui::{
        app::AppState,
        config::GameConfig,
        components::button::{ButtonBuilder, ButtonStyle, ButtonSize, button_interaction_system},
        screens::{
            utils::despawn_screen,
            splash::PreloadedStones,
        },
    },
};

#[derive(Component)]
struct OnTutorialScreen;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum TutorialState {
    #[default]
    WinExample,
    CaptureExample,
}

pub fn tutorial_plugin(app: &mut App) {
    app
        .init_state::<TutorialState>()
        .add_systems(OnEnter(AppState::HowToPlay), (reset_tutorial_state, setup_tutorial).chain())
        .add_systems(
            Update,
            (
                button_interaction_system,
                handle_tutorial_navigation,
                update_tutorial_content,
            ).run_if(in_state(AppState::HowToPlay)),
        )
        .add_systems(OnExit(AppState::HowToPlay), despawn_screen::<OnTutorialScreen>);
}

fn reset_tutorial_state(mut tutorial_state: ResMut<NextState<TutorialState>>) {
    // Always reset to the default state when entering the tutorial
    tutorial_state.set(TutorialState::WinExample);
}

fn setup_tutorial(mut commands: Commands, config: Res<GameConfig>, preloaded_stones: Res<PreloadedStones>) {
    let colors = &config.colors;
    
    commands
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(colors.background.clone().into()),
            OnTutorialScreen,
        ))
        .with_children(|builder| {
            // Title
            builder.spawn((
                Text::new("How to Play Gomoku"),
                TextFont {
                    font_size: config.ui.font_sizes.title,
                    ..default()
                },
                TextColor(colors.accent.clone().into()),
            ));

            // Tutorial content container - spawn with initial content
            builder.spawn((
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    width: Val::Percent(90.0),
                    height: Val::Percent(70.0),
                    justify_content: JustifyContent::SpaceEvenly,
                    align_items: AlignItems::Center,
                    ..default()
                },
                TutorialContent,
            )).with_children(|builder| {
                // Spawn initial content (Win Example)
                spawn_win_example(builder, &preloaded_stones, colors);
            });

            // Navigation buttons
            builder.spawn((
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(20.0),
                    ..default()
                },
            )).with_children(|builder| {
                // Win Example Button
                ButtonBuilder::new("Win Example")
                    .with_style(ButtonStyle::Secondary)
                    .with_size(ButtonSize::Large)
                    .with_margin(UiRect::all(Val::Px(12.0)))
                    .spawn(builder, TutorialButton::WinExample, colors);
                
                // Capture Example Button
                ButtonBuilder::new("Capture Example")
                    .with_style(ButtonStyle::Secondary)
                    .with_size(ButtonSize::Large)
                    .with_margin(UiRect::all(Val::Px(12.0)))
                    .spawn(builder, TutorialButton::CaptureExample, colors);
                
                // Back Button
                ButtonBuilder::new("Back to Menu")
                    .with_style(ButtonStyle::Secondary)
                    .with_size(ButtonSize::Large)
                    .with_margin(UiRect::all(Val::Px(12.0)))
                    .spawn(builder, TutorialButton::BackToMenu, colors);
            });
        });
}

#[derive(Component)]
struct TutorialContent;

#[derive(Component)]
enum TutorialButton {
    WinExample,
    CaptureExample,
    BackToMenu,
}

fn handle_tutorial_navigation(
    mut tutorial_state: ResMut<NextState<TutorialState>>,
    mut app_state: ResMut<NextState<AppState>>,
    interaction_query: Query<(&Interaction, &TutorialButton), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            match button {
                TutorialButton::WinExample => tutorial_state.set(TutorialState::WinExample),
                TutorialButton::CaptureExample => tutorial_state.set(TutorialState::CaptureExample),
                TutorialButton::BackToMenu => app_state.set(AppState::Menu),
            }
        }
    }
}

fn update_tutorial_content(
    mut commands: Commands,
    tutorial_state: Res<State<TutorialState>>,
    content_query: Query<Entity, With<TutorialContent>>,
    children_query: Query<&Children>,
    preloaded_stones: Res<PreloadedStones>,
    config: Res<GameConfig>,
) {
    if tutorial_state.is_changed() {
        let colors = &config.colors;
        
        // Clear existing content
        if let Ok(content_entity) = content_query.single() {
            // Manually despawn all children
            if let Ok(children) = children_query.get(content_entity) {
                for child in children.iter() {
                    commands.entity(child).despawn();
                }
            }
            
            // Add new content based on state
            commands.entity(content_entity).with_children(|builder| {
                match tutorial_state.get() {
                    TutorialState::WinExample => spawn_win_example(builder, &preloaded_stones, colors),
                    TutorialState::CaptureExample => spawn_capture_example(builder, &preloaded_stones, colors),
                }
            });
        }
    }
}

fn spawn_win_example(
    builder: &mut ChildSpawnerCommands, 
    preloaded_stones: &PreloadedStones,
    colors: &crate::ui::config::ColorConfig,
) {
    // Left side: explanation
    builder.spawn((
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            width: Val::Percent(45.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(20.0),
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        BorderColor(colors.secondary.clone().into()),
        BorderRadius::all(Val::Px(10.0)),
    )).with_children(|builder| {
        builder.spawn((
            Text::new("How to Win"),
            TextFont {
                font_size: 32.0,
                ..default()
            },
            TextColor(colors.accent.clone().into()),
        ));
        
        builder.spawn((
            Text::new("Connect 5 stones in a row to win!\n\nYou can connect:\n- Horizontally\n- Vertically\n- Diagonally\n\nThe example shows a winning horizontal line."),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(colors.text_primary.clone().into()),
        ));
    });

    // Right side: demo board
    builder.spawn((
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            width: Val::Percent(45.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
    )).with_children(|builder| {
        spawn_demo_board(builder, preloaded_stones, create_win_pattern());
    });
}

fn spawn_capture_example(
    builder: &mut ChildSpawnerCommands, 
    preloaded_stones: &PreloadedStones,
    colors: &crate::ui::config::ColorConfig,
) {
    // Left side: explanation
    builder.spawn((
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            width: Val::Percent(45.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(20.0),
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        BorderColor(colors.secondary.clone().into()),
        BorderRadius::all(Val::Px(10.0)),
    )).with_children(|builder| {
        builder.spawn((
            Text::new("How to Capture"),
            TextFont {
                font_size: 32.0,
                ..default()
            },
            TextColor(colors.accent.clone().into()),
        ));
        
        builder.spawn((
            Text::new("Surround 2 enemy stones to capture them!\n\nPattern: YOUR - ENEMY - ENEMY - YOUR\n\nThe surrounded stones are removed from the board.\n\nExample shows blue capturing pink stones."),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(colors.text_primary.clone().into()),
        ));
    });

    // Right side: demo board
    builder.spawn((
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            width: Val::Percent(45.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
    )).with_children(|builder| {
        spawn_demo_board(builder, preloaded_stones, create_capture_pattern());
    });
}

fn spawn_demo_board(builder: &mut ChildSpawnerCommands, preloaded_stones: &PreloadedStones, pattern: Vec<(usize, usize, StoneType)>) {
    let board_size = 9; // Smaller demo board
    let cell_size = 40.0;
    let total_size = (board_size as f32) * cell_size;
    let line_thickness = 2.0;

    builder.spawn((
        Node {
            display: Display::Grid,
            width: Val::Px(total_size),
            height: Val::Px(total_size),
            position_type: PositionType::Relative,
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.0, 0.08, 0.9)),
        BorderRadius::all(Val::Px(8.0)),
    )).with_children(|builder| {
        // Draw vertical lines at cell centers (creating intersections)
        for i in 0..board_size {
            builder.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(i as f32 * cell_size + cell_size / 2.0 - line_thickness / 2.0),
                    top: Val::Px(0.0),
                    width: Val::Px(line_thickness),
                    height: Val::Px(cell_size * board_size as f32),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.8, 0.4, 1.0, 0.4)),
            ));
        }

        // Draw horizontal lines at cell centers (creating intersections)
        for i in 0..board_size {
            builder.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(i as f32 * cell_size + cell_size / 2.0 - line_thickness / 2.0),
                    width: Val::Px(cell_size * board_size as f32),
                    height: Val::Px(line_thickness),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.8, 0.4, 1.0, 0.4)),
            ));
        }

        // Add stones according to pattern - now at intersections (cell centers)
        for (x, y, stone_type) in pattern {
            let stone_handle = match stone_type {
                StoneType::Pink => preloaded_stones.pink_stone.clone(),
                StoneType::Blue => preloaded_stones.blue_stone.clone(),
            };

            let stone_size = 30.0;
            builder.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(x as f32 * cell_size + cell_size / 2.0 - stone_size / 2.0),
                    top: Val::Px(y as f32 * cell_size + cell_size / 2.0 - stone_size / 2.0),
                    width: Val::Px(stone_size),
                    height: Val::Px(stone_size),
                    ..default()
                },
                ImageNode::new(stone_handle),
                ZIndex(10),
            ));
        }
    });
}

#[derive(Clone)]
enum StoneType {
    Pink,
    Blue,
}

fn create_win_pattern() -> Vec<(usize, usize, StoneType)> {
    vec![
        // Winning horizontal line
        (2, 4, StoneType::Pink),
        (3, 4, StoneType::Pink),
        (4, 4, StoneType::Pink),
        (5, 4, StoneType::Pink),
        (6, 4, StoneType::Pink),
        // Some blue stones for context
        (3, 3, StoneType::Blue),
        (4, 3, StoneType::Blue),
        (3, 5, StoneType::Blue),
        (5, 5, StoneType::Blue),
    ]
}

fn create_capture_pattern() -> Vec<(usize, usize, StoneType)> {
    vec![
        // Capture pattern: Blue - Pink - Pink - Blue (horizontal)
        (2, 4, StoneType::Blue),   // Capturing stone
        (3, 4, StoneType::Pink),   // Captured stone 1
        (4, 4, StoneType::Pink),   // Captured stone 2
        (5, 4, StoneType::Blue),   // Capturing stone
        // Some context stones
        (3, 3, StoneType::Blue),
        (4, 3, StoneType::Blue),
        (2, 5, StoneType::Pink),
        (5, 5, StoneType::Pink),
    ]
}
