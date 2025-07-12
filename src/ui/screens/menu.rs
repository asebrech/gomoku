
    use bevy::{
        app::AppExit,
        color::palettes::css::CRIMSON,
        ecs::{relationship::RelatedSpawnerCommands, spawn::SpawnIter},
        prelude::*,
    };

    use crate::ui::{app::{AppState, GameSettings}, screens::utils::despawn_screen};

    // This plugin manages the menu, with 5 different screens:
    // - a main menu with "New Game", "Settings", "Quit"
    // - a settings menu with two submenus and a back button
    // - two settings screen with a setting that can be set and a back button
    pub fn menu_plugin(app: &mut App) {
        app
            // At start, the menu is not enabled. This will be changed in `menu_setup` when
            // entering the `GameState::Menu` state.
            // Current screen in the menu is handled by an independent state from `GameState`
            .init_state::<MenuState>()
            .add_systems(OnEnter(AppState::Menu), menu_setup)
            // Systems to handle the main menu screen
            .add_systems(OnEnter(MenuState::Main), main_menu_setup)
            .add_systems(OnExit(MenuState::Main), despawn_screen::<OnMainMenuScreen>)
            // Systems to handle the settings menu screen
            //.add_systems(OnEnter(MenuState::Settings), settings_menu_setup)
            .add_systems(
                OnExit(MenuState::Settings),
                despawn_screen::<OnSettingsMenuScreen>,
            )
            // Systems to handle the display settings screen
            /*.add_systems(
                OnEnter(MenuState::SettingsDisplay),
                display_settings_menu_setup,
            )*/
           /*.add_systems(
                Update,
                (setting_button::<DisplayQuality>.run_if(in_state(MenuState::SettingsDisplay)),),
            )*/
            .add_systems(
                OnExit(MenuState::SettingsDisplay),
                despawn_screen::<OnDisplaySettingsMenuScreen>,
            )
            // Systems to handle the sound settings screen
            //.add_systems(OnEnter(MenuState::SettingsSound), sound_settings_menu_setup)
            .add_systems(
                Update,
                setting_button::<GameSettings>.run_if(in_state(MenuState::SettingsSound)),
            )
            .add_systems(
                OnExit(MenuState::SettingsSound),
                despawn_screen::<OnSoundSettingsMenuScreen>,
            )
            // Common systems to all screens that handles buttons behavior
            .add_systems(
                Update,
                (menu_action, button_system).run_if(in_state(AppState::Menu)),
            );
    }

    // State used for the current menu screen
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

    // Tag component used to tag entities added on the main menu screen
    #[derive(Component)]
    struct OnMainMenuScreen;

    // Tag component used to tag entities added on the settings menu screen
    #[derive(Component)]
    struct OnSettingsMenuScreen;

    // Tag component used to tag entities added on the display settings menu screen
    #[derive(Component)]
    struct OnDisplaySettingsMenuScreen;

    // Tag component used to tag entities added on the sound settings menu screen
    #[derive(Component)]
    struct OnSoundSettingsMenuScreen;

    const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
    const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
    const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
    const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

    // Tag component used to mark which setting is currently selected
    #[derive(Component)]
    struct SelectedOption;

    // All actions that can be triggered from a button click
    #[derive(Component)]
    enum MenuButtonAction {
		Load,
        Play,
        Settings,
        SettingsSound,
        BackToMainMenu,
        Quit,
    }

    // This system handles changing all buttons color based on mouse interaction
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

    // This system updates the settings when a new value for a setting is selected, and marks
    // the button as the one currently selected
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

		commands.spawn((
			Node {
				width: Val::Percent(100.0),
				height: Val::Percent(100.0),
				align_items: AlignItems::Center,
				row_gap: Val::Px(12.0),
				justify_content: JustifyContent::Center,
				flex_direction: FlexDirection::Column, // Changed to Row to place nodes side by side
				..default()
			},
			OnMainMenuScreen,
		))
		.with_children(|parent| {
			insert_title_node(parent);
			insert_load_play_node(parent, button_node, button_text_font);
		});
    }

	fn insert_title_node(parent: &mut RelatedSpawnerCommands<'_, ChildOf>) {
        parent.spawn((
            Node {
                width: Val::Percent(100.0), // Adjusted to fit two nodes side by side
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(CRIMSON.into()),
        ))
        .with_children(|parent| {
            // Game name
            parent.spawn((
                Text::new("Mecha-Gomoku"),
                TextFont {
                    font_size: 67.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::all(Val::Px(50.0)),
                    ..default()
                },
            ));
        });
	}

	fn insert_load_play_node(parent: &mut RelatedSpawnerCommands<'_, ChildOf>, button_node: Node, button_text_font: TextFont) {
        parent.spawn((
            Node {
                width: Val::Percent(50.0), // Adjusted to fit two nodes side by side
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(CRIMSON.into()),
        ))
        .with_children(|parent| {
            // Test text
            parent.spawn((
                Text::new("test"),
                TextFont {
                    font_size: 67.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::all(Val::Px(50.0)),
                    ..default()
                },
            ));

            // Test buttons
            parent.spawn((
                Button,
                button_node.clone(),
                BackgroundColor(NORMAL_BUTTON),
                MenuButtonAction::Play,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("test New Game"),
                    button_text_font.clone(),
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));
            });

            parent.spawn((
                Button,
                button_node.clone(),
                BackgroundColor(NORMAL_BUTTON),
                MenuButtonAction::Settings,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("test Settings"),
                    button_text_font.clone(),
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));
            });

            parent.spawn((
                Button,
                button_node.clone(),
                BackgroundColor(NORMAL_BUTTON),
                MenuButtonAction::Quit,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("test Quit"),
                    button_text_font.clone(),
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));
            });
        });
	}

	fn settings_credit_quit_buttons() {

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