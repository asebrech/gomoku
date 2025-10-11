
use bevy::prelude::*;

use crate::ui::{app::AppState, screens::utils::despawn_screen, config::GameConfig};

// Resource to hold preloaded stone images
#[derive(Resource)]
pub struct PreloadedStones {
    pub pink_stone: Handle<Image>,
    pub blue_stone: Handle<Image>,
}

pub fn splash_plugin(app: &mut App) {
	app
		.add_systems(OnEnter(AppState::Splash), splash_setup)
		.add_systems(Update, countdown.run_if(in_state(AppState::Splash)))
		.add_systems(OnExit(AppState::Splash), despawn_screen::<OnSplashScreen>);
}

#[derive(Component)]
struct OnSplashScreen;

#[derive(Resource, Deref, DerefMut)]
struct SplashTimer(Timer);

fn splash_setup(
	mut commands: Commands, 
	asset_server: Res<AssetServer>,
	config: Res<GameConfig>,
	mut game_state: ResMut<NextState<AppState>>,
) {
	let pink_stone = asset_server.load("icons/synthwave/pink-stone.png");
	let blue_stone = asset_server.load("icons/synthwave/blue-stone.png");
	
	// Insert preloaded stones resource
	commands.insert_resource(PreloadedStones {
		pink_stone,
		blue_stone,
	});
	
	// Check if devMode is enabled - skip the UI but keep the stones
	if config.dev_mode {
		// Skip directly to menu
		game_state.set(AppState::Menu);
		return;
	}
	
	let icon = asset_server.load("le_cat.png");
	
	commands.spawn((
		Node {
			align_items: AlignItems::Center,
			justify_content: JustifyContent::Center,
			width: Val::Percent(100.0),
			height: Val::Percent(100.0),
			..default()
		},
		OnSplashScreen,
		children![(
			ImageNode::new(icon),
			Node {
				..default()
			},
		)],
	));
	commands.insert_resource(SplashTimer(Timer::from_seconds(0.2, TimerMode::Once)));
}

fn countdown(
	mut game_state: ResMut<NextState<AppState>>,
	time: Res<Time>,
	mut timer: ResMut<SplashTimer>,
) {
	if timer.tick(time.delta()).finished() {
		game_state.set(AppState::Menu);
	}
}