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

const FONT_PATH: &str = "fonts/Jersey15-Regular.ttf";

// Component to mark the player
#[derive(Component)]
struct Player;

#[derive(Component)]
struct DashTimer {
    timer: Timer,
}

#[derive(Component)]
struct GunOwner {
    bullet_timer: Timer,
}

#[derive(Component)]
struct HealthBar {
    pub entity: Entity,
}

// Component for the gun
#[derive(Component)]
struct Gun;

// Component for bullets
#[derive(Component)]
struct Bullet {
    direction: Vec3,
    speed: f32,
    lifetime: f32,
}

// Component for enemies
#[derive(Component)]
struct Enemy {
    speed: f32,
    jitter: f32,
}

#[derive(Component)]
struct SlamAttacker {
    slam_timer: Timer,
    slam_range: f32,
    slam_damage: i32,
}

#[derive(Component)]
struct FadeEffect {
    radius: f32,
    alpha: f32,
}

#[derive(Component)]
struct Living {
    health: i32,
    max_health: i32,
}

// Timer for enemy spawning
#[derive(Resource)]
struct EnemySpawnTimer {
    timer: Timer,
}

#[derive(Event)]
struct SpawnEvent {
    entity: Entity,
}

#[derive(Resource)]
struct GameScore {
    current: i32,
    kills: i32,
    combo: i32,
    combo_timer: Timer,
}

impl Default for GameScore {
    fn default() -> Self {
        Self {
            current: 0,
            kills: 0,
            combo: 0,
            combo_timer: Timer::from_seconds(3.0, TimerMode::Once),
        }
    }
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Playing,
    GameOver,
}

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct ComboText;

#[derive(Component)]
struct GameOverScreen;

#[derive(Component)]
struct FinalScoreText;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AudioPlugin {
            global_volume: Volume::Linear(2.0).into(),
            ..default()
        }))
        .add_audio_source::<fx::fm::FMSound>()
        .init_state::<GameState>()
        .init_resource::<GameScore>()
        .insert_resource(EnemySpawnTimer {
            timer: Timer::from_seconds(2.0, TimerMode::Repeating),
        })
        .add_event::<SpawnEvent>()
        .add_systems(Startup, (setup, fx::blood::setup_blood_materials, setup_ui))
        .add_systems(
            Update,
            (
                move_player,
                shoot_gun,
                move_bullets,
                cleanup_bullets,
                spawn_enemies,
                move_enemies,
                bullet_enemy_collision,
                fx::blood::blood_particle_physics,
                fx::blood::blood_particle_rendering,
                fx::blood::cleanup_blood_particles,
                fx::blood::fade_blood_splatters,
                spawn_healthbar,
                update_health_bars,
                cleanup_enemy_health_bars,
                camera_follow_player,
                enemy_slam_attack,
                animate_fade_effects,
                update_score_display,
                update_combo_system,
            ).run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (
                handle_game_over_input,
                update_game_over_screen,
            ).run_if(in_state(GameState::GameOver)),
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
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: ResMut<AssetServer>,
) {
    // Player spawn point (invisible, camera will follow this)
    let player = commands
        .spawn((
            Transform::from_xyz(0.0, 0.5, 0.0), // Eye level height
            Player,
            Mesh3d(meshes.add(Sphere::new(0.5))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(10.0, 10.0, 10.0),
                ..default()
            })),
            DashTimer {
                timer: Timer::from_seconds(2.0, TimerMode::Once), // Dash cooldown
            },
            Living {
                health: 100,
                max_health: 100,
            },
        ))
        .id();

    let ground_plane = asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/ground.glb"));

    commands.spawn(SceneRoot(ground_plane));

    commands.send_event(SpawnEvent { entity: player });

    // First-person camera
    let camera_entity = commands
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
            GunOwner {
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
        ))
        .id();

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

fn update_combo_system(
    mut score: ResMut<GameScore>,
    time: Res<Time>,
) {
    score.combo_timer.tick(time.delta());
    
    // Reset combo if timer expires
    if score.combo_timer.finished() && score.combo > 0 {
        score.combo = 0;
    }
}

