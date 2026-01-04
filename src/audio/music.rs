use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};

// This component will be attached to an entity to fade the audio in
#[derive(Component)]
struct FadeIn {
    start: f64,
}

// This component will be attached to an entity to fade the audio out
#[derive(Component)]
struct FadeOut {
    start: f64,
}

// Fade effect duration
const FADE_TIME: f64 = 0.5;
const VOLUME: f32 = 0.1;

// Fades in the audio of entities that has the FadeIn component. Removes the FadeIn component once
// full volume is reached.
fn fade_in(
    mut commands: Commands,
    mut audio_sink: Query<(&mut AudioSink, Entity, &FadeIn)>,
    time: Res<Time>,
) {
    for (mut audio, entity, fade_in) in audio_sink.iter_mut() {
        let volume = ((time.elapsed_secs_f64() - fade_in.start) / FADE_TIME) as f32;

        if volume >= 1.0 {
            audio.set_volume(Volume::Linear(VOLUME));
            commands.entity(entity).remove::<FadeIn>();
        } else {
            audio.set_volume(Volume::Linear(volume * VOLUME));
        }
    }
}

// Fades out the audio of entities that has the FadeOut component. Despawns the entities once audio
// volume reaches zero.
fn fade_out(
    mut commands: Commands,
    mut audio_sink: Query<(&mut AudioSink, Entity, &FadeOut), With<FadeOut>>,
    time: Res<Time>,
) {
    for (mut audio, entity, fade_out) in audio_sink.iter_mut() {
        let volume = 1.0 - ((time.elapsed_secs_f64() - fade_out.start) / FADE_TIME) as f32;

        if volume <= 0.0 {
            audio.set_volume(Volume::Linear(0.0));
            commands.entity(entity).despawn();
        } else {
            audio.set_volume(Volume::Linear(VOLUME * volume));
        }
    }
}

#[derive(Event, Debug)]
pub struct PlayMusicEvent {
    pub track_name: &'static str,
    pub mode: PlaybackMode,
}

#[derive(Component)]
pub struct MusicPlayer;

pub fn handle_music_control_events(
    mut commands: Commands,
    query: Query<Entity, With<MusicPlayer>>,
    mut events: EventReader<PlayMusicEvent>,
    assets: ResMut<AssetServer>,
    time: Res<Time>,
) {
    for event in events.read() {
        info!("Received {:?}.", event);

        for mp_entity in query {
            commands.entity(mp_entity).insert(FadeOut {
                start: time.elapsed_secs_f64(),
            });
        }

        commands.spawn((
            AudioPlayer(assets.load::<AudioSource>(event.track_name)),
            PlaybackSettings {
                mode: event.mode,
                volume: Volume::Linear(0.0),
                ..Default::default()
            },
            FadeIn {
                start: time.elapsed_secs_f64(),
            },
            MusicPlayer,
        ));
    }
}

pub fn plugin(app: &mut App) {
    app.add_event::<PlayMusicEvent>();
    app.add_systems(Update, (handle_music_control_events, fade_in, fade_out));
}
