
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
                (menu_action, button_system).run_if(in_state(AppState::Menu)),
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