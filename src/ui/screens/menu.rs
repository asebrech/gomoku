
    use bevy::{
        app::AppExit,
        color::palettes::css::{AZURE, BISQUE, CRIMSON, PURPLE},
        ecs::relationship::RelatedSpawnerCommands,
        prelude::*,
    };

    use crate::ui::{app::{AppState, GameSettings}, screens::utils::despawn_screen};

    pub fn menu_plugin(app: &mut App) {
        app
            .init_state::<MenuState>()
            .add_systems(OnEnter(AppState::Menu), menu_setup)
            .add_systems(OnEnter(MenuState::Main), main_menu_setup)
            .add_systems(OnEnter(MenuState::Settings), settings_menu_setup)
            .add_systems(OnExit(MenuState::Main), despawn_screen::<OnMainMenuScreen>)
            .add_systems(
                OnExit(MenuState::Settings),
                despawn_screen::<OnSettingsMenuScreen>,
            )
            .add_systems(
                OnExit(MenuState::SettingsDisplay),
                despawn_screen::<OnDisplaySettingsMenuScreen>,
            )
            .add_systems(
                Update,
                setting_button::<GameSettings>.run_if(in_state(MenuState::SettingsSound)),
            )
            .add_systems(
                OnExit(MenuState::SettingsSound),
                despawn_screen::<OnSoundSettingsMenuScreen>,
            )
            .add_systems(
                Update,
                (menu_action, button_system, handle_settings_buttons).run_if(in_state(AppState::Menu)),
            );
    }

    #[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
    enum MenuState {
        Main,
        Settings,
		Load,
        SettingsDisplay,
        SettingsSound,
        #[default]
        Disabled,
    }

    #[derive(Component)]
    struct OnMainMenuScreen;

    #[derive(Component)]
    struct OnSettingsMenuScreen;

    #[derive(Component)]
    struct OnDisplaySettingsMenuScreen;

    #[derive(Component)]
    struct OnSoundSettingsMenuScreen;

    // Settings menu components
    #[derive(Component, Clone)]
    pub struct MenuSettingsBoardSizeButton {
        pub increment: bool,
    }

    #[derive(Component, Clone)]
    pub struct MenuSettingsChainToWinButton {
        pub increment: bool,
    }

    #[derive(Component, Clone)]
    pub struct MenuSettingsCaptureToWinButton {
        pub increment: bool,
    }

    #[derive(Component, Clone)]
    pub struct MenuSettingsAIDepthButton {
        pub increment: bool,
    }

    #[derive(Component, Clone)]
    pub struct MenuSettingsGameModeButton;

    #[derive(Component, Clone)]
    pub struct MenuSettingsTimeLimitButton {
        pub increment: bool,
    }

    #[derive(Component, Clone)]
    pub struct MenuSettingsTimeLimitToggleButton;

    // Text display markers for settings values
    #[derive(Component)]
    pub struct MenuBoardSizeText;

    #[derive(Component)]
    pub struct MenuChainToWinText;

    #[derive(Component)]
    pub struct MenuCaptureToWinText;

    #[derive(Component)]
    pub struct MenuGameModeText;

    #[derive(Component)]
    pub struct MenuAIDepthText;

    #[derive(Component)]
    pub struct MenuTimeLimitText;

    const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
    const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
    const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
    const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

    #[derive(Component)]
    struct SelectedOption;

    #[derive(Component)]
    enum MenuButtonAction {
		Load,
        Play,
        Settings,
        SettingsSound,
        BackToMainMenu,
        Quit,
    }

    fn button_system(
        mut interaction_query: Query<
            (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
            (Changed<Interaction>, With<Button>),
        >,
    ) {
        for (interaction, mut background_color, selected) in &mut interaction_query {
            *background_color = match (*interaction, selected) {
                (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
                (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
                (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
                (Interaction::None, None) => NORMAL_BUTTON.into(),
            }
        }
    }

    fn setting_button<T: Resource + Component + PartialEq + Copy>(
        interaction_query: Query<(&Interaction, &T, Entity), (Changed<Interaction>, With<Button>)>,
        selected_query: Single<(Entity, &mut BackgroundColor), With<SelectedOption>>,
        mut commands: Commands,
        mut setting: ResMut<T>,
    ) {
        let (previous_button, mut previous_button_color) = selected_query.into_inner();
        for (interaction, button_setting, entity) in &interaction_query {
            if *interaction == Interaction::Pressed && *setting != *button_setting {
                *previous_button_color = NORMAL_BUTTON.into();
                commands.entity(previous_button).remove::<SelectedOption>();
                commands.entity(entity).insert(SelectedOption);
                *setting = *button_setting;
            }
        }
    }

    fn menu_setup(mut menu_state: ResMut<NextState<MenuState>>) {
        menu_state.set(MenuState::Main);
    }

fn main_menu_setup(mut commands: Commands) {
    // Common style for all buttons on the screen
    let button_node = Node {
        width: Val::Px(300.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_font = TextFont {
        font_size: 33.0,
        ..default()
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            OnMainMenuScreen,
        ))
        .with_children(|parent| {
            insert_title_node(parent);

            parent
                .spawn((
                    Node {
						width: Val::Percent(100.0),
						height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        margin: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(CRIMSON.into()),
                ))
                .with_children(|parent| {
                    insert_load_play_node(parent, button_node.clone(), button_text_font.clone());
                    insert_settings_credit_quit_buttons(parent, button_node, button_text_font);
                });
        });
}

fn insert_title_node(parent: &mut RelatedSpawnerCommands<'_, ChildOf>) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                margin: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(AZURE.into()),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Mecha-Gomoku"),
                TextFont {
                    font_size: 67.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
        });
}

fn insert_load_play_node(
    parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
    button_node: Node,
    button_text_font: TextFont,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(BISQUE.into()),
        ))
        .with_children(|parent| {
            // Play button
            parent
                .spawn((
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Play,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("New Game"),
                        button_text_font.clone(),
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });

            // Settings button
            parent
                .spawn((
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Settings,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Settings"),
                        button_text_font.clone(),
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });

            // Quit button
            parent
                .spawn((
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Quit,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Quit"),
                        button_text_font.clone(),
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
        });
}

fn insert_settings_credit_quit_buttons(
    parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
    button_node: Node,
    button_text_font: TextFont,
) {
    parent
        .spawn((
            Node {
                width: Val::Px(400.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(PURPLE.into()),
        ))
        .with_children(|parent| {
            // Credits button
            parent
                .spawn((
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Play,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Credits"),
                        button_text_font.clone(),
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });

            // Quit button
            parent
                .spawn((
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Quit,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Quit"),
                        button_text_font.clone(),
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
        });
}

    fn menu_action(
        interaction_query: Query<
            (&Interaction, &MenuButtonAction),
            (Changed<Interaction>, With<Button>),
        >,
        mut app_exit_events: EventWriter<AppExit>,
        mut menu_state: ResMut<NextState<MenuState>>,
        mut game_state: ResMut<NextState<AppState>>,
    ) {
        for (interaction, menu_button_action) in &interaction_query {
            if *interaction == Interaction::Pressed {
                match menu_button_action {
                    MenuButtonAction::Quit => {
                        app_exit_events.write(AppExit::Success);
                    }
                    MenuButtonAction::Play => {
                        game_state.set(AppState::Game);
                        menu_state.set(MenuState::Disabled);
                    }
					MenuButtonAction::Load => {
						menu_state.set(MenuState::Load);
                    }
                    MenuButtonAction::Settings => menu_state.set(MenuState::Settings),
                    MenuButtonAction::SettingsSound => {
                        menu_state.set(MenuState::SettingsSound);
                    }
                    MenuButtonAction::BackToMainMenu => menu_state.set(MenuState::Main),
                }
            }
        }
    }

fn settings_menu_setup(mut commands: Commands, game_settings: Res<GameSettings>) {
    let button_style = Node {
        width: Val::Px(50.0),
        height: Val::Px(40.0),
        margin: UiRect::all(Val::Px(5.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = TextFont {
        font_size: 18.0,
        ..default()
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            OnSettingsMenuScreen,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Game Settings"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // Settings container
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(20.0),
                        width: Val::Px(600.0),
                        padding: UiRect::all(Val::Px(30.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.05, 0.05, 0.05)),
                    BorderColor(Color::srgb(0.3, 0.3, 0.3)),
                    BorderRadius::all(Val::Px(10.0)),
                ))
                .with_children(|builder| {
                    // Game Mode Setting
                    let game_mode = if game_settings.versus_ai { "vs AI" } else { "Multiplayer" };
                    spawn_menu_toggle_setting_row(builder, "Game Mode", game_mode, MenuSettingsGameModeButton, MenuGameModeText, button_style.clone(), button_text_style.clone());
                    
                    // Board Size Setting
                    let board_size = format!("{}x{}", game_settings.board_size, game_settings.board_size);
                    spawn_menu_interactive_setting_row(
                        builder, 
                        "Board Size", 
                        &board_size,
                        Some((MenuSettingsBoardSizeButton { increment: false }, MenuSettingsBoardSizeButton { increment: true })),
                        MenuBoardSizeText,
                        button_style.clone(),
                        button_text_style.clone(),
                    );

                    // Chain to Win Setting
                    let chain_to_win = game_settings.minimum_chain_to_win.to_string();
                    spawn_menu_interactive_setting_row(
                        builder, 
                        "Chain to Win", 
                        &chain_to_win,
                        Some((MenuSettingsChainToWinButton { increment: false }, MenuSettingsChainToWinButton { increment: true })),
                        MenuChainToWinText,
                        button_style.clone(),
                        button_text_style.clone(),
                    );

                    // Captures to Win Setting
                    let capture_to_win = game_settings.total_capture_to_win.to_string();
                    spawn_menu_interactive_setting_row(
                        builder, 
                        "Captures to Win", 
                        &capture_to_win,
                        Some((MenuSettingsCaptureToWinButton { increment: false }, MenuSettingsCaptureToWinButton { increment: true })),
                        MenuCaptureToWinText,
                        button_style.clone(),
                        button_text_style.clone(),
                    );

                    // AI Depth Setting
                    let ai_depth = game_settings.ai_depth.to_string();
                    spawn_menu_interactive_setting_row(
                        builder, 
                        "AI Depth", 
                        &ai_depth,
                        Some((MenuSettingsAIDepthButton { increment: false }, MenuSettingsAIDepthButton { increment: true })),
                        MenuAIDepthText,
                        button_style.clone(),
                        button_text_style.clone(),
                    );

                    // Time Limit Setting
                    let time_limit = if let Some(limit) = game_settings.time_limit {
                        limit.to_string()
                    } else {
                        "Disabled".to_string()
                    };
                    spawn_menu_time_limit_setting_row(
                        builder,
                        "Time Limit (ms)",
                        &time_limit,
                        MenuTimeLimitText,
                        button_style.clone(),
                        button_text_style.clone(),
                    );
                });

            // Back button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(50.0),
                        margin: UiRect::top(Val::Px(40.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::BackToMainMenu,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Back to Main Menu"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
        });
}

fn spawn_menu_interactive_setting_row<T: Component + Clone>(
    builder: &mut ChildSpawnerCommands,
    label: &str,
    value: &str,
    buttons: Option<(T, T)>, // (decrement, increment)
    text_marker: impl Component,
    button_style: Node,
    button_text_style: TextFont,
) {
    builder
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.08, 0.08)),
            BorderRadius::all(Val::Px(5.0)),
        ))
        .with_children(|builder| {
            // Label
            builder.spawn((
                Text::new(label),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));

            // Controls section
            builder
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(15.0),
                        ..default()
                    },
                ))
                .with_children(|builder| {
                    if let Some((decrement_button, _increment_button)) = &buttons {
                        // Decrement button
                        builder
                            .spawn((
                                Button,
                                button_style.clone(),
                                BackgroundColor(Color::srgb(0.6, 0.2, 0.2)),
                                BorderRadius::all(Val::Px(4.0)),
                                decrement_button.clone(),
                            ))
                            .with_children(|builder| {
                                builder.spawn((
                                    Text::new("-"),
                                    button_text_style.clone(),
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }

                    // Value display
                    builder.spawn((
                        Text::new(value),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 1.0)),
                        Node {
                            min_width: Val::Px(80.0),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        text_marker,
                    ));

                    if let Some((_, increment_button)) = &buttons {
                        // Increment button
                        builder
                            .spawn((
                                Button,
                                button_style.clone(),
                                BackgroundColor(Color::srgb(0.2, 0.6, 0.2)),
                                BorderRadius::all(Val::Px(4.0)),
                                increment_button.clone(),
                            ))
                            .with_children(|builder| {
                                builder.spawn((
                                    Text::new("+"),
                                    button_text_style.clone(),
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }
                });
        });
}

fn spawn_menu_toggle_setting_row<T: Component + Clone>(
    builder: &mut ChildSpawnerCommands,
    label: &str,
    value: &str,
    toggle_button: T,
    text_marker: impl Component,
    _button_style: Node,
    button_text_style: TextFont,
) {
    builder
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.08, 0.08)),
            BorderRadius::all(Val::Px(5.0)),
        ))
        .with_children(|builder| {
            // Label
            builder.spawn((
                Text::new(label),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));

            // Controls section
            builder
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(15.0),
                        ..default()
                    },
                ))
                .with_children(|builder| {
                    // Value display
                    builder.spawn((
                        Text::new(value),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 1.0)),
                        Node {
                            min_width: Val::Px(80.0),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        text_marker,
                    ));

                    // Toggle button
                    builder
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(80.0),
                                height: Val::Px(40.0),
                                margin: UiRect::all(Val::Px(5.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.3, 0.6)),
                            BorderRadius::all(Val::Px(4.0)),
                            toggle_button,
                        ))
                        .with_children(|builder| {
                            builder.spawn((
                                Text::new("Toggle"),
                                button_text_style.clone(),
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        });
}

fn spawn_menu_time_limit_setting_row(
    builder: &mut ChildSpawnerCommands,
    label: &str,
    value: &str,
    text_marker: impl Component,
    button_style: Node,
    button_text_style: TextFont,
) {
    builder
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.08, 0.08)),
            BorderRadius::all(Val::Px(5.0)),
        ))
        .with_children(|builder| {
            // Label
            builder.spawn((
                Text::new(label),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));

            // Controls section
            builder
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(15.0),
                        ..default()
                    },
                ))
                .with_children(|builder| {
                    // Decrement button
                    builder
                        .spawn((
                            Button,
                            button_style.clone(),
                            BackgroundColor(Color::srgb(0.6, 0.2, 0.2)),
                            BorderRadius::all(Val::Px(4.0)),
                            MenuSettingsTimeLimitButton { increment: false },
                        ))
                        .with_children(|builder| {
                            builder.spawn((
                                Text::new("-"),
                                button_text_style.clone(),
                                TextColor(Color::WHITE),
                            ));
                        });

                    // Value display
                    builder.spawn((
                        Text::new(value),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 1.0)),
                        Node {
                            min_width: Val::Px(80.0),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        text_marker,
                    ));

                    // Increment button
                    builder
                        .spawn((
                            Button,
                            button_style.clone(),
                            BackgroundColor(Color::srgb(0.2, 0.6, 0.2)),
                            BorderRadius::all(Val::Px(4.0)),
                            MenuSettingsTimeLimitButton { increment: true },
                        ))
                        .with_children(|builder| {
                            builder.spawn((
                                Text::new("+"),
                                button_text_style.clone(),
                                TextColor(Color::WHITE),
                            ));
                        });

                    // Toggle enable/disable button
                    builder
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(80.0),
                                height: Val::Px(40.0),
                                margin: UiRect::all(Val::Px(5.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.3, 0.6)),
                            BorderRadius::all(Val::Px(4.0)),
                            MenuSettingsTimeLimitToggleButton,
                        ))
                        .with_children(|builder| {
                            builder.spawn((
                                Text::new("Toggle"),
                                button_text_style.clone(),
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        });
}

fn handle_settings_buttons(
    mut game_settings: ResMut<GameSettings>,
    mut button_query: Query<
        (&Interaction, Option<&MenuSettingsBoardSizeButton>, Option<&MenuSettingsChainToWinButton>, 
         Option<&MenuSettingsCaptureToWinButton>, Option<&MenuSettingsAIDepthButton>, 
         Option<&MenuSettingsTimeLimitButton>, Option<&MenuSettingsGameModeButton>, 
         Option<&MenuSettingsTimeLimitToggleButton>),
        (Changed<Interaction>, With<Button>)
    >,
    mut board_size_query: Query<&mut Text, (With<MenuBoardSizeText>, Without<MenuChainToWinText>, Without<MenuCaptureToWinText>, Without<MenuGameModeText>, Without<MenuAIDepthText>, Without<MenuTimeLimitText>)>,
    mut chain_to_win_query: Query<&mut Text, (With<MenuChainToWinText>, Without<MenuBoardSizeText>, Without<MenuCaptureToWinText>, Without<MenuGameModeText>, Without<MenuAIDepthText>, Without<MenuTimeLimitText>)>,
    mut capture_to_win_query: Query<&mut Text, (With<MenuCaptureToWinText>, Without<MenuBoardSizeText>, Without<MenuChainToWinText>, Without<MenuGameModeText>, Without<MenuAIDepthText>, Without<MenuTimeLimitText>)>,
    mut game_mode_query: Query<&mut Text, (With<MenuGameModeText>, Without<MenuBoardSizeText>, Without<MenuChainToWinText>, Without<MenuCaptureToWinText>, Without<MenuAIDepthText>, Without<MenuTimeLimitText>)>,
    mut ai_depth_query: Query<&mut Text, (With<MenuAIDepthText>, Without<MenuBoardSizeText>, Without<MenuChainToWinText>, Without<MenuCaptureToWinText>, Without<MenuGameModeText>, Without<MenuTimeLimitText>)>,
    mut time_limit_query: Query<&mut Text, (With<MenuTimeLimitText>, Without<MenuBoardSizeText>, Without<MenuChainToWinText>, Without<MenuCaptureToWinText>, Without<MenuGameModeText>, Without<MenuAIDepthText>)>,
) {
    for (interaction, board_size, chain_to_win, capture_to_win, ai_depth, time_limit, game_mode, time_limit_toggle) in button_query.iter_mut() {
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
                    if game_settings.ai_depth < 15 {
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

            // Game Mode Button
            if game_mode.is_some() {
                game_settings.versus_ai = !game_settings.versus_ai;
                settings_updated = true;
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
                // Update UI text displays
                for mut text in board_size_query.iter_mut() {
                    text.0 = format!("{}x{}", game_settings.board_size, game_settings.board_size);
                }
                
                for mut text in chain_to_win_query.iter_mut() {
                    text.0 = game_settings.minimum_chain_to_win.to_string();
                }
                
                for mut text in capture_to_win_query.iter_mut() {
                    text.0 = game_settings.total_capture_to_win.to_string();
                }
                
                for mut text in game_mode_query.iter_mut() {
                    text.0 = if game_settings.versus_ai { "vs AI".to_string() } else { "Multiplayer".to_string() };
                }
                
                for mut text in ai_depth_query.iter_mut() {
                    text.0 = game_settings.ai_depth.to_string();
                }
                
                for mut text in time_limit_query.iter_mut() {
                    if let Some(time_limit) = game_settings.time_limit {
                        text.0 = time_limit.to_string();
                    } else {
                        text.0 = "Disabled".to_string();
                    }
                }
            }
        }
    }
}