/// System to handle player movement with WASD keys (camera-relative)
fn move_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut player_query: Query<(&mut Transform, &mut DashTimer), With<Player>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if let Ok((mut player_transform, mut dash_timer)) = player_query.single_mut() {
        let mut velocity = Vec3::ZERO;
        let speed = 4.0;

        dash_timer.timer.tick(time.delta());

        // Get camera's forward and right vectors, but keep them horizontal for ground movement
        let forward = Vec3::Z; // camera_transform.forward();
        let right = Vec3::X; //camera_transform.right();

        // Project forward and right vectors onto the horizontal plane (y=0)
        let forward_horizontal = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
        let right_horizontal = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

        // WASD movement relative to camera direction
        if keyboard_input.pressed(KeyCode::KeyW) {
            velocity += forward_horizontal;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            velocity -= forward_horizontal;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            velocity += right_horizontal;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            velocity -= right_horizontal;
        }

        if keyboard_input.pressed(KeyCode::Space) {
           if dash_timer.timer.finished() {
                // Dash forward in the direction the player is facing
                let dash_distance = 2.5; // Distance to dash


                 commands
        .spawn((
            player_transform.clone(), // Eye level height
            Mesh3d(meshes.add(Sphere::new(1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(10.0, 10.0, 10.0),
                ..default()
            })),
            FadeEffect {
                radius: 1.0,
                alpha: 1.0, // Start fully visible
            },
        ));

                player_transform.translation += velocity * dash_distance;
                dash_timer.timer.reset(); // Reset dash timer

                
            }
        }

        // Normalize diagonal movement and apply
        if velocity.length() > 0.0 {
            velocity = velocity.normalize();
            player_transform.translation += velocity * speed * time.delta_secs();
        }
    }
}

/// System to make camera follow player with smooth interpolation
fn camera_follow_player(
    player_query: Query<&Transform, (With<Player>, Without<Camera3d>)>,
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<Player>)>,
    time: Res<Time>,
) {
    if let (Ok(player_transform), Ok(mut camera_transform)) =
        (player_query.single(), camera_query.single_mut())
    {
        // Define the offset from player to camera (above and behind)
        let camera_offset = Vec3::new(0.0, 10.0, -10.0);

        // Calculate desired camera position
        let target_position = player_transform.translation + camera_offset;

        // Lerp factor - higher values = faster following, lower = smoother
        let lerp_factor = 1.0 * time.delta_secs();

        // Smoothly move camera towards target position
        camera_transform.translation = camera_transform
            .translation
            .lerp(target_position, lerp_factor);

        // Make camera look at the player
        let look_target = player_transform.translation + Vec3::new(0.0, 0.5, 0.0); // Look slightly above player center
        camera_transform.look_at(look_target, Vec3::Y);
    }
}

