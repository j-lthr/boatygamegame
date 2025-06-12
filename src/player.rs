use bevy::prelude::*;
use std::marker::PhantomData;

use crate::ability::AttemptCastEvent;
use crate::ability::basic_projectile_attack::{BasicProjectileAttack, BasicProjectileAttackParams};
use crate::ability::dash::Dash;
use crate::ability::dash::DashParams;
use crate::common::{Inertia, VelocityEWA};
use crate::common::Living;
use crate::event::SpawnEvent;
use crate::init::DespawnOnReset;
use crate::init::GameInit;
use crate::rune::Collector;
use crate::ui;

use bevy::core_pipeline::bloom::Bloom;
use bevy::core_pipeline::motion_blur::MotionBlur;
use bevy::core_pipeline::post_process::ChromaticAberration;
use bevy::core_pipeline::tonemapping::Tonemapping;

use crate::ability::AbilitySlot;
use crate::common::Faction;
use crate::enemy::spawn::SpawnerTarget;

// Component to mark the player
#[derive(Component)]
pub struct Player {
    pub speed: f32,
}

#[derive(Component)]
pub struct PlayerCamera {
    pub ground_offset: f32,
    pub height_offset: f32,
}

pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: ResMut<AssetServer>,
) {
    let player_color = Color::srgb(10.0, 10.0, 10.0);
    // Player spawn point (invisible, camera will follow this)
    let player = commands
        .spawn((
            Transform::from_xyz(0.0, 0.5, 0.0), // Eye level height
            Player { speed: 10.0 },
            Mesh3d(meshes.add(Sphere::new(0.5))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: player_color,
                ..default()
            })),
            AbilitySlot {
                cooldown: Timer::from_seconds(1.0, TimerMode::Once),
                name: "Dash",
                ability: Dash { range: 10.0 },
            },
            Living {
                health: 100,
                max_health: 100,
            },
            Inertia {
                prev_pos: Vec3::new(0.0, 0.5, 0.0), // Initial previous position
                damping: 0.1,                       // Damping factor for Verlet integration
            },
            AbilitySlot {
                cooldown: Timer::from_seconds(1.0, TimerMode::Once),
                name: "Shotgun",
                ability: BasicProjectileAttack {
                    bullet_count: 5,
                    spread: 0.05,
                    speed: 100.0,
                    lifetime: 1.0,
                    damage: 10,
                    color: player_color,
                },
            },
            Collector {
                collect_radius: 1.0,
                magnet_radius: 50.0,
                magnet_force: 5000.0,
            },
            SpawnerTarget,
            Faction::Friendly,
            VelocityEWA {
                velocity_ewa: Vec3::ZERO,
                tau: 2.0,
            },
            DespawnOnReset,
        ))
        .id();

    commands.send_event(SpawnEvent { entity: player });
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 0,
            hdr: true, // Enable HDR for better lighting
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Projection::from(PerspectiveProjection {
            fov: 95.0_f32.to_radians(),
            ..default()
        }),
        Transform::from_xyz(0.0, 10.0, -6.0).looking_at(Vec3::ZERO, Vec3::Z),
        // FirstPersonCamera::default(),
        SpatialListener::default(), // Spatial audio listener
        Tonemapping::TonyMcMapface, // 2. Using a tonemapper that desaturates to white is recommended
        Bloom::ANAMORPHIC,
        MotionBlur {
            shutter_angle: 1.0,
            samples: 2,
        },
        ChromaticAberration::default(),
        PlayerCamera {
            ground_offset: 10.0,
            height_offset: 15.0,
        }, //Atmosphere::EARTH,
           /*AtmosphereSettings {
               aerial_view_lut_max_distance: 3.2e5,
               scene_units_to_m: 1e+4,
               ..Default::default()
           },*/
    ));

    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        },
    ));
}

/// System to handle player movement with WASD keys (camera-relative)
pub fn handle_movement(
    mut player_query: Query<(Entity, &mut Transform, &Player)>,
    camera_query: Query<&GlobalTransform, With<Camera3d>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut dash_action: EventWriter<AttemptCastEvent<Dash>>,
    time: Res<Time>,
) {
    if let (Ok((player_entity, mut player_transform, player)), Ok(camera_transform)) =
        (player_query.single_mut(), camera_query.single())
    {
        let mut velocity = Vec3::ZERO;
        let speed = player.speed;

        // Get camera's forward and right vectors, but keep them horizontal for ground movement
        let forward = camera_transform.forward();
        let right = -camera_transform.right();
;
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

        if keyboard_input.pressed(KeyCode::Space) && velocity.length() > 1e-6 {
            dash_action.write(AttemptCastEvent {
                caster: player_entity,
                params: DashParams::Directional(velocity),
            });
        }

        // Normalize diagonal movement and apply
        if velocity.length() > 0.0 {
            velocity = velocity.normalize();
            player_transform.translation += velocity * speed * time.delta_secs();
        }
    }
}

pub fn handle_camera(
    player_query: Query<(&Transform, &Inertia, &Player, &VelocityEWA)>,
    mut camera_query: Query<(&mut Transform, &PlayerCamera), Without<Player>>,
    time: Res<Time>,
) {
    if let (Ok((player_transform, player_inertia, player, ewa)), Ok((mut camera_transform, camera))) =
        (player_query.single(), camera_query.single_mut())
    {

        let player_velocity = ewa.velocity_ewa;
        // Fixed camera offset - 60 degree downward angle (10 units up, 5.77 units back)
        let camera_offset = (( 0.1 * player_velocity)
            * camera.ground_offset)
            .with_y(camera.height_offset);

        let target_position = player_transform.translation + camera_offset;

        let lerp_factor = player.speed;

        // Smoothly move camera towards target position
        camera_transform.translation = camera_transform
            .translation
            .lerp(target_position,  lerp_factor * time.delta_secs());

        let target_transform = camera_transform.looking_at(player_transform.translation, Vec3::Y);

        camera_transform.rotation = camera_transform
            .rotation
            .slerp(target_transform.rotation, 0.5 * lerp_factor  * time.delta_secs());
    }
}

/// System to handle shooting using ability system
pub fn shoot_gun(
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    player_query: Query<Entity, With<Player>>,
    camera_query: Query<(&GlobalTransform, &Camera), With<PlayerCamera>>,
    mut shotgun_action: EventWriter<AttemptCastEvent<BasicProjectileAttack>>,
) {
    if let (Ok(player), Ok((camera_transform, camera))) =
        (player_query.single(), camera_query.single())
    {
        if mouse_input.pressed(MouseButton::Left) {
            if let Some(cursor_pos) = ui::compute_3d_cursor_pos(windows, camera, camera_transform)
            {
                shotgun_action.write(AttemptCastEvent {
                    caster: player,
                    params: BasicProjectileAttackParams {
                        target_position: cursor_pos,
                    },
                });
            }
        }
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(GameInit, spawn_player);
    app.add_systems(Startup, spawn_camera);
}
