#![allow(dead_code)]

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::ecs::error::{GLOBAL_ERROR_HANDLER, error};
use bevy::prelude::*;

use bevy::audio::{AddAudioSource, AudioPlugin, SpatialScale, Volume};

use avian3d::prelude::*;

use bevy::window::WindowMode;

mod ability;
mod audio;
mod common;
mod enemy;
mod event;
mod fx;
mod init;
mod input;
mod localization;
mod loot;
mod modifiers;
mod player;
mod procedural;
mod projectile;
mod rune;
mod state;
mod ui;
mod utils;

use crate::audio::music::PlayMusicEvent;
use crate::init::GameInit;
use crate::state::GameState;

fn main() {
    GLOBAL_ERROR_HANDLER
        .set(error)
        .expect("failed to set global error handler");
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AudioPlugin {
                    global_volume: Volume::Linear(1.0).into(),
                    default_spatial_scale: SpatialScale::new(1.0),
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
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            player::plugin,
            ui::plugin,
            ability::plugin,
            enemy::plugin,
            rune::plugin,
            loot::plugin,
            event::plugin,
            init::plugin,
            audio::plugin,
            input::plugin,
            localization::plugin,
            common::plugin,
        ))
        .add_audio_source::<fx::fm::FMSound>()
        .init_state::<state::GameState>()
        .init_resource::<state::GameScore>()
        .add_systems(Startup, ui::hud::score::setup_score_ui)
        .add_systems(
            Update,
            (
                player::shoot_gun,
                player::handle_movement,
                fx::blood::handle_particle_physics,
                fx::blood::blood_particle_rendering,
                fx::blood::cleanup_blood_particles,
                //fx::blood::spawn_particles,
                common::handle_damage_events,
                common::handle_player_death,
                common::emit_death_events,
                common::handle_npc_death,
                common::handle_velocity_averaging,
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
        .add_systems(GameInit, play_main_music)
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

fn play_main_music(mut commands: Commands) {
    commands.send_event(PlayMusicEvent {
        track_name: "audio/bg_main.wav",
        mode: bevy::audio::PlaybackMode::Loop,
    });
}
