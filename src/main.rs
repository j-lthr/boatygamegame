use bevy::prelude::*;

use bevy::audio::{AddAudioSource, AudioPlugin, Volume};

mod ability;
mod common;
mod enemy;
mod event;
mod fx;
mod init;
mod loot;
mod player;
mod procedural;
mod projectile;
mod rune;
mod state;
mod ui;
mod utils;

use bevy::window::WindowMode;
use state::GameState;


fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AudioPlugin {
                    global_volume: Volume::Linear(1.0).into(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resizable: false,
                        mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                        ..default()
                    }),
                    ..Default::default()
                }),
        )
        .add_plugins((
            player::plugin,
            ui::HealthBarPlugin,
            ui::DamageNumbersPlugin,
            ui::hud::plugin,
            ability::plugin,
            enemy::plugin,
            rune::plugin,
            loot::plugin,
            event::plugin,
            init::plugin,
        ))
        .add_audio_source::<fx::fm::FMSound>()
        .init_state::<state::GameState>()
        .init_resource::<state::GameScore>()
        .add_systems(
            Startup,
            (
                spawn_music,
                ui::hud::score::setup_score_ui,
            ),
        )
        .add_systems(
            Update,
            (
                player::shoot_gun,
                player::handle_movement,
                fx::blood::blood_particle_physics,
                fx::blood::blood_particle_rendering,
                fx::blood::cleanup_blood_particles,
                fx::blood::spawn_blood_explosion,
                common::handle_damage_events,
                common::handle_player_death,
                common::emit_death_events,
                common::handle_npc_death,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (
                player::handle_camera,
                common::handle_inertia,
                projectile::handle_movement,
                projectile::collide,
                projectile::cleanup,
            ),
        )
        .add_systems(
            Update,
            (
                ui::menu::game_over::handle_game_over_input,
                ui::menu::game_over::update_game_over_screen,
            )
                .run_if(in_state(GameState::GameOver)),
        )
        .add_systems(
            OnEnter(GameState::GameOver),
            ui::menu::game_over::setup_game_over_screen,
        )
        .add_systems(
            OnExit(GameState::GameOver),
            ui::menu::game_over::cleanup_game_over_screen,
        )
        .run();
}

/// set up a simple 3D scene
fn spawn_music(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn((
        AudioPlayer(asset_server.load::<AudioSource>("audio/orch_game.wav")),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: Volume::Decibels(-24.0),
            ..default()
        },
    ));
}
