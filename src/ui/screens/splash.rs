
use bevy::prelude::*;

use crate::ui::{app::AppState, screens::utils::despawn_screen};

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

fn splash_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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