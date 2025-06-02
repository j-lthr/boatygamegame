use bevy::core_pipeline::bloom::Bloom;
use bevy::core_pipeline::motion_blur::MotionBlur;
use bevy::core_pipeline::post_process::ChromaticAberration;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::{AccumulatedMouseMotion, MouseMotion};
use bevy::picking::window;
use bevy::prelude::*;
use bevy::render::camera::{self, CameraOutputMode};
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy::winit::cursor;
use procedural::generate_rock_mesh;

use std::f32::consts::PI;
use std::time::Duration;

use bevy::audio::{AddAudioSource, AudioPlugin, Volume};

mod fx;
mod procedural;
mod ui;
mod enemy;
mod event;
mod player;
mod common;
mod state;
mod bullet;
mod powerup;

const FONT_PATH: &str = "fonts/Jersey15-Regular.ttf";


use state::GameState;
use state::GameScore;






// Timer for enemy spawning





#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct ComboText;

#[derive(Component)]
struct GameOverScreen;

#[derive(Component)]
struct FinalScoreText;

#[derive(Component)]
struct Inertia {
    prev_pos: Vec3,
    damping: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AudioPlugin {
            global_volume: Volume::Linear(2.0).into(),
            ..default()
        }))
        .add_plugins(
            ui::HealthBarPlugin
        )
        .add_audio_source::<fx::fm::FMSound>()
        .init_state::<state::GameState>()
        .init_resource::<state::GameScore>()
        .insert_resource(enemy::EnemySpawnTimer {
            timer: Timer::from_seconds(2.0, TimerMode::Repeating),
        })
        .add_event::<event::SpawnEvent>()
        .add_systems(Startup, (setup, fx::blood::setup_blood_materials, setup_ui))
        .add_systems(
            Update,
            (
                player::move_player,
                player::shoot_gun,
                bullet::handle_movement,
                bullet::collide::<enemy::Enemy>,
                bullet::cleanup,
                enemy::spawn_enemies,
                enemy::move_enemies,
                fx::blood::blood_particle_physics,
                fx::blood::blood_particle_rendering,
                fx::blood::cleanup_blood_particles,
                fx::blood::fade_blood_splatters,
                player::camera_follow_player,
                enemy::enemy_slam_attack,
                enemy::animate_fade_effects,
                update_score_display,
                update_combo_system,
                handle_inertia
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (handle_game_over_input, update_game_over_screen).run_if(in_state(GameState::GameOver)),
        )
        .add_systems(OnEnter(GameState::GameOver), setup_game_over_screen)
        .add_systems(OnExit(GameState::GameOver), cleanup_game_over_screen)
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
            player::DashTimer {
                timer: Timer::from_seconds(2.0, TimerMode::Once), // Dash cooldown
            },
            common::Living {
                health: 100,
                max_health: 100,
            },
            Inertia {
                prev_pos: Vec3::new(0.0, 0.5, 0.0), // Initial previous position
                damping: 0.1,         // Damping factor for Verlet integration
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
            player::GunOwner {
                bullet_timer: Timer::from_seconds(1.0, TimerMode::Once), // 2 shots per second
            },
            SpatialListener::default(), // Spatial audio listener
            Tonemapping::TonyMcMapface, // 2. Using a tonemapper that desaturates to white is recommended
            Bloom::ANAMORPHIC,
            MotionBlur {
                shutter_angle: 1.0,
                samples: 2,
            },
            ChromaticAberration::default(),
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

// Setup the score UI
fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Score display
    commands.spawn((
        Text::new("Score: 0"),
        TextFont {
            font: asset_server.load(FONT_PATH),
            font_size: 32.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(20.0),
            ..default()
        },
        ScoreText,
    ));

    // Combo display (initially hidden)
    commands.spawn((
        Text::new(""),
        TextFont {
            font: asset_server.load(FONT_PATH),
            font_size: 28.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.7, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(60.0),
            left: Val::Px(20.0),
            ..default()
        },
        ComboText,
    ));
}

// Update score display system
fn update_score_display(
    mut score_query: Query<&mut Text, (With<ScoreText>, Without<ComboText>)>,
    mut combo_query: Query<&mut Text, (With<ComboText>, Without<ScoreText>)>,
    score: Res<GameScore>,
) {
    // Update main score
    if let Ok(mut score_text) = score_query.single_mut() {
        score_text.0 = format!("{}", score.current);
    }

    // Update combo display
    if let Ok(mut combo_text) = combo_query.single_mut() {
        if score.combo > 1 {
            combo_text.0 = format!("x{}", score.combo);
        } else {
            combo_text.0 = String::new();
        }
    }
}

fn update_combo_system(mut score: ResMut<GameScore>, time: Res<Time>) {
    score.combo_timer.tick(time.delta());

    // Reset combo if timer expires
    if score.combo_timer.finished() && score.combo > 0 {
        score.combo = 0;
    }
}




// Setup game over screen
fn setup_game_over_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    score: Res<GameScore>,
) {
    // Background overlay
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            GameOverScreen,
        ))
        .with_children(|parent| {
            // Game Over title
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font: asset_server.load(FONT_PATH),
                    font_size: 64.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.2, 0.2)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            // Final score
            parent.spawn((
                Text::new(format!("Final Score: {}", score.current)),
                TextFont {
                    font: asset_server.load(FONT_PATH),
                    font_size: 36.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
                FinalScoreText,
            ));

            // Kill count
            parent.spawn((
                Text::new(format!("Enemies Defeated: {}", score.kills)),
                TextFont {
                    font: asset_server.load(FONT_PATH),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // Restart instructions
            parent.spawn((
                Text::new("Press R to Restart | Press ESC to Quit"),
                TextFont {
                    font: asset_server.load(FONT_PATH),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 1.0)),
            ));
        });
}

// Handle input on game over screen
fn handle_game_over_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        next_state.set(GameState::Playing);
    }

    if keyboard_input.just_pressed(KeyCode::Escape) {
        app_exit_events.send(AppExit::Success);
    }
}

