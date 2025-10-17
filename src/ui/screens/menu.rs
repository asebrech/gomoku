
    use bevy::{
        app::AppExit,
        audio::{PlaybackSettings, Volume, AudioSink},
        ecs::relationship::RelatedSpawnerCommands,
        prelude::*,
        window::{WindowMode, MonitorSelection},
    };
    use bevy_gstreamer::camera::BackgroundImageMarker;
    use gstreamer::prelude::*;
    use gstreamer::{Element, State as GstState};
    use gstreamer_app::AppSink;

    use crate::{
        audio::PlayClickSound,
        ui::{
            app::{AppState, GameSettings}, 
            screens::{utils::despawn_screen, splash::PreloadedStones, game::game::preload_game_video_background},
            config::GameConfig,
            theme::{ThemeManager, update_theme_elements},
        }
    };

    #[derive(Component)]
    struct VideoBackground {
        timer: Timer,
        total_frames: usize,
    }

    #[derive(Component)]
    struct GStreamerVideoBackground {
        pub video_path: String,
    }

    #[derive(Component)]
    struct VideoFilePlayer {
        pipeline: Option<Element>,
        app_sink: Option<AppSink>,
        image_handle: Option<Handle<Image>>,
        initialized: bool,
        frame_timer: f32,
        frame_buffer: std::collections::VecDeque<Vec<u8>>,
        video_width: u32,
        video_height: u32,
    }

    impl Default for VideoFilePlayer {
        fn default() -> Self {
            Self {
                pipeline: None,
                app_sink: None,
                image_handle: None,
                initialized: false,
                frame_timer: 0.0,
                frame_buffer: std::collections::VecDeque::with_capacity(3),
                video_width: 0,
                video_height: 0,
            }
        }
    }

    impl Drop for VideoFilePlayer {
        fn drop(&mut self) {
            if let Some(ref pipeline) = self.pipeline {
                println!("VideoFilePlayer being dropped - stopping pipeline");
                if let Err(e) = pipeline.set_state(gstreamer::State::Null) {
                    eprintln!("Failed to stop pipeline in Drop: {}", e);
                }
            }
        }
    }

    /// Global resource to track the current frame across all screens
    #[derive(Resource)]
    pub struct GlobalVideoBackgroundState {
        pub current_frame: usize,
    }

    impl Default for GlobalVideoBackgroundState {
        fn default() -> Self {
            Self { current_frame: 0 }
        }
    }

    #[derive(Resource)]
    pub struct VideoFrames {
        pub frames: Vec<Handle<Image>>,
        #[allow(dead_code)]
        all_loaded: bool,
    }

    #[derive(Resource)]
    pub struct GameBackgroundFrames {
        pub frames: Vec<Handle<Image>>,
        #[allow(dead_code)]
        all_loaded: bool,
    }

    #[derive(Resource)]
    struct PreloadedAssets {
        logo: Handle<Image>,
    }

    #[derive(Component)]
    pub struct PersistentVideoBackground;

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
        video_ready: bool,
    }

    impl Default for LoadingProgress {
        fn default() -> Self {
            Self {
                total_assets: 0,
                loaded_assets: 0,
                loading_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                video_ready: false,
            }
        }
    }

    #[derive(Resource)]
    pub struct GameAudio {
        pub background_music: Handle<AudioSource>,
        pub music_entity: Option<Entity>,
        pub music_sink: Option<Entity>,
    }

    #[derive(Resource, Default)]
    pub struct MenuInitialized {
        pub audio_started: bool,
        pub first_time: bool,
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

    #[derive(Component, Clone, Debug)]
    enum SettingControl {
        BoardSizeInc,
        BoardSizeDec,
        WinConditionInc,
        WinConditionDec,
        AIMaxDepthInc,
        AIMaxDepthDec,
        AITimeLimitInc,
        AITimeLimitDec,
        PairCapturesInc,
        PairCapturesDec,
        Fullscreen,
        #[allow(dead_code)]
        VolumeControl,
        AudioMute,
        ThemePrev,
        ThemeNext,
    }

    #[derive(Component)]
    struct SettingDisplay {
        setting_type: SettingDisplayType,
    }

    #[derive(Debug, Clone)]
    enum SettingDisplayType {
        BoardSizeValue,
        WinConditionValue,
        PairCapturesValue,
        FullscreenToggle,
        MutedToggle,
        AIMaxDepthValue,
        AITimeLimitValue,
        ThemeValue,
    }

    pub fn menu_plugin(app: &mut App) {
        app
            .init_state::<MenuState>()
            .init_resource::<MenuInitialized>()
            .init_resource::<LoadingProgress>()
            .init_resource::<GlobalVideoBackgroundState>()
            .add_systems(OnEnter(AppState::Splash), preload_menu_video_background)
            .add_systems(OnEnter(AppState::Menu), (init_dev_mode_resources, menu_setup, setup_persistent_video_background, preload_game_video_background).chain())
            .add_systems(OnEnter(AppState::HowToPlay), setup_persistent_video_background)
            .add_systems(OnEnter(MenuState::Splash), splash_screen_setup)
            .add_systems(OnEnter(MenuState::Main), (main_menu_setup, setup_audio_if_needed))
            .add_systems(OnEnter(MenuState::Settings), (settings_menu_setup, force_settings_display_update))
            .add_systems(OnEnter(MenuState::GameModeSelect), game_mode_select_setup)
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
                OnExit(MenuState::GameModeSelect),
                despawn_screen::<OnGameModeSelectScreen>,
            )
            .add_systems(OnEnter(AppState::GameOptions), cleanup_persistent_video_background)
            .add_systems(OnEnter(AppState::Credit), cleanup_persistent_video_background)
            .add_systems(OnEnter(AppState::Game), hide_persistent_video_in_game)
            .add_systems(OnEnter(AppState::Menu), show_persistent_video_background)
            .add_systems(OnEnter(AppState::HowToPlay), show_persistent_video_background)
            .add_systems(
                Update,
                (
                    loading_progress_system,
                    check_video_readiness,
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
                    update_volume_display,
                    handle_settings_controls,
                ).run_if(in_state(AppState::Menu).and(not(in_state(MenuState::Splash)))),
            )
            .add_systems(
                Update,
                (
                    initialize_video_players,
                    update_video_players,
                    handle_video_looping,
                ).run_if(in_state(AppState::Menu).or(in_state(AppState::HowToPlay)).or(in_state(AppState::Splash))),
            )
            .add_systems(
                Update,
                hide_persistent_video_in_game.run_if(in_state(AppState::Game)),
            )
            .add_systems(
                Update,
                (
                    update_settings_display,
                    update_theme_elements,
                    rebuild_settings_on_config_change,
                    handle_escape_key,
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
        GameModeSelect,
        Disabled,
    }

    #[derive(Component)]
    struct OnMainMenuScreen;

    #[derive(Component)]
    struct OnSplashScreen;



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
    struct OnGameModeSelectScreen;

    #[derive(Component)]
    struct SelectedOption;

    #[derive(Component)]
    enum MenuButtonAction {
        #[allow(dead_code)]
		Load,
        Play,
        PlayVsAI,
        Play1v1,
        HowToPlay,
        Settings,
        #[allow(dead_code)]
        SettingsSound,
        BackToMainMenu,
        Quit,
    }

    fn button_system(
        mut interaction_query: Query<
            (&Interaction, &mut BackgroundColor, &mut BorderColor, Option<&SelectedOption>, Option<&SettingControl>, Option<&SettingDisplay>),
            (Changed<Interaction>, With<Button>),
        >,
        config: Res<GameConfig>,
    ) {
        let colors = &config.colors;
        
        for (interaction, mut background_color, mut border_color, selected, setting_control, _setting_display) in &mut interaction_query {
            // Handle settings controls with special hover behavior
            if let Some(setting_control) = setting_control {
                match *interaction {
                    Interaction::Hovered => {
                        // For settings buttons, slightly brighten on hover but don't override the base color
                        let current_color = background_color.0;
                        let rgba = current_color.to_srgba();
                        let brightened = Color::srgba(
                            (rgba.red + 0.1).min(1.0),
                            (rgba.green + 0.1).min(1.0),
                            (rgba.blue + 0.1).min(1.0),
                            rgba.alpha
                        );
                        *background_color = BackgroundColor(brightened);
                        *border_color = BorderColor(colors.accent.clone().into());
                    },
                    Interaction::Pressed => {
                        // On press, slightly darken
                        let current_color = background_color.0;
                        let rgba = current_color.to_srgba();
                        let darkened = Color::srgba(
                            (rgba.red - 0.1).max(0.0),
                            (rgba.green - 0.1).max(0.0),
                            (rgba.blue - 0.1).max(0.0),
                            rgba.alpha
                        );
                        *background_color = BackgroundColor(darkened);
                        *border_color = BorderColor(colors.accent.clone().into());
                    },
                    Interaction::None => {
                        // Restore the proper base color based on setting state
                        let base_color = match setting_control {
                            SettingControl::Fullscreen => {
                                let (fullscreen, _) = config.get_display_settings();
                                if fullscreen { colors.button_pressed.clone() } else { colors.button_normal.clone() }
                            },
                            SettingControl::AudioMute => {
                                let (_, muted) = config.get_audio_settings();
                                if muted { colors.button_pressed.clone() } else { colors.button_normal.clone() }
                            },
                            _ => colors.button_normal.clone(),
                        };
                        *background_color = BackgroundColor(base_color.into());
                        *border_color = BorderColor(colors.secondary.clone().into());
                    }
                }
                continue;
            }
            
            // Handle regular menu buttons
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

    fn init_dev_mode_resources(
        mut commands: Commands,
        config: Res<GameConfig>,
        asset_server: Res<AssetServer>,
    ) {
        if config.dev_mode {
            let pink_stone = asset_server.load("icons/synthwave/pink-stone.png");
            let blue_stone = asset_server.load("icons/synthwave/blue-stone.png");
            
            commands.insert_resource(PreloadedStones {
                pink_stone,
                blue_stone,
            });
            
            commands.insert_resource(VideoFrames { 
                frames: Vec::new(),
                all_loaded: true,
            });
            commands.insert_resource(PreloadedAssets {
                logo: Handle::default(),
            });
            commands.insert_resource(TrackedAssets::new());
        }
    }

    fn menu_setup(
        mut menu_state: ResMut<NextState<MenuState>>,
        mut menu_initialized: ResMut<MenuInitialized>,
        config: Res<GameConfig>,
    ) {
        if !menu_initialized.first_time {
            if config.dev_mode {
                menu_state.set(MenuState::Main);
            } else {
                menu_state.set(MenuState::Splash);
            }
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
        
        // Check if devMode is enabled
        if config.dev_mode {
            // Create minimal tracked assets for compatibility
            let tracked_assets = TrackedAssets::new();
            commands.insert_resource(tracked_assets);
            
            // Create empty video frames resource
            commands.insert_resource(VideoFrames { 
                frames: Vec::new(),
                all_loaded: true,
            });
            
            // Create dummy preloaded assets
            commands.insert_resource(PreloadedAssets {
                logo: Handle::default(),
            });
            
            // Mark loading as complete
            loading_progress.total_assets = 1;
            loading_progress.loaded_assets = 1;
            
            // Skip to menu immediately by not creating the splash screen UI
            return;
        }
        
        // PRIORITY LOAD: Load splash background first with high priority
        let splash_bg = asset_server.load(&config.assets.backgrounds.splash);
        
        // Create splash screen UI immediately with placeholder
        let _splash_entity = commands
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
                BackgroundColor(colors.background.clone().into()), // Show themed background immediately
                OnSplashScreen,
            ))
            .with_children(|parent| {
                // Splash background image (will appear as soon as it loads)
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(0.0),
                        left: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    ImageNode::new(splash_bg.clone()),
                    SplashBackground, // Tag for updating when loaded
                ));


            }).id();
        
        // Now create tracked assets system for remaining assets
        let mut tracked_assets = TrackedAssets::new();
        
        // Add splash background to tracking (already loading)
        tracked_assets.add_image(splash_bg);
        
        // Load logo for main menu
        let logo_handle = asset_server.load(&config.assets.icons.logo);
        tracked_assets.add_image(logo_handle.clone());
        
        // Load audio
        let background_music: Handle<AudioSource> = asset_server.load(&config.assets.sounds.menu_theme);
        tracked_assets.add_audio(background_music);
        
        // Skip loading video frames since we're using GStreamer for video playback
        let video_frames = Vec::new();
        
        // Skip loading game background frames since we're using GStreamer for video playback
        let game_bg_frames = Vec::new();
        
        // Store resources
        commands.insert_resource(VideoFrames { 
            frames: video_frames,
            all_loaded: false,
        });
        commands.insert_resource(GameBackgroundFrames {
            frames: game_bg_frames,
            all_loaded: false,
        });
        commands.insert_resource(PreloadedAssets {
            logo: logo_handle,
        });
        
        // Set total assets count from tracked system
        let total_count = tracked_assets.total_count;
        loading_progress.total_assets = total_count;
        commands.insert_resource(tracked_assets);
        
        println!("[LOADING] Priority loading: Splash background loading first, {} total assets to follow", total_count);
    }

    fn loading_progress_system(
        asset_server: Res<AssetServer>,
        mut loading_progress: ResMut<LoadingProgress>,
        tracked_assets: Option<Res<TrackedAssets>>,
        mut commands: Commands,
        time: Res<Time>,
        fade_query: Query<Entity, With<FadeTransition>>,
        config: Res<GameConfig>,
    ) {
        loading_progress.loading_timer.tick(time.delta());
        
        if loading_progress.loading_timer.just_finished() {
            if config.dev_mode && fade_query.is_empty() {
                start_fade_transition(&mut commands);
                return;
            }
            
            if let Some(tracked) = tracked_assets.as_ref() {
                // Use the tracked assets system
                let loaded_count = tracked.count_loaded(&asset_server);
                let total_count = tracked.total_count;
                
                loading_progress.loaded_assets = loaded_count;
                
                // Update total assets if not set
                if loading_progress.total_assets == 0 {
                    loading_progress.total_assets = total_count;
                }
                
                // Start fade transition when loading is complete AND video is ready
                if loaded_count >= total_count && total_count > 0 && loading_progress.video_ready && fade_query.is_empty() {
                    println!("Loading complete! Starting beautiful fade transition...");
                    println!("Loaded {} out of {} assets", loaded_count, total_count);
                    start_fade_transition(&mut commands);
                }
            }
        }
    }

    fn check_video_readiness(
        mut loading_progress: ResMut<LoadingProgress>,
        video_players: Query<&VideoFilePlayer, With<PersistentVideoBackground>>,
        config: Res<GameConfig>,
    ) {
        // In dev mode, video is always "ready" (no video needed)
        if config.dev_mode {
            loading_progress.video_ready = true;
            return;
        }
        
        // Check if any persistent video background is ready and streaming
        for player in video_players.iter() {
            if player.initialized && !player.frame_buffer.is_empty() {
                if !loading_progress.video_ready {
                    println!("Video streaming started with buffered frames!");
                    loading_progress.video_ready = true;
                }
                return;
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
                    println!("[TRANSITION] Perfectly smooth fade complete! Transitioning to main menu...");
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

    #[derive(Component)]
    struct SplashBackground;

fn main_menu_setup(
    mut commands: Commands, 
    config: Res<GameConfig>, 
    asset_server: Res<AssetServer>, 
    video_frames: Option<Res<VideoFrames>>,
    _global_bg_state: Res<GlobalVideoBackgroundState>,
    preloaded_assets: Option<Res<PreloadedAssets>>,
) {
    println!("Setting up main menu");
    let colors = &config.colors;
    
    // Skip loading old video frames since we're using GStreamer now
    let _video_frame_handles: Vec<Handle<Image>> = Vec::new();
    
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

    // Store empty frames since we're using GStreamer for video playback
    if video_frames.is_none() {
        commands.insert_resource(VideoFrames { 
            frames: Vec::new(),
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
            // Video background is now handled at the menu level, not per-screen

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
                    PlayClickSound,
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
                let (volume, muted) = config.get_audio_settings();
                let volume_text = if muted {
                    "MUTED".to_string()
                } else {
                    format!("{}%", (volume * 100.0) as u32)
                };
                parent.spawn((
                    Text::new(volume_text),
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
                    PlayClickSound,
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
                    PlayClickSound,
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
                            // Logo (skip in devMode)
                            if !config.dev_mode {
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
                            } else {
                                // In devMode, show text instead of logo
                                parent.spawn((
                                    Text::new("GOMOKU [DEV MODE]"),
                                    TextFont {
                                        font_size: config.ui.font_sizes.title * 1.5,
                                        ..default()
                                    },
                                    TextColor(colors.primary.clone().into()),
                                    Node {
                                        margin: UiRect::bottom(Val::Px(20.0)),
                                        ..default()
                                    },
                                ));
                            }
                            
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
                            // New Game button (primary)
                            create_menu_button(
                                parent,
                                "NEW GAME",
                                MenuButtonAction::Play,
                                button_node.clone(),
                                button_text_font.clone(),
                                colors,
                                true, // is_primary
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
                                Text::new("Press ESC to quit - Made with Rust"),
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

fn settings_menu_setup(
    mut commands: Commands,
    config: Res<GameConfig>,
    asset_server: Res<AssetServer>,
    video_frames: Option<Res<VideoFrames>>,
    global_bg_state: Res<GlobalVideoBackgroundState>,
) {
    settings_menu_setup_internal(&mut commands, &config, &asset_server, video_frames.as_deref(), &global_bg_state);
}

fn settings_menu_setup_internal(
    commands: &mut Commands,
    config: &GameConfig,
    asset_server: &AssetServer,
    video_frames: Option<&VideoFrames>,
    global_bg_state: &GlobalVideoBackgroundState,
) {
    let colors = &config.colors;
    
    // Get the already loaded video frames for background (or empty if devMode)
    let video_frame_handles = if config.dev_mode {
        Vec::new()
    } else if let Some(frames) = video_frames {
        frames.frames.clone()
    } else {
        // Fallback: load frames if somehow not available
        let mut frames = Vec::new();
        for i in 1..=config.assets.animations.main_menu_frames.frame_count {
            let frame_path = config.get_animation_frame_path(i);
            frames.push(asset_server.load(frame_path));
        }
        frames
    };

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
            OnSettingsMenuScreen,
        ))
        .with_children(|parent| {
            // Video background using frame sequence (skip in devMode)
            if !config.dev_mode && !video_frame_handles.is_empty() {
                // Use current global frame to maintain continuity
                let current_frame = global_bg_state.current_frame.min(video_frame_handles.len() - 1);
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(0.0),
                        left: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    ImageNode::new(video_frame_handles[current_frame].clone()),
                    VideoBackground {
                        timer: Timer::from_seconds(1.0 / 15.0, TimerMode::Repeating),
                        total_frames: 120,
                    },
                ));
            } else if config.dev_mode {
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(0.0),
                        left: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(colors.background.clone().into()),
                ));
            }

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
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            ));

            // Main settings container
            parent
                .spawn((
                    Node {
                        width: Val::Px(800.0),
                        height: Val::Auto,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::FlexStart,
                        padding: UiRect::all(Val::Px(40.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
                    BorderColor(colors.accent.clone().into()),
                ))
                .with_children(|parent| {
                    // Title
                    parent.spawn((
                        Text::new("SETTINGS"),
                        TextFont {
                            font_size: config.ui.font_sizes.title,
                            ..default()
                        },
                        TextColor(colors.accent.clone().into()),
                        Node {
                            margin: UiRect::bottom(Val::Px(30.0)),
                            ..default()
                        },
                    ));

                    // Settings tabs container
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Auto,
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::FlexStart,
                                justify_content: JustifyContent::SpaceBetween,
                                column_gap: Val::Px(40.0),
                                ..default()
                            },
                        ))
                        .with_children(|parent| {
                            // Audio Settings Column
                            create_settings_column(
                                parent,
                                "AUDIO",
                                &[
                                    ("Volume", SettingType::VolumeSlider),
                                    ("Muted", SettingType::AudioMute),
                                ],
                                &config,
                            );

                            // Game Settings Column
                            create_settings_column(
                                parent,
                                "GAME",
                                &[
                                    ("Board Size", SettingType::BoardSize),
                                    // ("Win Condition", SettingType::WinCondition),
                                    // ("Pair Captures", SettingType::PairCaptures),
                                    ("AI Depth Limit", SettingType::AIMaxDepth),
                                    ("AI Time Limit", SettingType::AITimeLimit),
                                ],
                                &config,
                            );

                            // Display Settings Column
                            create_settings_column(
                                parent,
                                "DISPLAY",
                                &[
                                    ("Fullscreen", SettingType::Fullscreen),
                                    ("Theme", SettingType::Theme),
                                ],
                                &config,
                            );
                        });

                    // Bottom buttons
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                margin: UiRect::top(Val::Px(40.0)),
                                column_gap: Val::Px(20.0),
                                ..default()
                            },
                        ))
                        .with_children(|parent| {
                            // Back to Main Menu button
                            parent
                                .spawn((
                                    Button,
                                    PlayClickSound,
                                    Node {
                                        width: Val::Px(250.0),
                                        height: Val::Px(50.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border: UiRect::all(Val::Px(2.0)),
                                        padding: UiRect::all(Val::Px(10.0)),
                                        ..default()
                                    },
                                    BackgroundColor(colors.button_normal.clone().into()),
                                    BorderColor(colors.secondary.clone().into()),
                                    MenuButtonAction::BackToMainMenu,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("BACK TO MENU"),
                                        TextFont {
                                            font_size: config.ui.font_sizes.button,
                                            ..default()
                                        },
                                        TextColor(colors.text_primary.clone().into()),
                                        TextLayout {
                                            justify: JustifyText::Center,
                                            linebreak: LineBreak::NoWrap,
                                        },
                                    ));
                                });
                        });
                });
        });
}