fn compute_3d_cursor_pos(
    windows: Query<&Window>,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec3> {
    let window = windows.single().ok()?;
    let cursor_pos = window.cursor_position()?;
    let ray = camera
        .viewport_to_world(camera_transform, cursor_pos)
        .ok()?;

    ray.intersect_plane(Vec3::ZERO, InfinitePlane3d { normal: Dir3::Y })
        .map(|intersection| ray.origin + ray.direction * intersection)
}

/// System to handle shooting
fn shoot_gun(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut shoot_sounds: ResMut<Assets<fx::fm::FMSound>>,
    time: Res<Time>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<(&GlobalTransform, &mut GunOwner, &Camera)>,
) {
    if let (Ok((camera_transform, mut gun_owner, camera)), Ok(player_transform)) =
        (camera_query.single_mut(), player_query.single())
    {
        gun_owner.bullet_timer.tick(time.delta());

        // Check if the gun's bullet timer allows shooting
        if mouse_input.pressed(MouseButton::Left) && gun_owner.bullet_timer.finished() {
            gun_owner.bullet_timer.reset(); // Reset timer for next shot

            // Play shooting sound (FM synthesis)
            let shoot_sound_handle = shoot_sounds.add(fx::fm::FMSound {
                config: fx::fm::GUN_SOUND,             // How much the frequency varies
                duration: Duration::from_millis(1000), // Short punchy sound
            });

            let bullet_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.2, 0.2, 0.2),
                metallic: 0.5,
                perceptual_roughness: 0.5,
                emissive: Color::srgb(5.0, 5.0, 5.0).into(), // Slightly glowing
                ..default()
            });

            if let Some(cursor_pos) = compute_3d_cursor_pos(windows, camera, &camera_transform) {
                for i in 0..5 {
                    // Calculate bullet direction based on camera forward vector
                    let direction = (cursor_pos - player_transform.translation)
                        .normalize()
                        .with_y(0.0)
                        + Vec3::new(fastrand::f32() - 0.5, 0.0, fastrand::f32() - 0.5) * 0.5; // Add slight random jitter

                    // Spawn bullet at camera position
                    commands.spawn((
                        Mesh3d(meshes.add(Sphere::new(0.1))),
                        MeshMaterial3d(bullet_mat.clone()), // Red color
                        Transform::from_translation(player_transform.translation),
                        Bullet {
                            direction,
                            speed: 30.0,
                            lifetime: 1.0, // 5 seconds before cleanup
                        },
                        AudioPlayer(shoot_sound_handle.clone()),
                        PlaybackSettings::ONCE
                            .with_spatial(true)
                            .with_volume(Volume::Decibels(24.0)), // Play sound once with spatial audio
                    ));
                }
            }
        }
    }
}

/// System to move bullets
fn move_bullets(mut bullet_query: Query<(&mut Transform, &Bullet)>, time: Res<Time>) {
    for (mut transform, bullet) in &mut bullet_query {
        transform.translation += bullet.direction * bullet.speed * time.delta_secs();
    }
}

/// System to cleanup old bullets
fn cleanup_bullets(
    mut commands: Commands,
    mut bullet_query: Query<(Entity, &mut Bullet)>,
    time: Res<Time>,
) {
    for (entity, mut bullet) in &mut bullet_query {
        bullet.lifetime -= time.delta_secs();
        if bullet.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// System to spawn enemies
fn spawn_enemies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawn_timer: ResMut<EnemySpawnTimer>,
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut spawn_event_writer: EventWriter<SpawnEvent>,
) {
    spawn_timer.timer.tick(time.delta());

    if spawn_timer.timer.just_finished() {
        if let Ok(player_transform) = player_query.single() {
            // Spawn enemy at random position around the player
            let angle = fastrand::f32() * 2.0 * PI;
            let distance = 10.0 + fastrand::f32() * 5.0; // 10-15 units away
            let spawn_pos = Vec3::new(
                player_transform.translation.x + angle.cos() * distance,
                0.75, // Ground level (half of capsule height)
                player_transform.translation.z + angle.sin() * distance,
            );

            let mesh = meshes.add(generate_rock_mesh(&procedural::rock::RockConfig::default()));

            let enemy = commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(10.0, 5.0, 0.0),
                        emissive: Color::srgb(4.0, 2.0, 0.0).into(), // Slightly glowing
                        ..default()
                    })),
                    Transform::from_translation(spawn_pos),
                    Enemy {
                        speed: 5.0 + fastrand::f32() * 0.2,
                        jitter: 0.1, //0.1 + fastrand::f32() * 0.2, // Random jitter between 0.1 and 0.3
                    },
                    SlamAttacker {
                        slam_timer: Timer::from_seconds(0.25, TimerMode::Once),
                        slam_range: 3.0,
                        slam_damage: 33,
                    },
                    Living {
                        health: 8,
                        max_health: 8,
                    },
                ))
                .id();

            spawn_event_writer.write(SpawnEvent { entity: enemy });
        }
    }
}