// Update game over screen (for any animations or dynamic content)
fn update_game_over_screen(
    mut final_score_query: Query<&mut Text, With<FinalScoreText>>,
    score: Res<GameScore>,
    time: Res<Time>,
) {
    // Add a pulsing effect to the final score
    if let Ok(mut score_text) = final_score_query.single_mut() {
        let pulse = (time.elapsed_secs() * 2.0).sin() * 0.3 + 0.7;
        score_text.0 = format!("Final Score: {}", score.current);
    }
}

// Cleanup game over screen
fn cleanup_game_over_screen(
    mut commands: Commands,
    game_over_query: Query<Entity, With<GameOverScreen>>,
    mut score: ResMut<GameScore>,
    mut enemy_spawn_timer: ResMut<enemy::EnemySpawnTimer>,
    // Reset all game entities
    enemy_query: Query<Entity, With<enemy::Enemy>>,
    bullet_query: Query<Entity, With<bullet::Bullet>>,
    mut player_query: Query<(&mut Transform, &mut common::Living), With<player::Player>>,
) {
    // Remove game over UI
    for entity in &game_over_query {
        commands.entity(entity).despawn_recursive();
    }

    // Reset score
    *score = GameScore::default();

    // Reset enemy spawn timer
    enemy_spawn_timer.timer.reset();

    // Clean up all game entities
    for entity in &enemy_query {
        commands.entity(entity).despawn();
    }
    for entity in &bullet_query {
        commands.entity(entity).despawn();
    }


    // Reset player
    if let Ok((mut player_transform, mut player_living)) = player_query.single_mut() {
        player_transform.translation = Vec3::new(0.0, 0.5, 0.0);
        player_living.health = player_living.max_health;
    }
}

fn handle_inertia(mut player_query: Query<(&mut Transform, &mut Inertia)>) {
    for (mut transform, mut inertia) in &mut player_query {
        let last_timestep_movement = transform.translation - inertia.prev_pos;
        inertia.prev_pos = transform.translation;
        transform.translation += last_timestep_movement * inertia.damping;
    }
}

