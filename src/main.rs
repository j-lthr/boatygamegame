use bevy::core_pipeline::bloom::Bloom;
use bevy::core_pipeline::motion_blur::MotionBlur;
use bevy::core_pipeline::post_process::ChromaticAberration;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;


use bevy::audio::{AddAudioSource, AudioPlugin, Volume};

mod fx;
mod procedural;
mod ui;
mod enemy;
mod event;
mod player;
mod common;
mod state;
mod projectile;
mod powerup;
mod ability;

use state::GameState;

use crate::ability::dash::Dash;
use crate::ability::shotgun::Shotgun;
use crate::ability::slam::Slam;
use crate::ability::AbilitySlot;
use crate::player::PlayerCamera;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AudioPlugin {
            global_volume: Volume::Linear(2.0).into(),
            ..default()
        }))
        .add_plugins((
            ui::HealthBarPlugin,
            ui::DamageNumbersPlugin,
            ability::AbilityPlugin::<Dash>::new(),
            ability::AbilityPlugin::<Shotgun>::new(),
            ability::AbilityPlugin::<Slam>::new()
        )
        )
        .add_audio_source::<fx::fm::FMSound>()
        .init_state::<state::GameState>()
        .init_resource::<state::GameScore>()
        .insert_resource(enemy::EnemySpawnTimer {
            timer: Timer::from_seconds(2.0, TimerMode::Repeating),
        })
        .add_event::<event::SpawnEvent>()
        .add_event::<event::DamageEvent>()
        .add_systems(Startup, (setup, fx::blood::setup_blood_materials, ui::hud::score::setup_score_ui))
        .add_systems(
            Update,
            (
                player::shoot_gun,
                player::handle_camera,
                player::handle_movement,
                projectile::handle_movement,
                projectile::collide,
                projectile::cleanup,
                enemy::spawn_enemies,
                enemy::move_enemies,
                fx::blood::blood_particle_physics,
                fx::blood::blood_particle_rendering,
                fx::blood::cleanup_blood_particles,
                fx::blood::fade_blood_splatters,
                enemy::enemy_combat_ai,
                ui::hud::score::update_score_display,
                ui::hud::score::update_combo_system,
                common::handle_inertia,
                common::handle_damage_events,
                common::check_player_death,
                common::handle_enemy_deaths
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (ui::menu::game_over::handle_game_over_input, ui::menu::game_over::update_game_over_screen).run_if(in_state(GameState::GameOver)),
        )
        .add_systems(OnEnter(GameState::GameOver), ui::menu::game_over::setup_game_over_screen)
        .add_systems(OnExit(GameState::GameOver), ui::menu::game_over::cleanup_game_over_screen)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: ResMut<AssetServer>,
) {
    // Player spawn point (invisible, camera will follow this)
    let player = commands
        .spawn((
            Transform::from_xyz(0.0, 0.5, 0.0), // Eye level height
            player::Player,
            Mesh3d(meshes.add(Sphere::new(0.5))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(10.0, 10.0, 10.0),
                ..default()
            })),
            AbilitySlot {
                cooldown: Timer::from_seconds(1.0, TimerMode::Once),
                name: "Dash",
                ability: Dash {
                    range: 2.0
                }
            },
            common::Living {
                health: 100,
                max_health: 100,
            },
            common::Inertia {
                prev_pos: Vec3::new(0.0, 0.5, 0.0), // Initial previous position
                damping: 0.1,         // Damping factor for Verlet integration
            },
            AbilitySlot {
                cooldown: Timer::from_seconds(0.5, TimerMode::Once),
                name: "Shotgun",
                ability: Shotgun {
                    bullet_count: 10,
                    spread: 0.5,
                    speed: 30.0,
                    lifetime: 1.0,
                    damage: 1,
                }
            },
        ))
        .id();

    let ground_plane = asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/ground.glb"));

    commands.spawn(SceneRoot(ground_plane));

    commands.send_event(event::SpawnEvent { entity: player });

    // First-person camera
    commands
        .spawn((
            Camera3d::default(),
            Camera {
                hdr: true, // Enable HDR for better lighting
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..default()
            },
            Projection::from(PerspectiveProjection {
                fov: 70.0_f32.to_radians(),
                ..default()
            }),
            Transform::from_xyz(0.0, 10.0, -10.0).looking_at(Vec3::ZERO, Vec3::Y),
            // FirstPersonCamera::default(),
            SpatialListener::default(), // Spatial audio listener
            Tonemapping::TonyMcMapface, // 2. Using a tonemapper that desaturates to white is recommended
            Bloom::ANAMORPHIC,
            MotionBlur {
                shutter_angle: 1.0,
                samples: 2,
            },
            ChromaticAberration::default(),
            PlayerCamera,
        ));

    commands.spawn((
        Camera2d::default(),
        Camera {
            order: 1,
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        },
    ));
}