fn game_mode_select_setup(
    mut commands: Commands,
    config: Res<GameConfig>,
    asset_server: Res<AssetServer>,
    video_frames: Option<Res<VideoFrames>>,
    global_bg_state: Res<GlobalVideoBackgroundState>,
) {
    let colors = &config.colors;
    
    // Get the already loaded video frames for background (or empty if devMode)
    let video_frame_handles = if config.dev_mode {
        Vec::new()
    } else if let Some(frames) = video_frames.as_ref() {
        frames.frames.clone()
    } else {
        // Fallback: load frames if somehow not available
        let mut frames = Vec::new();
        for i in 1..=config.assets.animations.main_menu_frames.frame_count {
            let frame_path = config.get_animation_frame_path(i);
            frames.push(asset_server.load(frame_path));
        }
        frames
    };

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
            OnGameModeSelectScreen,
        ))
        .with_children(|parent| {
            // Video background using frame sequence (skip in devMode)
            if !config.dev_mode && !video_frame_handles.is_empty() {
                // Use current global frame to maintain continuity
                let current_frame = global_bg_state.current_frame.min(video_frame_handles.len() - 1);
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(0.0),
                        left: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    ImageNode::new(video_frame_handles[current_frame].clone()),
                    VideoBackground {
                        timer: Timer::from_seconds(1.0 / 15.0, TimerMode::Repeating),
                        total_frames: 120,
                    },
                ));
            } else if config.dev_mode {
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(0.0),
                        left: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(colors.background.clone().into()),
                ));
            }

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
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            ));

            // Main container
            parent
                .spawn((
                    Node {
                        width: Val::Px(600.0),
                        height: Val::Auto,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::all(Val::Px(40.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
                    BorderColor(colors.accent.clone().into()),
                ))
                .with_children(|parent| {
                    // Title
                    parent.spawn((
                        Text::new("SELECT GAME MODE"),
                        TextFont {
                            font_size: config.ui.font_sizes.title,
                            ..default()
                        },
                        TextColor(colors.accent.clone().into()),
                        Node {
                            margin: UiRect::bottom(Val::Px(40.0)),
                            ..default()
                        },
                    ));

                    // Button container
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                row_gap: Val::Px(20.0),
                                ..default()
                            },
                        ))
                        .with_children(|parent| {
                            // VS AI button (primary)
                            parent
                                .spawn((
                                    Button,
                                    PlayClickSound,
                                    Node {
                                        width: Val::Px(400.0),
                                        height: Val::Px(80.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border: UiRect::all(Val::Px(2.0)),
                                        ..default()
                                    },
                                    BackgroundColor(colors.accent.clone().into()),
                                    BorderColor(colors.accent.clone().into()),
                                    MenuButtonAction::PlayVsAI,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("VS AI"),
                                        TextFont {
                                            font_size: 28.0,
                                            ..default()
                                        },
                                        TextColor(colors.text_primary.clone().into()),
                                    ));
                                });

                            // 1v1 Local button
                            parent
                                .spawn((
                                    Button,
                                    PlayClickSound,
                                    Node {
                                        width: Val::Px(400.0),
                                        height: Val::Px(80.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border: UiRect::all(Val::Px(2.0)),
                                        ..default()
                                    },
                                    BackgroundColor(colors.button_normal.clone().into()),
                                    BorderColor(colors.secondary.clone().into()),
                                    MenuButtonAction::Play1v1,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("1V1 LOCAL"),
                                        TextFont {
                                            font_size: 28.0,
                                            ..default()
                                        },
                                        TextColor(colors.text_primary.clone().into()),
                                    ));
                                });

                            // Back button
                            parent
                                .spawn((
                                    Button,
                                    PlayClickSound,
                                    Node {
                                        width: Val::Px(250.0),
                                        height: Val::Px(50.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border: UiRect::all(Val::Px(2.0)),
                                        margin: UiRect::top(Val::Px(20.0)),
                                        ..default()
                                    },
                                    BackgroundColor(colors.button_normal.clone().into()),
                                    BorderColor(colors.secondary.clone().into()),
                                    MenuButtonAction::BackToMainMenu,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("BACK"),
                                        TextFont {
                                            font_size: config.ui.font_sizes.button,
                                            ..default()
                                        },
                                        TextColor(colors.text_primary.clone().into()),
                                    ));
                                });
                        });
                });
        });
}

