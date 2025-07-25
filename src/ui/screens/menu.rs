
    use bevy::{
        app::AppExit,
        audio::{PlaybackSettings, Volume, AudioSink},
        ecs::relationship::RelatedSpawnerCommands,
        prelude::*,
    };

    use crate::ui::{
        app::{AppState, GameSettings}, 
        screens::utils::despawn_screen,
        config::GameConfig
    };

    #[derive(Component)]
    struct VideoBackground {
        current_frame: usize,
        timer: Timer,
        total_frames: usize,
    }

    #[derive(Resource)]
    struct VideoFrames {
        frames: Vec<Handle<Image>>,
        all_loaded: bool,
    }

    #[derive(Resource)]
    struct PreloadedAssets {
        logo: Handle<Image>,
    }

    #[derive(Resource)]
    struct TrackedAssets {
        handles: Vec<UntypedHandle>,
        total_count: usize,
    }

    impl TrackedAssets {
        fn new() -> Self {
            Self {
                handles: Vec::new(),
                total_count: 0,
            }
        }

        fn add_image(&mut self, handle: Handle<Image>) {
            self.handles.push(handle.untyped());
            self.total_count += 1;
        }

        fn add_audio(&mut self, handle: Handle<AudioSource>) {
            self.handles.push(handle.untyped());
            self.total_count += 1;
        }

        fn count_loaded(&self, asset_server: &AssetServer) -> usize {
            self.handles.iter()
                .filter(|handle| matches!(
                    asset_server.get_load_state(handle.id()), 
                    Some(bevy::asset::LoadState::Loaded)
                ))
                .count()
        }
    }

    #[derive(Resource)]
    struct LoadingProgress {
        total_assets: usize,
        loaded_assets: usize,
        loading_timer: Timer,
    }

    impl Default for LoadingProgress {
        fn default() -> Self {
            Self {
                total_assets: 0,
                loaded_assets: 0,
                loading_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            }
        }
    }

    #[derive(Resource)]
    pub struct GameAudio {
        pub background_music: Handle<AudioSource>,
        pub music_entity: Option<Entity>,
        pub music_sink: Option<Entity>,
    }

    #[derive(Resource)]
    pub struct AudioSettings {
        pub volume: f32,
    }

    #[derive(Resource, Default)]
    pub struct MenuInitialized {
        pub audio_started: bool,
        pub first_time: bool,
    }

    impl Default for AudioSettings {
        fn default() -> Self {
            Self { volume: 0.5 }
        }
    }

    #[derive(Component)]
    struct SoundBar;

    #[derive(Component)]
    struct VolumeSlider;

    #[derive(Component)]
    struct VolumeUp;

    #[derive(Component)]
    struct VolumeDown;

    #[derive(Component)]
    struct VolumeDisplay;

    pub fn menu_plugin(app: &mut App) {
        app
            .init_state::<MenuState>()
            .init_resource::<AudioSettings>()
            .init_resource::<MenuInitialized>()
            .init_resource::<LoadingProgress>()
            .add_systems(OnEnter(AppState::Menu), menu_setup)
            .add_systems(OnEnter(MenuState::Splash), splash_screen_setup)
            .add_systems(OnEnter(MenuState::Main), (main_menu_setup, setup_audio_if_needed))
            .add_systems(OnExit(MenuState::Splash), despawn_screen::<OnSplashScreen>)
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
                (
                    loading_progress_system,
                    update_loading_bar,
                    fade_transition_system,
                ).run_if(in_state(MenuState::Splash)),
            )
            .add_systems(
                Update,
                (
                    menu_action, 
                    button_system, 
                    animate_video_background, 
                    handle_volume_control, 
                    update_volume_display
                ).run_if(in_state(AppState::Menu).and(not(in_state(MenuState::Splash)))),
            );
    }

    #[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
    enum MenuState {
        #[default]
        Splash,
        Main,
        Settings,
		Load,
        SettingsDisplay,
        SettingsSound,
        Disabled,
    }

    #[derive(Component)]
    struct OnMainMenuScreen;

    #[derive(Component)]
    struct OnSplashScreen;

    #[derive(Component)]
    struct LoadingBar;

    #[derive(Component)]
    struct LoadingText;

    #[derive(Component)]
    struct FadeTransition {
        timer: Timer,
        fade_out: bool, // true = fading out splash, false = fading in main menu
    }

    #[derive(Component)]
    struct OnSettingsMenuScreen;

    #[derive(Component)]
    struct OnDisplaySettingsMenuScreen;

    #[derive(Component)]
    struct OnSoundSettingsMenuScreen;

    #[derive(Component)]
    struct SelectedOption;

    #[derive(Component)]
    enum MenuButtonAction {
		Load,
        Play,
        HowToPlay,
        Settings,
        SettingsSound,
        BackToMainMenu,
        Quit,
    }

    fn button_system(
        mut interaction_query: Query<
            (&Interaction, &mut BackgroundColor, &mut BorderColor, Option<&SelectedOption>),
            (Changed<Interaction>, With<Button>),
        >,
        config: Res<GameConfig>,
    ) {
        let colors = &config.colors;
        
        for (interaction, mut background_color, mut border_color, selected) in &mut interaction_query {
            match (*interaction, selected) {
                (Interaction::Pressed, _) | (Interaction::None, Some(_)) => {
                    *background_color = BackgroundColor(colors.accent.clone().into());
                    *border_color = BorderColor(colors.accent.clone().into());
                },
                (Interaction::Hovered, Some(_)) => {
                    *background_color = BackgroundColor(colors.button_pressed.clone().into());
                    *border_color = BorderColor(colors.secondary.clone().into());
                },
                (Interaction::Hovered, None) => {
                    *background_color = BackgroundColor(colors.button_hovered.clone().into());
                    *border_color = BorderColor(colors.secondary.clone().into());
                },
                (Interaction::None, None) => {
                    *background_color = BackgroundColor(colors.button_normal.clone().into());
                    *border_color = BorderColor(colors.secondary.clone().into());
                },
            }
        }
    }

    fn setting_button<T: Resource + Component + PartialEq + Copy>(
        interaction_query: Query<(&Interaction, &T, Entity), (Changed<Interaction>, With<Button>)>,
        selected_query: Single<(Entity, &mut BackgroundColor), With<SelectedOption>>,
        mut commands: Commands,
        mut setting: ResMut<T>,
        config: Res<GameConfig>,
    ) {
        let (previous_button, mut previous_button_color) = selected_query.into_inner();
        for (interaction, button_setting, entity) in &interaction_query {
            if *interaction == Interaction::Pressed && *setting != *button_setting {
                *previous_button_color = BackgroundColor(config.colors.button_normal.clone().into());
                commands.entity(previous_button).remove::<SelectedOption>();
                commands.entity(entity).insert(SelectedOption);
                *setting = *button_setting;
            }
        }
    }

    fn menu_setup(
        mut menu_state: ResMut<NextState<MenuState>>,
        mut menu_initialized: ResMut<MenuInitialized>,
    ) {
        if !menu_initialized.first_time {
            // First time entering menu - do the splash screen
            menu_state.set(MenuState::Splash);
            menu_initialized.first_time = true;
        } else {
            // Returning to menu - go directly to main menu
            menu_state.set(MenuState::Main);
        }
    }

    fn splash_screen_setup(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        config: Res<GameConfig>,
        mut loading_progress: ResMut<LoadingProgress>,
    ) {
        let colors = &config.colors;
        
        // Create tracked assets system
        let mut tracked_assets = TrackedAssets::new();
        
        // Load splash background (contains logo)
        let splash_bg = asset_server.load(&config.assets.backgrounds.splash);
        tracked_assets.add_image(splash_bg.clone());
        
        // Load logo for main menu
        let logo_handle = asset_server.load(&config.assets.icons.logo);
        tracked_assets.add_image(logo_handle.clone());
        
        // Load audio
        let background_music: Handle<AudioSource> = asset_server.load(&config.assets.sounds.menu_theme);
        tracked_assets.add_audio(background_music);
        
        // Load all video frames
        let mut video_frames = Vec::new();
        let animation_config = &config.assets.animations.main_menu_frames;
        for i in 1..=animation_config.frame_count {
            let frame_path = config.get_animation_frame_path(i);
            let frame_handle = asset_server.load(frame_path);
            tracked_assets.add_image(frame_handle.clone());
            video_frames.push(frame_handle);
        }
        
        // Store resources
        commands.insert_resource(VideoFrames { 
            frames: video_frames,
            all_loaded: false,
        });
        commands.insert_resource(PreloadedAssets {
            logo: logo_handle,
        });
        
        // Set total assets count from tracked system
        loading_progress.total_assets = tracked_assets.total_count;
        commands.insert_resource(tracked_assets);
        
        // Create splash screen UI
        commands
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    position_type: PositionType::Relative,
                    ..default()
                },
                BackgroundColor(Color::NONE),
                OnSplashScreen,
            ))
            .with_children(|parent| {
                // Splash background image (contains the logo)
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(0.0),
                        left: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    ImageNode::new(splash_bg),
                ));

                // Loading UI positioned at bottom center
                parent
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            bottom: Val::Px(80.0),
                            left: Val::Percent(50.0),
                            width: Val::Px(500.0),
                            height: Val::Auto,
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            padding: UiRect::all(Val::Px(30.0)),
                            margin: UiRect::left(Val::Px(-250.0)), // Center the container
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                    ))
                    .with_children(|parent| {
                        // Loading text
                        parent.spawn((
                            Text::new("Initializing..."),
                            TextFont {
                                font_size: config.ui.font_sizes.loading,
                                ..default()
                            },
                            TextColor(colors.text_primary.clone().into()),
                            LoadingText,
                            Node {
                                margin: UiRect::bottom(Val::Px(15.0)),
                                ..default()
                            },
                        ));

                        // Loading bar container
                        parent
                            .spawn((
                                Node {
                                    width: Val::Px(400.0),
                                    height: Val::Px(16.0),
                                    border: UiRect::all(Val::Px(2.0)),
                                    padding: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(colors.surface.clone().into()),
                                BorderColor(colors.accent.clone().into()),
                            ))
                            .with_children(|parent| {
                                // Loading bar fill
                                parent.spawn((
                                    Node {
                                        width: Val::Percent(0.0),
                                        height: Val::Percent(100.0),
                                        ..default()
                                    },
                                    BackgroundColor(colors.accent.clone().into()),
                                    LoadingBar,
                                ));
                            });

                        // Loading percentage
                        parent.spawn((
                            Text::new("0%"),
                            TextFont {
                                font_size: config.ui.font_sizes.percentage,
                                ..default()
                            },
                            TextColor(colors.text_secondary.clone().into()),
                            Node {
                                margin: UiRect::top(Val::Px(8.0)),
                                ..default()
                            },
                        ));
                    });
            });
    }

    fn loading_progress_system(
        asset_server: Res<AssetServer>,
        mut loading_progress: ResMut<LoadingProgress>,
        tracked_assets: Option<Res<TrackedAssets>>,
        mut commands: Commands,
        time: Res<Time>,
        fade_query: Query<Entity, With<FadeTransition>>,
    ) {
        loading_progress.loading_timer.tick(time.delta());
        
        if loading_progress.loading_timer.just_finished() {
            if let Some(tracked) = tracked_assets.as_ref() {
                // Use the tracked assets system
                let loaded_count = tracked.count_loaded(&asset_server);
                let total_count = tracked.total_count;
                
                loading_progress.loaded_assets = loaded_count;
                
                // Update total assets if not set
                if loading_progress.total_assets == 0 {
                    loading_progress.total_assets = total_count;
                }
                
                // Start fade transition when loading is complete
                if loaded_count >= total_count && total_count > 0 && fade_query.is_empty() {
                    println!("Loading complete! Starting beautiful fade transition...");
                    println!("Loaded {} out of {} assets", loaded_count, total_count);
                    start_fade_transition(&mut commands);
                }
            }
        }
    }

    fn update_loading_bar(
        loading_progress: Res<LoadingProgress>,
        mut loading_bar_query: Query<&mut Node, With<LoadingBar>>,
        mut loading_text_query: Query<&mut Text, (With<Text>, Without<LoadingBar>)>,
        config: Res<GameConfig>,
    ) {
        if loading_progress.is_changed() && loading_progress.total_assets > 0 {
            let progress = loading_progress.loaded_assets as f32 / loading_progress.total_assets as f32;
            let percentage = (progress * 100.0) as u32;
            
            // Update loading bar width
            for mut node in loading_bar_query.iter_mut() {
                node.width = Val::Percent(progress * 100.0);
            }
            
            // Custom loading messages from config
            let loading_message = config.get_loading_message(percentage);
            
            // Update loading text with custom messages
            for mut text in loading_text_query.iter_mut() {
                if text.0.contains('%') {
                    text.0 = format!("{}%", percentage);
                } else {
                    text.0 = loading_message.clone();
                }
            }
        }
    }

    fn start_fade_transition(commands: &mut Commands) {
        // Create fade transition controller - start with fade out
        commands.spawn(FadeTransition {
            timer: Timer::from_seconds(1.2, TimerMode::Once), // 1.2 seconds for silky smooth fade
            fade_out: true,
        });
    }

    fn fade_transition_system(
        mut commands: Commands,
        mut fade_query: Query<(Entity, &mut FadeTransition)>,
        mut menu_state: ResMut<NextState<MenuState>>,
        time: Res<Time>,
        splash_screen_query: Query<Entity, With<OnSplashScreen>>,
        mut fade_overlay_query: Query<&mut BackgroundColor, With<FadeOverlay>>,
    ) {
        for (entity, mut fade) in fade_query.iter_mut() {
            fade.timer.tick(time.delta());
            let raw_progress = fade.timer.elapsed_secs() / fade.timer.duration().as_secs_f32();
            
            if fade.fade_out {
                // Create a black overlay that fades in for smooth transition
                if raw_progress < 0.02 && fade_overlay_query.is_empty() {
                    // Add fade overlay to splash screen
                    if let Ok(splash_entity) = splash_screen_query.single() {
                        commands.entity(splash_entity).with_children(|parent| {
                            parent.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    top: Val::Px(0.0),
                                    left: Val::Px(0.0),
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                                FadeOverlay,
                            ));
                        });
                    }
                }
                
                // Apply smooth easing curve for ultra-smooth fade
                let eased_progress = ease_in_out_cubic(raw_progress);
                
                // Update fade overlay opacity with refined progression
                for mut bg_color in fade_overlay_query.iter_mut() {
                    // Perfect curve: starts slow, accelerates smoothly, then slows to completion
                    let alpha = eased_progress.clamp(0.0, 1.0);
                    bg_color.0 = Color::srgba(0.0, 0.0, 0.0, alpha);
                }
                
                // When fade out completes, transition to main menu
                if fade.timer.just_finished() {
                    println!("✨ Perfectly smooth fade complete! Transitioning to main menu...");
                    menu_state.set(MenuState::Main);
                    commands.entity(entity).despawn();
                }
            }
        }
    }
    
    // Smooth easing function for professional fade transitions
    fn ease_in_out_cubic(t: f32) -> f32 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
        }
    }
    
    #[derive(Component)]
    struct FadeOverlay;