/// System to move enemies toward player with rolling motion
fn move_enemies(
    mut enemy_query: Query<(&mut Transform, &Enemy)>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (mut enemy_transform, enemy) in &mut enemy_query {
            // Calculate direction to player
            let direction =
                (player_transform.translation - enemy_transform.translation).normalize_or_zero();
            let horizontal_direction = Vec3::new(direction.x, 0.0, direction.z).normalize_or_zero();

            // Calculate movement for this frame
            let movement_distance = enemy.speed * time.delta_secs();
            let movement_vector = horizontal_direction * movement_distance;

            // Move enemy toward player
            enemy_transform.translation += movement_vector;

            enemy_transform.translation.x += enemy.jitter * (fastrand::f32() - 0.5); // Add slight random jitter
            enemy_transform.translation.z += enemy.jitter * (fastrand::f32() - 0.5); // Add slight random jitter

            // Add rolling motion
            if horizontal_direction.length() > 0.0 {
                // Assume boulder radius for rolling calculation (adjust as needed)
                let boulder_radius = 0.5; // Adjust this based on your boulder size

                // Calculate rotation angles based on movement
                let roll_angle = movement_vector.x / boulder_radius; // Roll around Z-axis for X movement
                let pitch_angle = -movement_vector.z / boulder_radius; // Pitch around X-axis for Z movement (negative for correct direction)

                // Apply rolling rotation
                let roll_rotation = Quat::from_rotation_z(roll_angle);
                let pitch_rotation = Quat::from_rotation_x(pitch_angle);

                // Combine rotations and apply to current rotation
                enemy_transform.rotation =
                    enemy_transform.rotation * roll_rotation * pitch_rotation;
            }
        }
    }
}
fn bullet_enemy_collision(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    bullet_query: Query<(Entity, &Transform, &Bullet), Without<Enemy>>,
    mut enemy_query: Query<(Entity, &mut Transform, &mut Enemy, &mut Living), Without<Bullet>>,
    mut shoot_sounds: ResMut<Assets<fx::fm::FMSound>>,
    blood_materials: Res<fx::blood::BloodMaterials>,
    mut score: ResMut<GameScore>,
) {
    for (bullet_entity, bullet_transform, bullet) in &bullet_query {
        for (entity, mut enemy_transform, mut enemy, mut living) in &mut enemy_query {
            let distance = bullet_transform
                .translation
                .distance(enemy_transform.translation);

            if distance < 0.7 {
                // Hit detection radius
                commands.entity(bullet_entity).despawn();

                // Damage enemy
                living.health -= 1;

                let bullet_velocity = bullet.direction * bullet.speed;

                fx::blood::spawn_blood_explosion(
                    &mut commands,
                    &mut meshes,
                    bullet_transform.translation,
                    &blood_materials,
                    15,
                    2.5,
                    -bullet_velocity * 0.1,
                    Vec3::new(1.0, 0.5, 0.0),
                );

                let shoot_sound_handle = shoot_sounds.add(fx::fm::FMSound {
                    config: fx::fm::HIT_SOUND,
                    duration: Duration::from_millis(100),
                });

                commands.spawn((
                    AudioPlayer(shoot_sound_handle),
                    PlaybackSettings::DESPAWN
                        .with_spatial(true)
                        .with_volume(Volume::Decibels(12.0)),
                    Transform::from_translation(enemy_transform.translation),
                ));

                if living.health <= 0 {
                    // Enemy killed - update score!
                    score.kills += 1;
                    score.combo += 1;
                    score.combo_timer.reset(); // Reset combo timer
                    
                    // Calculate points with combo multiplier
                    let base_points = 100;
                    let combo_bonus = (score.combo - 1) * 50; // 50 extra points per combo level
                    let points_earned = base_points + combo_bonus;
                    score.current += points_earned;

                    fx::blood::spawn_blood_explosion(
                        &mut commands,
                        &mut meshes,
                        enemy_transform.translation,
                        &blood_materials,
                        100,
                        5.0,
                        Vec3::ZERO,
                        Vec3::new(10.0, 5.0, 0.0),
                    );

                    let shoot_sound_handle = shoot_sounds.add(fx::fm::FMSound {
                        config: fx::fm::DEATH_SOUND,
                        duration: Duration::from_millis(100),
                    });

                    commands.spawn((
                        AudioPlayer(shoot_sound_handle),
                        PlaybackSettings::DESPAWN
                            .with_spatial(true)
                            .with_volume(Volume::Decibels(36.0)),
                        Transform::from_translation(enemy_transform.translation),
                    ));

                    commands.entity(entity).despawn();
                }

                enemy_transform.translation += bullet_velocity.with_y(0.0) * 0.01;
                break;
            }
        }
    }
}