#[derive(Debug, Clone)]
enum SettingType {
    VolumeSlider,
    AudioMute,
    BoardSize,
    #[allow(dead_code)]
    WinCondition,
    AIMaxDepth,
    AITimeLimit,
    #[allow(dead_code)]
    PairCaptures,
    Fullscreen,
    Theme,
}

fn create_settings_column(
    parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
    title: &str,
    settings: &[(&str, SettingType)],
    config: &GameConfig,
) {
    let colors = &config.colors;
    
    parent
        .spawn((
            Node {
                width: Val::Px(220.0),
                height: Val::Auto,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                padding: UiRect::all(Val::Px(20.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(
                colors.surface.r * 0.7,
                colors.surface.g * 0.7,
                colors.surface.b * 0.7,
                0.7
            )),
            BorderColor(colors.secondary.clone().into()),
        ))
        .with_children(|parent| {
            // Column title
            parent.spawn((
                Text::new(title),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(colors.text_secondary.clone().into()),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Settings items
            for (label, setting_type) in settings {
                create_setting_item(parent, label, setting_type, config);
            }
        });
}

fn create_setting_item(
    parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
    label: &str,
    setting_type: &SettingType,
    config: &GameConfig,
) {
    let colors = &config.colors;
    
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Auto,
                flex_direction: FlexDirection::Column,
                margin: UiRect::bottom(Val::Px(15.0)),
                ..default()
            },
        ))
        .with_children(|parent| {
            // Label
            parent.spawn((
                Text::new(label.to_string()),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(colors.text_primary.clone().into()),
                Node {
                    margin: UiRect::bottom(Val::Px(5.0)),
                    ..default()
                },
            ));

            // Control based on setting type
            match setting_type {
                SettingType::VolumeSlider => {
                    let (volume, muted) = config.get_audio_settings();
                    create_volume_control(parent, volume, muted, colors);
                }
                SettingType::AudioMute => {
                    let (_, muted) = config.get_audio_settings();
                    create_toggle_control(parent, muted, "Muted", colors);
                }
                SettingType::BoardSize => {
                    let (board_size, _, _, _, _) = config.get_game_settings();
                    create_number_selector(parent, board_size, 5, 20, "BoardSize", colors);
                }
                // Disabled settings - not currently in use
                SettingType::WinCondition | SettingType::PairCaptures => {
                    // Do nothing - these settings are currently disabled
                }
                SettingType::AIMaxDepth => {
                    let (_, _, ai_max_depth, _, _) = config.get_game_settings();
                    create_ai_depth_selector(parent, ai_max_depth, colors);
                }
                SettingType::AITimeLimit => {
                    let (_, _, _, ai_time_limit, _) = config.get_game_settings();
                    create_ai_time_limit_selector(parent, ai_time_limit, colors);
                }
                SettingType::Fullscreen => {
                    let (fullscreen, _) = config.get_display_settings();
                    create_toggle_control(parent, fullscreen, "Fullscreen", colors);
                }
                SettingType::Theme => {
                    let current_theme = config.get_current_theme();
                    create_theme_selector(parent, &current_theme, colors);
                }
            }
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
            PlayClickSound,
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
        mut game_settings: ResMut<GameSettings>,
    ) {
        for (interaction, menu_button_action) in &interaction_query {
            if *interaction == Interaction::Pressed {
                match menu_button_action {
                    MenuButtonAction::Quit => {
                        app_exit_events.write(AppExit::Success);
                    }
                    MenuButtonAction::Play => {
                        menu_state.set(MenuState::GameModeSelect);
                    }
                    MenuButtonAction::PlayVsAI => {
                        game_settings.versus_ai = true;
                        game_state.set(AppState::Game);
                        menu_state.set(MenuState::Disabled);
                    }
                    MenuButtonAction::Play1v1 => {
                        game_settings.versus_ai = false;
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

    fn handle_escape_key(
        keyboard_input: Res<ButtonInput<KeyCode>>,
        menu_state: Res<State<MenuState>>,
        mut next_menu_state: ResMut<NextState<MenuState>>,
        mut app_exit_events: EventWriter<AppExit>,
    ) {
        if keyboard_input.just_pressed(KeyCode::Escape) {
            match menu_state.get() {
                MenuState::Main => {
                    // In main menu, quit the app
                    app_exit_events.write(AppExit::Success);
                }
                MenuState::Settings | MenuState::SettingsDisplay | MenuState::SettingsSound | MenuState::Load | MenuState::GameModeSelect => {
                    // In submenus, go back to main menu
                    next_menu_state.set(MenuState::Main);
                }
                _ => {}
            }
        }
    }

    fn animate_video_background(
        time: Res<Time>,
        mut video_backgrounds: Query<(&mut VideoBackground, &mut ImageNode)>,
        video_frames: Option<Res<VideoFrames>>,
        mut global_state: ResMut<GlobalVideoBackgroundState>,
        config: Res<GameConfig>,
    ) {
        // Skip animation in devMode
        if config.dev_mode {
            return;
        }
        
        if let Some(frames) = video_frames {
            // Only animate if all frames are loaded (should be true after splash screen)
            for (mut video_bg, mut image_node) in video_backgrounds.iter_mut() {
                video_bg.timer.tick(time.delta());
                
                if video_bg.timer.just_finished() {
                    // Update global frame counter
                    global_state.current_frame = (global_state.current_frame + 1) % video_bg.total_frames;
                    
                    if global_state.current_frame < frames.frames.len() {
                        image_node.image = frames.frames[global_state.current_frame].clone();
                    }
                }
            }
        }
    }

    fn setup_audio_if_needed(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        config: Res<GameConfig>,
        mut menu_initialized: ResMut<MenuInitialized>,
        _game_audio: Option<Res<GameAudio>>,
    ) {
        // Skip audio setup in devMode
        if config.dev_mode {
            menu_initialized.audio_started = true;
            return;
        }
        
        // Only start audio if it hasn't been started yet
        if !menu_initialized.audio_started {
            let (volume, _muted) = config.get_audio_settings();
            println!("Starting audio for the first time with volume: {}", volume);
            
            let background_music = asset_server.load(&config.assets.sounds.menu_theme);
            
            // Start playing background music
            let audio_entity = commands.spawn((
                AudioPlayer::new(background_music.clone()),
                PlaybackSettings::LOOP.with_volume(Volume::Linear(volume)),
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

    #[allow(dead_code)]
    fn setup_audio(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        config: Res<GameConfig>,
        game_audio: Option<Res<GameAudio>>,
        audio_sink_query: Query<Entity, With<AudioSink>>,
    ) {
        let (volume, _muted) = config.get_audio_settings();
        println!("Setting up audio with initial volume: {}", volume);
        
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
            PlaybackSettings::LOOP.with_volume(Volume::Linear(volume)),
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
        mut config: ResMut<GameConfig>,
        mut audio_sink_query: Query<&mut AudioSink>,
        game_audio: Option<Res<GameAudio>>,
    ) {
        let mut volume_changed = false;
        let (current_volume, mut muted) = config.get_audio_settings();
        
        // Convert to percentage (0-10) for clean increments
        let mut volume_percent = (current_volume * 10.0).round() as i32;
        
        // Handle volume up button
        for interaction in volume_up_query.iter() {
            if *interaction == Interaction::Pressed {
                let old_percent = volume_percent;
                volume_percent = (volume_percent + 1).min(10);
                muted = false; // Unmute when adjusting volume
                volume_changed = true;
                println!("VOLUME UP: {}% -> {}%", old_percent * 10, volume_percent * 10);
            }
        }
        
        // Handle volume down button
        for interaction in volume_down_query.iter() {
            if *interaction == Interaction::Pressed {
                let old_percent = volume_percent;
                volume_percent = (volume_percent - 1).max(0);
                if volume_percent == 0 {
                    muted = true; // Auto-mute when volume reaches 0
                }
                volume_changed = true;
                println!("VOLUME DOWN: {}% -> {}%", old_percent * 10, volume_percent * 10);
            }
        }
        
        // Handle mute button
        for interaction in volume_mute_query.iter() {
            if *interaction == Interaction::Pressed {
                muted = !muted;
                if !muted && volume_percent == 0 {
                    volume_percent = 5; // Set to 50% when unmuting from 0 volume
                }
                volume_changed = true;
                println!("VOLUME MUTE: muted={}, volume={}%", muted, volume_percent * 10);
            }
        }
        
        // Handle keyboard controls for volume
        if keyboard_input.just_pressed(KeyCode::Equal) || keyboard_input.just_pressed(KeyCode::NumpadAdd) {
            let old_percent = volume_percent;
            volume_percent = (volume_percent + 1).min(10);
            muted = false;
            volume_changed = true;
            println!("KEYBOARD UP: {}% -> {}%", old_percent * 10, volume_percent * 10);
        }
        if keyboard_input.just_pressed(KeyCode::Minus) || keyboard_input.just_pressed(KeyCode::NumpadSubtract) {
            let old_percent = volume_percent;
            volume_percent = (volume_percent - 1).max(0);
            if volume_percent == 0 {
                muted = true;
            }
            volume_changed = true;
            println!("KEYBOARD DOWN: {}% -> {}%", old_percent * 10, volume_percent * 10);
        }
        
        // Apply volume changes and save to config
        if volume_changed {
            // Convert back to float (0.0-1.0) with clean values
            let volume = volume_percent as f32 / 10.0;
            
            // Save to persistent config
            if let Err(e) = config.save_audio_settings(volume, muted) {
                println!("Failed to save audio settings: {}", e);
            } else {
                println!("Saved audio settings: volume={}, muted={}", volume, muted);
            }
            
            // Apply to current audio
            let effective_volume = if muted { 0.0 } else { volume };
            if let Some(audio) = game_audio {
                if let Some(entity) = audio.music_entity {
                    if let Ok(mut sink) = audio_sink_query.get_mut(entity) {
                        sink.set_volume(Volume::Linear(effective_volume));
                        println!("Updated AudioSink volume to: {}", effective_volume);
                    } else {
                        println!("Could not find AudioSink component on entity");
                    }
                }
            }
        }
    }

    fn update_volume_display(
        config: Res<GameConfig>,
        mut volume_display_query: Query<&mut Text, With<VolumeDisplay>>,
    ) {
        if config.is_changed() {
            let (volume, muted) = config.get_audio_settings();
            for mut text in volume_display_query.iter_mut() {
                if muted {
                    text.0 = "MUTED".to_string();
                } else {
                    text.0 = format!("{}%", (volume * 100.0) as u32);
                }
            }
        }
    }

    fn handle_settings_controls(
        settings_query: Query<(&Interaction, &SettingControl), (Changed<Interaction>, With<Button>)>,
        mut config: ResMut<GameConfig>,
        mut theme_manager: ResMut<ThemeManager>,
        mut audio_sink_query: Query<&mut AudioSink>,
        game_audio: Option<Res<GameAudio>>,
        mut windows: Query<&mut bevy::window::Window>,
    ) {
        // Handle all settings controls
        for (interaction, setting_control) in settings_query.iter() {
            if *interaction == Interaction::Pressed {
                match setting_control {
                    SettingControl::BoardSizeInc => {
                        let (current_board_size, win_condition, ai_max_depth, ai_time_limit, pair_captures) = config.get_game_settings();
                        // Increment: 5 -> 6 -> ... -> 20 -> 5 (loop back)
                        let new_board_size = if current_board_size < 20 { current_board_size + 1 } else { 5 };
                        if let Err(e) = config.save_game_settings(new_board_size, win_condition, ai_max_depth, ai_time_limit, pair_captures) {
                            println!("Failed to save board size: {}", e);
                        } else {
                            println!("Board size changed to: {}", new_board_size);
                        }
                    }
                    SettingControl::BoardSizeDec => {
                        let (current_board_size, win_condition, ai_max_depth, ai_time_limit, pair_captures) = config.get_game_settings();
                        // Decrement: 5 -> 20 -> 19 -> ... -> 6 (loop back)
                        let new_board_size = if current_board_size > 5 { current_board_size - 1 } else { 20 };
                        if let Err(e) = config.save_game_settings(new_board_size, win_condition, ai_max_depth, ai_time_limit, pair_captures) {
                            println!("Failed to save board size: {}", e);
                        } else {
                            println!("Board size changed to: {}", new_board_size);
                        }
                    }
                    // Disabled settings - not currently in use
                    SettingControl::WinConditionInc | SettingControl::WinConditionDec => {
                        // Do nothing - WinCondition setting is currently disabled
                    }
                    SettingControl::PairCapturesInc | SettingControl::PairCapturesDec => {
                        // Do nothing - PairCaptures setting is currently disabled
                    }
                    SettingControl::AIMaxDepthInc => {
                        let (board_size, win_condition, current_ai_max_depth, ai_time_limit, pair_captures) = config.get_game_settings();
                        // Increment: 2 -> 3 -> ... -> 100 -> 2 (loop back)
                        let new_ai_max_depth = match current_ai_max_depth {
                            Some(depth) if depth < 100 => Some(depth + 1),
                            _ => Some(2), // 100 or None -> 2
                        };
                        if let Err(e) = config.save_game_settings(board_size, win_condition, new_ai_max_depth, ai_time_limit, pair_captures) {
                            println!("Failed to save AI max depth: {}", e);
                        } else {
                            println!("AI max depth changed to: {:?}", new_ai_max_depth);
                        }
                    }
                    SettingControl::AIMaxDepthDec => {
                        let (board_size, win_condition, current_ai_max_depth, ai_time_limit, pair_captures) = config.get_game_settings();
                        // Decrement: 2 -> 100 -> 99 -> ... -> 3 (loop back)
                        let new_ai_max_depth = match current_ai_max_depth {
                            Some(depth) if depth > 2 => Some(depth - 1),
                            _ => Some(100), // 2 or None -> 100
                        };
                        if let Err(e) = config.save_game_settings(board_size, win_condition, new_ai_max_depth, ai_time_limit, pair_captures) {
                            println!("Failed to save AI max depth: {}", e);
                        } else {
                            println!("AI max depth changed to: {:?}", new_ai_max_depth);
                        }
                    }
                    SettingControl::AITimeLimitInc => {
                        let (board_size, win_condition, ai_max_depth, current_ai_time_limit, pair_captures) = config.get_game_settings();
                        // Increment: 50ms -> 100ms -> ... -> 5000ms -> 50ms (loop back)
                        let new_ai_time_limit = match current_ai_time_limit {
                            Some(limit) if limit < 5000 => Some(limit + 50),
                            _ => Some(50), // 5000ms or None -> 50ms
                        };
                        if let Err(e) = config.save_game_settings(board_size, win_condition, ai_max_depth, new_ai_time_limit, pair_captures) {
                            println!("Failed to save AI time limit: {}", e);
                        } else {
                            println!("AI time limit changed to: {:?}", new_ai_time_limit);
                        }
                    }
                    SettingControl::AITimeLimitDec => {
                        let (board_size, win_condition, ai_max_depth, current_ai_time_limit, pair_captures) = config.get_game_settings();
                        // Decrement: 50ms -> 5000ms -> 4950ms -> ... -> 100ms (loop back)
                        let new_ai_time_limit = match current_ai_time_limit {
                            Some(limit) if limit > 50 => Some(limit - 50),
                            _ => Some(5000), // 50ms or None -> 5000ms
                        };
                        if let Err(e) = config.save_game_settings(board_size, win_condition, ai_max_depth, new_ai_time_limit, pair_captures) {
                            println!("Failed to save AI time limit: {}", e);
                        } else {
                            println!("AI time limit changed to: {:?}", new_ai_time_limit);
                        }
                    }
                    SettingControl::Fullscreen => {
                        let (current_fullscreen, vsync) = config.get_display_settings();
                        let new_fullscreen = !current_fullscreen;
                        if let Err(e) = config.save_display_settings(new_fullscreen, vsync) {
                            println!("Failed to save fullscreen setting: {}", e);
                        } else {
                            println!("Fullscreen changed to: {}", new_fullscreen);
                            // Apply the window mode immediately
                            if let Ok(mut window) = windows.single_mut() {
                                window.mode = if new_fullscreen {
                                    WindowMode::BorderlessFullscreen(MonitorSelection::Current)
                                } else {
                                    WindowMode::Windowed
                                };
                            }
                        }
                    }
                    SettingControl::AudioMute => {
                        let (volume, current_muted) = config.get_audio_settings();
                        let new_muted = !current_muted;
                        if let Err(e) = config.save_audio_settings(volume, new_muted) {
                            println!("Failed to save mute setting: {}", e);
                        } else {
                            println!("Audio muted changed to: {}", new_muted);
                            
                            // Apply the mute change to the audio sink immediately
                            let effective_volume = if new_muted { 0.0 } else { volume };
                            if let Some(audio) = game_audio.as_ref() {
                                if let Some(entity) = audio.music_entity {
                                    if let Ok(mut sink) = audio_sink_query.get_mut(entity) {
                                        sink.set_volume(Volume::Linear(effective_volume));
                                        println!("Updated AudioSink volume to: {} (muted: {})", effective_volume, new_muted);
                                    }
                                }
                            }
                        }
                    }
                    SettingControl::ThemePrev => {
                        // Get list of available themes
                        let themes = theme_manager.get_available_themes();
                        let current_theme = config.get_current_theme();
                        
                        // Find current theme index
                        let current_index = themes.iter().position(|t| t == &current_theme).unwrap_or(0);
                        
                        // Go to previous theme (wrapping around)
                        let new_index = if current_index == 0 { themes.len() - 1 } else { current_index - 1 };
                        let new_theme = themes[new_index].clone();
                        
                        // Set theme in theme manager and save to config
                        if theme_manager.set_theme(&new_theme) {
                            // Sync colors from theme to config
                            config.sync_colors_from_theme(&theme_manager.current_theme.colors);
                            
                            if let Err(e) = config.save_theme(new_theme.clone()) {
                                println!("Failed to save theme: {}", e);
                            } else {
                                println!("Theme changed to: {}", new_theme);
                            }
                        }
                    }
                    SettingControl::ThemeNext => {
                        // Get list of available themes
                        let themes = theme_manager.get_available_themes();
                        let current_theme = config.get_current_theme();
                        
                        // Find current theme index
                        let current_index = themes.iter().position(|t| t == &current_theme).unwrap_or(0);
                        
                        // Go to next theme (wrapping around)
                        let new_index = (current_index + 1) % themes.len();
                        let new_theme = themes[new_index].clone();
                        
                        // Set theme in theme manager and save to config
                        if theme_manager.set_theme(&new_theme) {
                            // Sync colors from theme to config
                            config.sync_colors_from_theme(&theme_manager.current_theme.colors);
                            
                            if let Err(e) = config.save_theme(new_theme.clone()) {
                                println!("Failed to save theme: {}", e);
                            } else {
                                println!("Theme changed to: {}", new_theme);
                            }
                        }
                    }
                    _ => {} // Other controls handled elsewhere
                }
            }
        }
    }

    fn update_settings_display(
        config: Res<GameConfig>,
        mut query_set: ParamSet<(
            Query<(&SettingDisplay, &mut Text), Without<Button>>,
            Query<(&SettingDisplay, &mut BackgroundColor, &Children), With<Button>>,
            Query<&mut Text>,
        )>,
    ) {
        if config.is_changed() {
            update_settings_display_internal(&config, &mut query_set);
        }
    }

    fn force_settings_display_update(
        config: Res<GameConfig>,
        mut query_set: ParamSet<(
            Query<(&SettingDisplay, &mut Text), Without<Button>>,
            Query<(&SettingDisplay, &mut BackgroundColor, &Children), With<Button>>,
            Query<&mut Text>,
        )>,
    ) {
        // Force update settings display when entering settings menu
        update_settings_display_internal(&config, &mut query_set);
    }

    fn update_settings_display_internal(
        config: &GameConfig,
        query_set: &mut ParamSet<(
            Query<(&SettingDisplay, &mut Text), Without<Button>>,
            Query<(&SettingDisplay, &mut BackgroundColor, &Children), With<Button>>,
            Query<&mut Text>,
        )>,
    ) {
            let (board_size, win_condition, ai_max_depth, ai_time_limit, pair_captures) = config.get_game_settings();
            let (fullscreen, _) = config.get_display_settings();
            let (_, muted) = config.get_audio_settings();
            let current_theme = config.get_current_theme();
            let colors = &config.colors;

            // Update value displays (text elements with SettingDisplay)
            for (setting_display, mut text) in query_set.p0().iter_mut() {
                match &setting_display.setting_type {
                    SettingDisplayType::BoardSizeValue => {
                        text.0 = board_size.to_string();
                    }
                    SettingDisplayType::WinConditionValue => {
                        text.0 = win_condition.to_string();
                    }
                    SettingDisplayType::PairCapturesValue => {
                        text.0 = pair_captures.to_string();
                    }
                    SettingDisplayType::AIMaxDepthValue => {
                        text.0 = match ai_max_depth {
                            Some(depth) => depth.to_string(),
                            None => "2".to_string(), // Fallback to 2 if somehow None
                        };
                    }
                    SettingDisplayType::AITimeLimitValue => {
                        text.0 = match ai_time_limit {
                            Some(ms) => format!("{}ms", ms),
                            None => "50ms".to_string(), // Fallback to 50ms if somehow None
                        };
                    }
                    SettingDisplayType::ThemeValue => {
                        text.0 = current_theme.clone();
                    }
                    _ => {}
                }
            }

            // Collect button update data first
            let mut button_updates = Vec::new();
            for (setting_display, mut bg_color, children) in query_set.p1().iter_mut() {
                match &setting_display.setting_type {
                    SettingDisplayType::FullscreenToggle => {
                        button_updates.push((children[0], if fullscreen { "ON" } else { "OFF" }));
                        *bg_color = BackgroundColor(if fullscreen { 
                            colors.accent.clone() 
                        } else { 
                            colors.button_normal.clone() 
                        }.into());
                    }
                    SettingDisplayType::MutedToggle => {
                        button_updates.push((children[0], if muted { "ON" } else { "OFF" }));
                        *bg_color = BackgroundColor(if muted { 
                            colors.accent.clone() 
                        } else { 
                            colors.button_normal.clone() 
                        }.into());
                    }
                    _ => {}
                }
            }

            // Apply text updates to button children
            for (entity, text_value) in button_updates {
                if let Ok(mut text) = query_set.p2().get_mut(entity) {
                    text.0 = text_value.to_string();
                }
            }
        }

    fn rebuild_settings_on_config_change(
        mut commands: Commands,
        config: Res<GameConfig>,
        asset_server: Res<AssetServer>,
        video_frames: Option<Res<VideoFrames>>,
        global_bg_state: Res<GlobalVideoBackgroundState>,
        menu_state: Res<State<MenuState>>,
        settings_screen_query: Query<Entity, With<OnSettingsMenuScreen>>,
    ) {
        // Only rebuild if we're in Settings state and config changed
        if config.is_changed() && !config.is_added() && *menu_state.get() == MenuState::Settings {
            // Despawn current settings screen
            for entity in settings_screen_query.iter() {
                commands.entity(entity).despawn();
            }
            
            // Rebuild settings screen
            settings_menu_setup_internal(&mut commands, &config, &asset_server, video_frames.as_deref(), &global_bg_state);
        }
    }

    fn create_volume_control(
        parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
        volume: f32,
        muted: bool,
        colors: &crate::ui::config::ColorConfig,
    ) {
        parent
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(30.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
            ))
            .with_children(|parent| {
                // Volume down button
                parent.spawn((
                    Button,
                    PlayClickSound,
                    Node {
                        width: Val::Px(25.0),
                        height: Val::Px(25.0),
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
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(colors.text_primary.clone().into()),
                    ));
                });

                // Volume display
                let volume_text = if muted {
                    "MUTED".to_string()
                } else {
                    format!("{}%", (volume * 100.0) as u32)
                };
                parent.spawn((
                    Text::new(volume_text),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(colors.text_secondary.clone().into()),
                    VolumeDisplay,
                    Node {
                        width: Val::Px(60.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ));

                // Volume up button
                parent.spawn((
                    Button,
                    PlayClickSound,
                    Node {
                        width: Val::Px(25.0),
                        height: Val::Px(25.0),
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
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(colors.text_primary.clone().into()),
                    ));
                });
            });
    }

    fn create_toggle_control(
        parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
        enabled: bool,
        setting_name: &str,
        colors: &crate::ui::config::ColorConfig,
    ) {
        parent.spawn((
            Button,
            PlayClickSound,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(30.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(if enabled { colors.accent.clone() } else { colors.button_normal.clone() }.into()),
            BorderColor(colors.secondary.clone().into()),
            match setting_name {
                "Fullscreen" => SettingControl::Fullscreen,
                _ => SettingControl::AudioMute, // Fallback for mute toggle
            },
            SettingDisplay {
                setting_type: match setting_name {
                    "Fullscreen" => SettingDisplayType::FullscreenToggle,
                    "Muted" => SettingDisplayType::MutedToggle,
                    _ => SettingDisplayType::MutedToggle,
                }
            },
        )).with_children(|parent| {
            parent.spawn((
                Text::new(if enabled { "ON" } else { "OFF" }),
                TextFont { font_size: 14.0, ..default() },
                TextColor(colors.text_primary.clone().into()),
            ));
        });
    }

    fn create_number_selector(
        parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
        current_value: u32,
        _min_value: u32,
        _max_value: u32,
        setting_name: &str,
        colors: &crate::ui::config::ColorConfig,
    ) {
        parent
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(30.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
            ))
            .with_children(|parent| {
                // Decrease button
                parent.spawn((
                    Button,
                    PlayClickSound,
                    Node {
                        width: Val::Px(25.0),
                        height: Val::Px(25.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(colors.button_normal.clone().into()),
                    BorderColor(colors.secondary.clone().into()),
                    match setting_name {
                        "BoardSize" => SettingControl::BoardSizeDec,
                        "WinCondition" => SettingControl::WinConditionDec,
                        "PairCaptures" => SettingControl::PairCapturesDec,
                        _ => SettingControl::BoardSizeDec,
                    },
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new("-"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(colors.text_primary.clone().into()),
                    ));
                });

                // Value display
                parent.spawn((
                    Text::new(current_value.to_string()),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(colors.text_secondary.clone().into()),
                    TextLayout {
                        justify: JustifyText::Center,
                        ..default()
                    },
                    Node {
                        width: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    SettingDisplay {
                        setting_type: match setting_name {
                            "BoardSize" => SettingDisplayType::BoardSizeValue,
                            "WinCondition" => SettingDisplayType::WinConditionValue,
                            "PairCaptures" => SettingDisplayType::PairCapturesValue,
                            _ => SettingDisplayType::BoardSizeValue,
                        }
                    },
                ));

                // Increase button
                parent.spawn((
                    Button,
                    PlayClickSound,
                    Node {
                        width: Val::Px(25.0),
                        height: Val::Px(25.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(colors.button_normal.clone().into()),
                    BorderColor(colors.secondary.clone().into()),
                    match setting_name {
                        "BoardSize" => SettingControl::BoardSizeInc,
                        "WinCondition" => SettingControl::WinConditionInc,
                        "PairCaptures" => SettingControl::PairCapturesInc,
                        _ => SettingControl::BoardSizeInc,
                    },
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new("+"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(colors.text_primary.clone().into()),
                    ));
                });
            });
    }

    fn create_ai_depth_selector(
        parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
        current_depth: Option<u32>,
        colors: &crate::ui::config::ColorConfig,
    ) {
        parent
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(30.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
            ))
            .with_children(|parent| {
                // Decrease button
                parent.spawn((
                    Button,
                    PlayClickSound,
                    Node {
                        width: Val::Px(25.0),
                        height: Val::Px(25.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(colors.button_normal.clone().into()),
                    BorderColor(colors.secondary.clone().into()),
                    SettingControl::AIMaxDepthDec,
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new("-"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(colors.text_primary.clone().into()),
                    ));
                });

                // Value display
                let value_text = match current_depth {
                    Some(depth) => depth.to_string(),
                    None => "2".to_string(), // Fallback to 2 if somehow None
                };
                parent.spawn((
                    Text::new(value_text),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(colors.text_secondary.clone().into()),
                    TextLayout {
                        justify: JustifyText::Center,
                        ..default()
                    },
                    Node {
                        width: Val::Px(80.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    SettingDisplay {
                        setting_type: SettingDisplayType::AIMaxDepthValue,
                    },
                ));

                // Increase button
                parent.spawn((
                    Button,
                    PlayClickSound,
                    Node {
                        width: Val::Px(25.0),
                        height: Val::Px(25.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(colors.button_normal.clone().into()),
                    BorderColor(colors.secondary.clone().into()),
                    SettingControl::AIMaxDepthInc,
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new("+"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(colors.text_primary.clone().into()),
                    ));
                });
            });
    }

    fn create_ai_time_limit_selector(
        parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
        current_time_limit: Option<u64>,
        colors: &crate::ui::config::ColorConfig,
    ) {
        parent
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(30.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
            ))
            .with_children(|parent| {
                // Decrease button
                parent.spawn((
                    Button,
                    PlayClickSound,
                    Node {
                        width: Val::Px(25.0),
                        height: Val::Px(25.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(colors.button_normal.clone().into()),
                    BorderColor(colors.secondary.clone().into()),
                    SettingControl::AITimeLimitDec,
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new("-"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(colors.text_primary.clone().into()),
                    ));
                });

                // Value display
                let value_text = match current_time_limit {
                    Some(ms) => format!("{}ms", ms),
                    None => "50ms".to_string(), // Fallback to 50ms if somehow None
                };
                parent.spawn((
                    Text::new(value_text),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(colors.text_secondary.clone().into()),
                    TextLayout {
                        justify: JustifyText::Center,
                        ..default()
                    },
                    Node {
                        width: Val::Px(80.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    SettingDisplay {
                        setting_type: SettingDisplayType::AITimeLimitValue,
                    },
                ));

                // Increase button
                parent.spawn((
                    Button,
                    PlayClickSound,
                    Node {
                        width: Val::Px(25.0),
                        height: Val::Px(25.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(colors.button_normal.clone().into()),
                    BorderColor(colors.secondary.clone().into()),
                    SettingControl::AITimeLimitInc,
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new("+"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(colors.text_primary.clone().into()),
                    ));
                });
            });
    }

    fn create_theme_selector(
        parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
        current_theme: &str,
        colors: &crate::ui::config::ColorConfig,
    ) {
        parent
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(30.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
            ))
            .with_children(|parent| {
                // Previous theme button (<)
                parent.spawn((
                    Button,
                    PlayClickSound,
                    Node {
                        width: Val::Px(25.0),
                        height: Val::Px(25.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(colors.button_normal.clone().into()),
                    BorderColor(colors.secondary.clone().into()),
                    SettingControl::ThemePrev,
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new("<"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(colors.text_primary.clone().into()),
                    ));
                });

                // Current theme display
                parent.spawn((
                    Text::new(current_theme),
                    TextFont { font_size: 13.0, ..default() },
                    TextColor(colors.text_secondary.clone().into()),
                    TextLayout {
                        justify: JustifyText::Center,
                        ..default()
                    },
                    Node {
                        min_width: Val::Px(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    SettingDisplay {
                        setting_type: SettingDisplayType::ThemeValue,
                    },
                ));

                // Next theme button (>)
                parent.spawn((
                    Button,
                    PlayClickSound,
                    Node {
                        width: Val::Px(25.0),
                        height: Val::Px(25.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(colors.button_normal.clone().into()),
                    BorderColor(colors.secondary.clone().into()),
                    SettingControl::ThemeNext,
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new(">"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(colors.text_primary.clone().into()),
                    ));
                });
            });
    }

    fn initialize_video_players(
        mut commands: Commands,
        mut video_players: Query<(Entity, &mut VideoFilePlayer, &GStreamerVideoBackground), Without<BackgroundImageMarker>>,
    ) {
        let player_count = video_players.iter().len();
        if player_count > 0 {
            println!("Found {} video players to initialize", player_count);
        }
        
        for (entity, mut player, video_bg) in video_players.iter_mut() {
            println!("Initializing video player for entity {:?}", entity);
            if !player.initialized {
                if let Err(e) = gstreamer::init() {
                    println!("Failed to initialize GStreamer: {}", e);
                    continue;
                }
                
                // Create file playback pipeline
                let video_file_path = format!("assets/{}", video_bg.video_path);
                println!("Attempting to load video file: {}", video_file_path);
                
                // Use a more robust pipeline with proper frame rate control and scaling
                let pipeline_description = format!(
                    "uridecodebin uri=file://{} ! videoconvert ! videoscale ! videorate ! video/x-raw,format=RGB,framerate=30/1 ! appsink name=appsink sync=true drop=false max-buffers=3",
                    std::path::Path::new(&video_file_path).canonicalize().unwrap_or_else(|_| std::path::PathBuf::from(&video_file_path)).display()
                );
                
                println!("GStreamer pipeline: {}", pipeline_description);
                
                match gstreamer::parse::launch(&pipeline_description) {
                    Ok(pipeline) => {
                        println!("Pipeline created successfully");
                        if let Some(sink_element) = pipeline
                            .clone()
                            .dynamic_cast::<gstreamer::Bin>()
                            .unwrap()
                            .by_name("appsink")
                        {
                            if let Ok(appsink) = sink_element.dynamic_cast::<AppSink>() {
                                println!("AppSink found and configured");
                                
                                player.pipeline = Some(pipeline);
                                player.app_sink = Some(appsink);
                                player.image_handle = None; // Will be created in update system
                                player.initialized = true;
                            
                            // Start playback
                            if let Some(ref pipeline) = player.pipeline {
                                match pipeline.set_state(GstState::Playing) {
                                    Ok(_) => println!("Video playback started successfully"),
                                    Err(e) => println!("Failed to start video playback: {}", e),
                                }
                            }
                            
                            // Add BackgroundImageMarker to enable background rendering
                            commands.entity(entity).insert(BackgroundImageMarker);
                            } else {
                                println!("Failed to cast sink element to AppSink");
                            }
                        } else {
                            println!("AppSink element not found in pipeline");
                        }
                    }
                    Err(e) => {
                        println!("Failed to create GStreamer pipeline: {}", e);
                    }
                }
            }
        }
    }

    fn update_video_players(
        mut video_players: Query<(&mut VideoFilePlayer, &mut ImageNode), With<BackgroundImageMarker>>,
        mut images: ResMut<Assets<Image>>,
        time: Res<Time>,
    ) {
        for (mut player, mut image_node) in video_players.iter_mut() {
            if !player.initialized {
                continue;
            }
            
            // Stream new frames into buffer (but don't overwhelm it)
            if let Some(app_sink) = player.app_sink.as_ref().cloned() {
                // Fill buffer with available frames (max 3 frames)
                while player.frame_buffer.len() < 3 {
                    if let Some(sample) = app_sink.try_pull_sample(gstreamer::ClockTime::from_mseconds(0)) {
                        if let (Some(buffer), Some(caps)) = (sample.buffer(), sample.caps()) {
                            if let Ok(map) = buffer.map_readable() {
                                let data = map.as_slice();
                                let structure = caps.structure(0).unwrap();
                                let width = structure.get::<i32>("width").unwrap() as u32;
                                let height = structure.get::<i32>("height").unwrap() as u32;
                                
                                // Store video dimensions on first frame
                                if player.video_width == 0 {
                                    player.video_width = width;
                                    player.video_height = height;
                                }
                                
                                // Convert RGB to RGBA and buffer it
                                let rgba_data: Vec<u8> = data.chunks(3)
                                    .flat_map(|chunk| {
                                        if chunk.len() == 3 {
                                            [chunk[0], chunk[1], chunk[2], 255u8]
                                        } else {
                                            [0, 0, 0, 255u8]
                                        }
                                    })
                                    .collect();
                                
                                player.frame_buffer.push_back(rgba_data);
                            }
                        }
                    } else {
                        break; // No more frames available
                    }
                }
            }
            
            // OPTIMIZED: Display frames at 30fps for smoother playback
            player.frame_timer += time.delta_secs();
            if player.frame_timer >= 0.033 && !player.frame_buffer.is_empty() { // ~30 FPS
                player.frame_timer = 0.0;
                
                // Get next frame from buffer
                if let Some(rgba_data) = player.frame_buffer.pop_front() {
                    if player.image_handle.is_none() {
                        // Create texture for the first time
                        let bevy_image = Image::new_fill(
                            bevy::render::render_resource::Extent3d {
                                width: player.video_width,
                                height: player.video_height,
                                depth_or_array_layers: 1,
                            },
                            bevy::render::render_resource::TextureDimension::D2,
                            &rgba_data,
                            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                            bevy::asset::RenderAssetUsages::all(),
                        );
                        
                        let image_handle = images.add(bevy_image);
                        player.image_handle = Some(image_handle.clone());
                        image_node.image = image_handle;
                        println!("Video streaming started with buffered frames!");
                    } else {
                        // OPTIMIZED: Reduce frequency of expensive texture operations
                        if let Some(ref handle) = player.image_handle {
                            // Only update texture every few frames to reduce GPU load
                            static MENU_FRAME_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
                            let frame_count = MENU_FRAME_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            
                            // Update every 2nd frame instead of every frame (reduces load by 50%)
                            if frame_count % 2 == 0 {
                                let updated_image = Image::new_fill(
                                    bevy::render::render_resource::Extent3d {
                                        width: player.video_width,
                                        height: player.video_height,
                                        depth_or_array_layers: 1,
                                    },
                                    bevy::render::render_resource::TextureDimension::D2,
                                    &rgba_data,
                                    bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                                    bevy::asset::RenderAssetUsages::all(),
                                );
                                
                                images.insert(handle, updated_image);
                            }
                        }
                    }
                }
            }
        }
    }

    fn handle_video_looping(
        mut video_players: Query<&mut VideoFilePlayer, With<BackgroundImageMarker>>,
    ) {
        use gstreamer::{MessageView, ClockTime};
        
        for player in video_players.iter_mut() {
            if let Some(ref pipeline) = player.pipeline {
                if let Some(bus) = pipeline.bus() {
                    while let Some(message) = bus.timed_pop(ClockTime::from_seconds(0)) {
                        match message.view() {
                            MessageView::Eos(..) => {
                                println!("Video reached end, seeking back to start for loop");
                                // Use seek with proper flags for smooth looping
                                if let Err(e) = pipeline.seek_simple(
                                    gstreamer::SeekFlags::FLUSH | gstreamer::SeekFlags::KEY_UNIT,
                                    ClockTime::from_seconds(0)
                                ) {
                                    println!("Failed to seek to start: {}", e);
                                    // Fallback: restart the pipeline
                                    let _ = pipeline.set_state(GstState::Ready);
                                    let _ = pipeline.set_state(GstState::Playing);
                                }
                            }
                            MessageView::Error(err) => {
                                println!("GStreamer error: {}", err.error());
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    /// Setup persistent video background that stays across all menu screens
    fn setup_persistent_video_background(
        mut commands: Commands,
        config: Res<GameConfig>,
        theme_manager: Res<ThemeManager>,
        existing_bg_query: Query<Entity, With<PersistentVideoBackground>>,
        mut existing_visibility_query: Query<&mut Visibility, With<PersistentVideoBackground>>,
    ) {
        let colors = &theme_manager.current_theme.colors;
        
        // If video background already exists, just make sure it's visible
        if !existing_bg_query.is_empty() {
            println!("Persistent video background already exists, ensuring it's visible");
            for mut visibility in existing_visibility_query.iter_mut() {
                *visibility = Visibility::Visible;
            }
            return;
        }
        
        if config.dev_mode {
            // In devMode, show a simple colored background
            commands.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(colors.background.clone().into()),
                PersistentVideoBackground, // Mark as persistent
            ));
        }
    }

    /// Cleanup persistent video background when leaving the menu system
    fn cleanup_persistent_video_background(
        mut commands: Commands,
        mut video_query: Query<(Entity, &mut VideoFilePlayer), With<PersistentVideoBackground>>,
    ) {
        println!("Cleaning up persistent video background");
        
        for (entity, mut player) in video_query.iter_mut() {
            // Properly stop and dispose of GStreamer pipeline
            if let Some(ref pipeline) = player.pipeline {
                // First set to PAUSED, then to NULL for proper shutdown
                if let Err(e) = pipeline.set_state(gstreamer::State::Paused) {
                    eprintln!("Failed to set pipeline to Paused state: {}", e);
                } else {
                    // Wait for the state change to complete
                    let (result, current_state, _pending_state) = pipeline.state(Some(gstreamer::ClockTime::from_seconds(1)));
                    match (result, current_state) {
                        (Ok(_), gstreamer::State::Paused) => {
                            println!("Menu pipeline paused successfully");
                        }
                        _ => {
                            println!("Warning: Menu pipeline pause may not have completed");
                        }
                    }
                    
                    // Now set to NULL
                    if let Err(e) = pipeline.set_state(gstreamer::State::Null) {
                        eprintln!("Failed to set pipeline to Null state: {}", e);
                    } else {
                        // Wait for the NULL state change to complete
                        let (result, current_state, _pending_state) = pipeline.state(Some(gstreamer::ClockTime::from_seconds(1)));
                        match (result, current_state) {
                            (Ok(_), gstreamer::State::Null) => {
                                println!("Menu GStreamer pipeline stopped successfully");
                            }
                            _ => {
                                println!("Warning: Menu pipeline stop may not have completed");
                            }
                        }
                    }
                }
            }
            
            // Clear the pipeline reference
            player.pipeline = None;
            player.app_sink = None;
            player.initialized = false;
            
            // Despawn the entity
            commands.entity(entity).despawn();
        }
    }

    /// Hide persistent video background when in game state
    fn hide_persistent_video_in_game(
        mut video_query: Query<&mut Visibility, With<PersistentVideoBackground>>,
    ) {
        for mut visibility in video_query.iter_mut() {
            *visibility = Visibility::Hidden;
        }
    }

    /// Show persistent video background when returning to menu states
    fn show_persistent_video_background(
        mut video_query: Query<&mut Visibility, With<PersistentVideoBackground>>,
    ) {
        for mut visibility in video_query.iter_mut() {
            *visibility = Visibility::Visible;
        }
    }

    /// Preload menu video background during splash screen to avoid flash screens
    fn preload_menu_video_background(
        mut commands: Commands,
        config: Res<GameConfig>,
        theme_manager: Res<ThemeManager>,
        existing_bg_query: Query<Entity, With<PersistentVideoBackground>>,
    ) {
        // Skip in devMode or if already exists
        if config.dev_mode || !existing_bg_query.is_empty() {
            return;
        }
        
        let _colors = &theme_manager.current_theme.colors;
        
        println!("Preloading dolphin video background during splash screen");
        // Spawn a hidden persistent GStreamer video background for the menu
        let entity_commands = commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            ZIndex(-10), // Put video behind everything including overlays
            ImageNode::default(), // Will be updated by the video player
            GStreamerVideoBackground {
                video_path: "backgrounds/dolphin/dolphin.webm".to_string(),
            },
            VideoFilePlayer::default(),
            PersistentVideoBackground, // Mark as persistent
            Visibility::Hidden, // Initially hidden during splash
        ));
        
        println!("Preloaded dolphin video background entity: {:?}", entity_commands.id());
    }



