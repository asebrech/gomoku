use bevy::prelude::*;
use rand::Rng;

/// Audio resources for the game
#[derive(Resource)]
pub struct GameAudio {
    pub ui_click: Handle<AudioSource>,
    pub stone_one: Handle<AudioSource>,
    pub stone_two: Handle<AudioSource>,
    pub game_win: Handle<AudioSource>,
    pub game_lose: Handle<AudioSource>,
}

/// Component to mark entities that should play UI click sounds
#[derive(Component)]
pub struct PlayClickSound;

/// Event to trigger stone placement sound (randomized between stone_one and stone_two)
#[derive(Event)]
pub struct PlayStonePlacementSound;

/// Event to trigger win sound
#[derive(Event)]
pub struct PlayWinSound;

/// Event to trigger lose sound
#[derive(Event)]
pub struct PlayLoseSound;

pub fn audio_plugin(app: &mut App) {
    app.add_event::<PlayStonePlacementSound>()
        .add_event::<PlayWinSound>()
        .add_event::<PlayLoseSound>()
        .add_systems(Startup, load_audio)
        .add_systems(
            Update,
            (
                handle_button_click_sounds,
                handle_stone_placement_sounds,
                handle_win_sounds,
                handle_lose_sounds,
            ),
        );
}

fn load_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    let audio = GameAudio {
        ui_click: asset_server.load("sound/click.mp3"),
        stone_one: asset_server.load("sound/stone_one.wav"),
        stone_two: asset_server.load("sound/stone_two.wav"),
        game_win: asset_server.load("sound/game_win.wav"),
        game_lose: asset_server.load("sound/game_lose.wav"),
    };
    commands.insert_resource(audio);
}

/// System to handle button click sounds
fn handle_button_click_sounds(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<PlayClickSound>)>,
    audio: Res<GameAudio>,
    mut commands: Commands,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            commands.spawn(AudioPlayer::new(audio.ui_click.clone()));
        }
    }
}

/// System to handle stone placement sounds (randomized)
fn handle_stone_placement_sounds(
    mut events: EventReader<PlayStonePlacementSound>,
    audio: Res<GameAudio>,
    mut commands: Commands,
) {
    for _event in events.read() {
        // Randomly choose between stone_one and stone_two
        let mut rng = rand::thread_rng();
        let sound = if rng.gen_bool(0.5) {
            audio.stone_one.clone()
        } else {
            audio.stone_two.clone()
        };
        
        commands.spawn(AudioPlayer::new(sound));
    }
}

/// System to handle win sounds
fn handle_win_sounds(
    mut events: EventReader<PlayWinSound>,
    audio: Res<GameAudio>,
    mut commands: Commands,
) {
    for _event in events.read() {
        commands.spawn(AudioPlayer::new(audio.game_win.clone()));
    }
}

/// System to handle lose sounds
fn handle_lose_sounds(
    mut events: EventReader<PlayLoseSound>,
    audio: Res<GameAudio>,
    mut commands: Commands,
) {
    for _event in events.read() {
        commands.spawn(AudioPlayer::new(audio.game_lose.clone()));
    }
}