// Updated enemy_slam_attack system to trigger game over
fn enemy_slam_attack(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut shoot_sounds: ResMut<Assets<fx::fm::FMSound>>,
    time: Res<Time>,
    mut enemy_query: Query<(&mut Transform, &mut Enemy, &mut SlamAttacker)>,
    mut player_query: Query<(&mut Transform, &mut Living), (With<Player>, Without<Enemy>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if let Ok((mut player_transform, mut player_living)) = player_query.single_mut() {
        for (mut enemy_transform, mut enemy, mut slam_attacker) in &mut enemy_query {
            slam_attacker.slam_timer.tick(time.delta());

            let distance = enemy_transform
                .translation
                .distance(player_transform.translation);

            if distance <= slam_attacker.slam_range && slam_attacker.slam_timer.finished() {
                slam_attacker.slam_timer.reset();
                player_living.health -= slam_attacker.slam_damage;

                let knockback_vector = (player_transform.translation - enemy_transform.translation)
                    .normalize_or_zero()
                    .with_y(0.0)
                    * slam_attacker.slam_range
                    * 1.0;

                player_transform.translation += knockback_vector;

                spawn_slam_effect(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    enemy_transform.translation,
                    slam_attacker.slam_range,
                );

                let slam_sound_handle = shoot_sounds.add(fx::fm::FMSound {
                    config: fx::fm::SLAM,
                    duration: Duration::from_millis(400),
                });

                commands.spawn((
                    AudioPlayer(slam_sound_handle),
                    PlaybackSettings::DESPAWN
                        .with_spatial(true)
                        .with_volume(Volume::Decibels(18.0)),
                    Transform::from_translation(enemy_transform.translation),
                ));

                println!("Player hit by slam attack! Health: {}", player_living.health);

                // Check if player is dead
                if player_living.health <= 0 {
                    println!("Game Over!");
                    next_state.set(GameState::GameOver);
                }
            }
        }
    }
}

// New function to spawn the circular slam effect
fn spawn_slam_effect(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    radius: f32,
) {
    // Create a circular mesh (torus or cylinder would work, but let's use a thin cylinder)
    let circle_mesh = meshes.add(Cylinder::new(2.0, 0.1)); // radius 2.0, height 0.1

    // Create a material with high emission and transparency
    let slam_material = materials.add(StandardMaterial {
        base_color: Color::srgba(2.0, 1.0, 1.0, 0.8), // Orange-red with transparency
        emissive: Color::srgb(2.0, 1.0, 1.0).into(),  // Bright orange glow
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    // Spawn the slam effect slightly above ground
    commands.spawn((
        Mesh3d(circle_mesh),
        MeshMaterial3d(slam_material),
        Transform::from_translation(position + Vec3::new(0.0, 0.05, 0.0)),
        FadeEffect {
            radius: radius * 1.1,
            alpha: 1.0, // Start fully visible
        },
    ));
}

// New system to animate and cleanup slam effects
fn animate_fade_effects(
    mut commands: Commands,
    mut slam_effect_query: Query<(Entity, &mut Transform, &mut FadeEffect)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut slam_effect) in &mut slam_effect_query {
        // Fade duration - effect lasts for 0.5 seconds
        let fade_speed = 10.0; // Higher = faster fade
        slam_effect.alpha -= fade_speed * time.delta_secs();

        // Expand the circle slightly as it fades
        slam_effect.radius += 3.0 * time.delta_secs(); // Expand at 3 units per second
        transform.scale = Vec3::splat(slam_effect.radius / 2.0); // Scale based on radius

        // Update material alpha if we can access it
        if let Ok(material_handle) = material_query.get(entity) {
            if let Some(material) = materials.get_mut(&material_handle.0) {
                // Update both base color alpha and emissive intensity
                material.base_color.set_alpha(slam_effect.alpha.max(0.0));
                let emissive_intensity = slam_effect.alpha.max(0.0) * 5.0; // Scale emissive with alpha
                material.emissive =
                    Color::srgb(emissive_intensity, emissive_intensity * 0.3, 0.0).into();
            }
        }

        // Remove effect when fully faded
        if slam_effect.alpha <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_healthbar(
    mut commands: Commands,
    mut spawn_event_reader: EventReader<SpawnEvent>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for event in spawn_event_reader.read() {
        // Spawn health bar above enemy
        let texture = asset_server.load("sprites/healthbar.png");
        let layout = TextureAtlasLayout::from_grid(UVec2::new(64, 4), 1, 8, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        commands.spawn((
            Sprite::from_atlas_image(
                texture,
                TextureAtlas {
                    layout: texture_atlas_layout,
                    index: 7,
                },
            ),
            Transform::from_xyz(0.0, 0.0, 0.0),
            HealthBar {
                entity: event.entity,
            },
        ));
    }
}

fn update_health_bars(
    mut health_bar_query: Query<(&HealthBar, &mut Sprite, &mut Transform)>,
    living_query: Query<(&Living, &GlobalTransform)>,
    camera_3d_query: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    camera_2d_query: Query<(&GlobalTransform, &Camera), With<Camera2d>>,
) {
    for (health_bar, mut sprite, mut healthbar_transform) in &mut health_bar_query {
        if let Ok((living, enemy_transform)) = living_query.get(health_bar.entity) {
            // Update health bar position above enemy
            if let Some(ref mut texture_atlas) = sprite.texture_atlas {
                texture_atlas.index = ((7.0 * (living.health as f32 / living.max_health as f32))
                    as usize)
                    .max(0)
                    .min(7); // Update based on health
            }

            if let (Ok((camera_3d_transform, camera_3d)), Ok((camera_2d_transform, camera_2d))) =
                (camera_3d_query.single(), camera_2d_query.single())
            {
                let _ = camera_3d
                    .world_to_viewport(
                        camera_3d_transform,
                        enemy_transform.translation() + Vec3::Y * 1.5,
                    )
                    .map(|viewport_position| {
                        camera_2d
                            .viewport_to_world_2d(camera_2d_transform, viewport_position)
                            .map(|world_pos_2d| {
                                // Update health bar position in 2D space
                                healthbar_transform.translation =
                                    Vec3::new(world_pos_2d.x, world_pos_2d.y, 0.0);
                            })
                    });
            }
        }
    }
}

fn cleanup_enemy_health_bars(
    mut commands: Commands,
    health_bar_query: Query<(Entity, &HealthBar)>,
    enemy_query: Query<()>,
) {
    for (health_bar_entity, health_bar) in &health_bar_query {
        // If the enemy no longer exists, remove its health bar
        if enemy_query.get(health_bar.entity).is_err() {
            commands.entity(health_bar_entity).despawn();
        }
    }
}


// Setup game over screen
fn setup_game_over_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    score: Res<GameScore>,
) {
    // Background overlay
    commands.spawn((
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
        score_text.0 = format!("🏆 Final Score: {} 🏆", score.current);
    }
}

// Cleanup game over screen
fn cleanup_game_over_screen(
    mut commands: Commands,
    game_over_query: Query<Entity, With<GameOverScreen>>,
    mut score: ResMut<GameScore>,
    mut enemy_spawn_timer: ResMut<EnemySpawnTimer>,
    // Reset all game entities
    enemy_query: Query<Entity, With<Enemy>>,
    bullet_query: Query<Entity, With<Bullet>>,
    effect_query: Query<Entity, With<FadeEffect>>,
    mut player_query: Query<(&mut Transform, &mut Living), With<Player>>,
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
    for entity in &effect_query {
        commands.entity(entity).despawn();
    }

    // Reset player
    if let Ok((mut player_transform, mut player_living)) = player_query.single_mut() {
        player_transform.translation = Vec3::new(0.0, 0.5, 0.0);
        player_living.health = player_living.max_health;
    }
}