fn main_menu_setup(
    mut commands: Commands, 
    config: Res<GameConfig>, 
    asset_server: Res<AssetServer>, 
    video_frames: Option<Res<VideoFrames>>,
    preloaded_assets: Option<Res<PreloadedAssets>>,
) {
    let colors = &config.colors;
    
    // Get the already loaded video frames
    let video_frame_handles = if let Some(frames) = video_frames.as_ref() {
        frames.frames.clone()
    } else {
        // Fallback: load frames if somehow not available
        let mut frames = Vec::new();
        for i in 1..=config.assets.animations.main_menu_frames.frame_count {
            let frame_path = config.get_animation_frame_path(i);
            frames.push(asset_server.load(frame_path));
        }
        commands.insert_resource(VideoFrames { 
            frames: frames.clone(),
            all_loaded: true,
        });
        frames
    };
    
    // Common style for all buttons on the screen
    let button_node = Node {
        width: Val::Px(280.0),
        height: Val::Px(60.0),
        margin: UiRect::all(Val::Px(12.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(Val::Px(2.0)),
        ..default()
    };
    
    let button_text_font = TextFont {
        font_size: config.ui.font_sizes.button,
        ..default()
    };

    // Store frames for the animation system
    if video_frames.is_none() {
        commands.insert_resource(VideoFrames { 
            frames: video_frame_handles.clone(),
            all_loaded: true,
        });
    }

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Relative,
                ..default()
            },
            BackgroundColor(Color::NONE),
            OnMainMenuScreen,
        ))
        .with_children(|parent| {
            // Video background using frame sequence
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ImageNode::new(video_frame_handles[0].clone()),
                VideoBackground {
                    current_frame: 0,
                    timer: Timer::from_seconds(1.0 / 15.0, TimerMode::Repeating), // 15 FPS
                    total_frames: 120,
                },
            ));

            // Dark overlay for better text readability
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.4)),
            ));

            // Sound control bar in top-right corner
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(20.0),
                    right: Val::Px(20.0),
                    width: Val::Px(180.0),
                    height: Val::Px(50.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect::all(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                BorderColor(colors.secondary.clone().into()),
                SoundBar,
            )).with_children(|parent| {
                // Volume down button
                parent.spawn((
                    Button,
                    Node {
                        width: Val::Px(30.0),
                        height: Val::Px(30.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(colors.button_normal.clone().into()),
                    BorderColor(colors.secondary.clone().into()),
                    VolumeDown,
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new("-"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(colors.text_primary.clone().into()),
                    ));
                });

                // Volume display
                parent.spawn((
                    Text::new("50%"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(colors.text_secondary.clone().into()),
                    VolumeDisplay,
                    Node {
                        width: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ));

                // Volume up button
                parent.spawn((
                    Button,
                    Node {
                        width: Val::Px(30.0),
                        height: Val::Px(30.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(colors.button_normal.clone().into()),
                    BorderColor(colors.secondary.clone().into()),
                    VolumeUp,
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new("+"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(colors.text_primary.clone().into()),
                    ));
                });

                // Mute/unmute button
                parent.spawn((
                    Button,
                    Node {
                        width: Val::Px(35.0),
                        height: Val::Px(30.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(colors.button_normal.clone().into()),
                    BorderColor(colors.secondary.clone().into()),
                    VolumeSlider,
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new("MUTE"),
                        TextFont {
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(colors.text_primary.clone().into()),
                    ));
                });
            });            // Main container with glassmorphism effect
            parent
                .spawn((
                    Node {
                        width: Val::Px(500.0),
                        height: Val::Auto,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::all(Val::Px(40.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    BorderColor(Color::NONE),
                ))
                .with_children(|parent| {
                    // Title section
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                margin: UiRect::bottom(Val::Px(40.0)),
                                ..default()
                            },
                        ))
                        .with_children(|parent| {
                            // Logo
                            let logo_handle = if let Some(assets) = preloaded_assets.as_ref() {
                                assets.logo.clone()
                            } else {
                                asset_server.load(&config.assets.icons.logo)
                            };
                            
                            parent.spawn((
                                ImageNode::new(logo_handle),
                                Node {
                                    width: Val::Px(400.0),  // Scale down the logo
                                    height: Val::Auto,      // Maintain aspect ratio
                                    margin: UiRect::bottom(Val::Px(20.0)),
                                    ..default()
                                },
                            ));
                            
                            // Subtitle
                            parent.spawn((
                                Text::new("The Ultimate Strategic Experience"),
                                TextFont {
                                    font_size: config.ui.font_sizes.subtitle,
                                    ..default()
                                },
                                TextColor(colors.text_secondary.clone().into()),
                                Node {
                                    margin: UiRect::top(Val::Px(8.0)),
                                    ..default()
                                },
                            ));
                        });

                    // Button container
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                row_gap: Val::Px(8.0),
                                ..default()
                            },
                        ))
                        .with_children(|parent| {
                            // New Game button (primary) with play icon
                            let play_icon = asset_server.load(&config.assets.icons.synthwave.play);
                            create_menu_button_with_icon(
                                parent,
                                "NEW GAME",
                                MenuButtonAction::Play,
                                button_node.clone(),
                                button_text_font.clone(),
                                colors,
                                true, // is_primary
                                Some(play_icon),
                            );

                            // How to Play button
                            create_menu_button(
                                parent,
                                "HOW TO PLAY",
                                MenuButtonAction::HowToPlay,
                                button_node.clone(),
                                button_text_font.clone(),
                                colors,
                                false,
                            );

                            // Settings button
                            create_menu_button(
                                parent,
                                "SETTINGS",
                                MenuButtonAction::Settings,
                                button_node.clone(),
                                button_text_font.clone(),
                                colors,
                                false,
                            );

                            // Quit button
                            create_menu_button(
                                parent,
                                "QUIT",
                                MenuButtonAction::Quit,
                                button_node.clone(),
                                button_text_font.clone(),
                                colors,
                                false,
                            );
                        });

                    // Footer with credits
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                margin: UiRect::top(Val::Px(30.0)),
                                ..default()
                            },
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Press ESC to quit • Made with ❤️ and Rust"),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(colors.text_secondary.clone().into()),
                            ));
                        });
                });
        });
}

fn create_menu_button(
    parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
    text: &str,
    action: MenuButtonAction,
    button_node: Node,
    button_text_font: TextFont,
    colors: &crate::ui::config::ColorConfig,
    is_primary: bool,
) {
    create_menu_button_with_icon(parent, text, action, button_node, button_text_font, colors, is_primary, None);
}

fn create_menu_button_with_icon(
    parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
    text: &str,
    action: MenuButtonAction,
    button_node: Node,
    button_text_font: TextFont,
    colors: &crate::ui::config::ColorConfig,
    is_primary: bool,
    icon_handle: Option<Handle<Image>>,
) {
    let (bg_color, border_color) = if is_primary {
        (colors.accent.clone(), colors.accent.clone())
    } else {
        (colors.button_normal.clone(), colors.secondary.clone())
    };

    parent
        .spawn((
            Button,
            button_node,
            BackgroundColor(bg_color.into()),
            BorderColor(border_color.into()),
            action,
        ))
        .with_children(|parent| {
            // Create a horizontal container for icon + text
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                },
            )).with_children(|parent| {
                // Add icon if provided
                if let Some(icon) = icon_handle {
                    parent.spawn((
                        ImageNode::new(icon),
                        Node {
                            width: Val::Px(24.0),
                            height: Val::Px(24.0),
                            ..default()
                        },
                    ));
                }
                
                // Add text
                parent.spawn((
                    Text::new(text),
                    button_text_font,
                    TextColor(colors.text_primary.clone().into()),
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
                    MenuButtonAction::HowToPlay => {
                        game_state.set(AppState::HowToPlay);
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

    fn animate_video_background(
        time: Res<Time>,
        mut video_backgrounds: Query<(&mut VideoBackground, &mut ImageNode)>,
        video_frames: Option<Res<VideoFrames>>,
    ) {
        if let Some(frames) = video_frames {
            // Only animate if all frames are loaded (should be true after splash screen)
            for (mut video_bg, mut image_node) in video_backgrounds.iter_mut() {
                video_bg.timer.tick(time.delta());
                
                if video_bg.timer.just_finished() {
                    video_bg.current_frame = (video_bg.current_frame + 1) % video_bg.total_frames;
                    
                    if video_bg.current_frame < frames.frames.len() {
                        image_node.image = frames.frames[video_bg.current_frame].clone();
                    }
                }
            }
        }
    }

    fn setup_audio_if_needed(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        audio_settings: Res<AudioSettings>,
        config: Res<GameConfig>,
        mut menu_initialized: ResMut<MenuInitialized>,
        _game_audio: Option<Res<GameAudio>>,
    ) {
        // Only start audio if it hasn't been started yet
        if !menu_initialized.audio_started {
            println!("Starting audio for the first time with volume: {}", audio_settings.volume);
            
            let background_music = asset_server.load(&config.assets.sounds.menu_theme);
            
            // Start playing background music
            let audio_entity = commands.spawn((
                AudioPlayer::new(background_music.clone()),
                PlaybackSettings::LOOP.with_volume(Volume::Linear(audio_settings.volume)),
            )).id();
            
            println!("Spawned audio entity: {:?}", audio_entity);
            
            let audio = GameAudio {
                background_music,
                music_entity: Some(audio_entity),
                music_sink: Some(audio_entity),
            };
            commands.insert_resource(audio);
            menu_initialized.audio_started = true;
            println!("Audio setup complete - music will continue playing across screens");
        } else {
            println!("Audio already playing - keeping existing music");
        }
    }

    fn setup_audio(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        audio_settings: Res<AudioSettings>,
        config: Res<GameConfig>,
        game_audio: Option<Res<GameAudio>>,
        audio_sink_query: Query<Entity, With<AudioSink>>,
    ) {
        println!("Setting up audio with initial volume: {}", audio_settings.volume);
        
        // Stop and despawn any existing audio entities (safely)
        if let Some(existing_audio) = game_audio {
            if let Some(entity) = existing_audio.music_entity {
                println!("Stopping existing audio entity: {:?}", entity);
                match commands.get_entity(entity) {
                    Ok(mut entity_commands) => {
                        entity_commands.despawn();
                    }
                    Err(_) => {
                        println!("Entity {:?} no longer exists", entity);
                    }
                }
            }
        }
        
        // Also clean up any orphaned AudioSink entities (safely)
        for entity in audio_sink_query.iter() {
            println!("Cleaning up orphaned audio entity: {:?}", entity);
            match commands.get_entity(entity) {
                Ok(mut entity_commands) => {
                    entity_commands.despawn();
                }
                Err(_) => {
                    println!("Entity {:?} no longer exists", entity);
                }
            }
        }
        
        let background_music = asset_server.load(&config.assets.sounds.menu_theme);
        
        // Start playing background music
        let audio_entity = commands.spawn((
            AudioPlayer::new(background_music.clone()),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(audio_settings.volume)),
        )).id();
        
        println!("Spawned new audio entity: {:?}", audio_entity);
        
        let audio = GameAudio {
            background_music,
            music_entity: Some(audio_entity),
            music_sink: Some(audio_entity),
        };
        commands.insert_resource(audio);
        println!("Audio setup complete - checking for AudioSink on entity");
    }

    fn handle_volume_control(
        volume_up_query: Query<&Interaction, (Changed<Interaction>, With<VolumeUp>, With<Button>)>,
        volume_down_query: Query<&Interaction, (Changed<Interaction>, With<VolumeDown>, With<Button>)>,
        volume_mute_query: Query<&Interaction, (Changed<Interaction>, With<VolumeSlider>, With<Button>)>,
        keyboard_input: Res<ButtonInput<KeyCode>>,
        mut audio_settings: ResMut<AudioSettings>,
        mut audio_sink_query: Query<&mut AudioSink>,
        game_audio: Option<Res<GameAudio>>,
    ) {
        let mut volume_changed = false;
        
        // Handle volume up button
        for interaction in volume_up_query.iter() {
            if *interaction == Interaction::Pressed {
                let old_volume = audio_settings.volume;
                audio_settings.volume = (audio_settings.volume + 0.1).min(1.0);
                volume_changed = true;
                println!("VOLUME UP: {} -> {}", old_volume, audio_settings.volume);
            }
        }
        
        // Handle volume down button
        for interaction in volume_down_query.iter() {
            if *interaction == Interaction::Pressed {
                let old_volume = audio_settings.volume;
                audio_settings.volume = (audio_settings.volume - 0.1).max(0.0);
                volume_changed = true;
                println!("VOLUME DOWN: {} -> {}", old_volume, audio_settings.volume);
            }
        }
        
        // Handle mute button
        for interaction in volume_mute_query.iter() {
            if *interaction == Interaction::Pressed {
                let old_volume = audio_settings.volume;
                audio_settings.volume = if audio_settings.volume > 0.0 { 0.0 } else { 0.5 };
                volume_changed = true;
                println!("VOLUME MUTE: {} -> {}", old_volume, audio_settings.volume);
            }
        }
        
        // Handle keyboard controls for volume
        if keyboard_input.just_pressed(KeyCode::Equal) || keyboard_input.just_pressed(KeyCode::NumpadAdd) {
            let old_volume = audio_settings.volume;
            audio_settings.volume = (audio_settings.volume + 0.1).min(1.0);
            volume_changed = true;
            println!("KEYBOARD UP: {} -> {}", old_volume, audio_settings.volume);
        }
        if keyboard_input.just_pressed(KeyCode::Minus) || keyboard_input.just_pressed(KeyCode::NumpadSubtract) {
            let old_volume = audio_settings.volume;
            audio_settings.volume = (audio_settings.volume - 0.1).max(0.0);
            volume_changed = true;
            println!("KEYBOARD DOWN: {} -> {}", old_volume, audio_settings.volume);
        }
        
        // Apply volume changes using AudioSink component
        if volume_changed {
            if let Some(audio) = game_audio {
                if let Some(entity) = audio.music_entity {
                    if let Ok(mut sink) = audio_sink_query.get_mut(entity) {
                        sink.set_volume(Volume::Linear(audio_settings.volume));
                        println!("Updated AudioSink volume to: {}", audio_settings.volume);
                    } else {
                        println!("Could not find AudioSink component on entity");
                    }
                }
            }
        }
    }

    fn update_volume_display(
        audio_settings: Res<AudioSettings>,
        mut volume_display_query: Query<&mut Text, With<VolumeDisplay>>,
    ) {
        if audio_settings.is_changed() {
            for mut text in volume_display_query.iter_mut() {
                text.0 = format!("{}%", (audio_settings.volume * 100.0) as u32);
            }
        }
    }