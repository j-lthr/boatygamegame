use crate::ability::DynamicAbility;
use crate::ability::components::blast::BlastBundle;
use crate::ability::components::common::{DynamicTarget, Lifetime, SelectNearestTargetOnSpawn};
use crate::ability::components::projectile::{
    DamageOnCollision, DespawnOnCollision, Homing, InitialVelocity,
};
use crate::ability::components::spawn::{RadialSubCastOffset, RandomSpawnOffset};
use crate::ability::components::subcast::SubCastOnce;
use crate::ability::slots::{AbilityKeymap, AbilitySlots, AbilityTargeting, SlottedAbility};
use crate::common::{HealthBundle, Targetable};
use crate::event::SpawnEvent;
use crate::init::DespawnOnReset;
use crate::init::GameInit;
use crate::input::Cursor;
use crate::modifiers::*;
use crate::rune::Collector;

use avian3d::prelude::{Collider, LinearVelocity, LockedAxes, RigidBody};
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use std::f32;

use bevy::core_pipeline::bloom::Bloom;
use bevy::core_pipeline::motion_blur::MotionBlur;
use bevy::core_pipeline::post_process::ChromaticAberration;
use bevy::core_pipeline::tonemapping::Tonemapping;

use crate::ability::components::visual::LifetimeFadeout;
use crate::common::Faction;
use crate::enemy::spawn::SpawnerTarget;
use crate::fx::stars::StarEffect;

// Component to mark the player
#[derive(Component, Debug)]
pub struct Player {
    pub base_speed: f32,
}


#[derive(Component, Debug)]
pub struct PlayerCamera {
    pub ground_offset: f32,
    pub height_offset: f32,
    pub pan_factor: f32,
    pub pan_ratio: f32,
    pub lerp_factor: f32,
}

pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let player_color = Color::srgb(10.0, 10.0, 10.0);

    let bullet_mat = materials.add(StandardMaterial {
        base_color: player_color,
        metallic: 0.5,
        perceptual_roughness: 0.5,
        emissive: player_color.into(),
        ..default()
    });

    let mortar_blast = DynamicAbility::from_components((
        BlastBundle::new(
            &mut meshes,
            &mut materials,
            Color::srgb(30.0, 30.0, 30.0),
            0.5,
            10,
            0.1,
        ),
        DespawnOnReset,
    ));

    let projectile = DynamicAbility::from_components((
        RigidBody::Dynamic,
        Lifetime::fixed(0.5),
        LifetimeFadeout::new(0.1),
        Collider::sphere(0.1),
        InitialVelocity::forward(50.0),
        Homing {base_turn_speed: 1.0},
        //LifetimeFromCursor,
        (
            DespawnOnCollision,
            DamageOnCollision { base_damage: 10.0 },
            DynamicTarget::new(),
            SelectNearestTargetOnSpawn::new(3.0),
            RadialSubCastOffset::from_degrees_per_cast(0.0, 10.0),
            RandomSpawnOffset::new(0.0, 0.01),
        ),
        Mesh3d(meshes.add(Sphere::new(0.1))),
        MeshMaterial3d(bullet_mat.clone()),
        //CastOnDespawn::new(subcast_ability, 5)
    ));

    let projectile_ability = DynamicAbility::from_components((
        SubCastOnce::new(projectile, 1).modified_by(PROJECTILE_COUNT_MODIFIER),
    ));

    // Create abilities for the slots
    let abilities = vec![
        SlottedAbility::new(
            projectile_ability,
            AbilityTargeting::NearestEnemyToCursor { max_range: 20.0 },
            0.5, // 0.5 second cooldown
        ),
        SlottedAbility::new(
            mortar_blast.clone(),
            AbilityTargeting::Cursor,
            2.0, // 2 second cooldown
        ),
    ];

    // Create custom keymap
    let keymap = AbilityKeymap::new();

    // Player spawn point (invisible, camera will follow this)
    let player = commands
        .spawn((
            (
                Transform::from_xyz(0.0, 0.0, 0.0), // Eye level height
                Player { base_speed: 15.0 },
                Mesh3d(meshes.add(Sphere::new(0.5))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: player_color,
                    ..default()
                })),
                AbilitySlots::with_abilities(abilities),
                keymap,
                HealthBundle::new(50, 1),
                RigidBody::Kinematic,
                Collector {
                    collect_radius: 1.0,
                    magnet_radius: 25.0,
                    magnet_force: 2000.0,
                },
            ),
            ModifierStack::default(),
            SpawnerTarget,
            Faction::Friendly,
            DespawnOnReset,
            StarEffect {
                spawn_rate: 100.0,
            },
            Targetable,
            LockedAxes::new().lock_translation_y(),
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
            fov: 105.0_f32.to_radians(),
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
            ground_offset: 0.0,
            height_offset: 40.0,
            pan_factor: 1.0,
            pan_ratio: 1.0,
            lerp_factor: 10.0,
        },
        // PlayerCamera {
        //     ground_offset: 0.0,
        //     height_offset: 40.0,
        //     pan_factor: 1.0,
        //     pan_ratio: -2.0,
        //     lerp_factor: 5.0,
        // }

        // Atmosphere::EARTH,
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
    mut player_query: Query<(
        Entity,
        &mut Transform,
        &mut LinearVelocity,
        &Player,
        Option<&ModifierStack>,
    )>,
    mut camera_query: Query<(&GlobalTransform, &mut PlayerCamera), With<Camera3d>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    _scroll_wheel: Res<AccumulatedMouseScroll>,
    _time: Res<Time>,
) {
    if let (
        Ok((player_entity, _player_transform, mut linear_velocity, player, modifier_stack)),
        Ok((_camera_transform, _camera)),
    ) = (player_query.single_mut(), camera_query.single_mut())
    {
        let mut velocity = Vec3::ZERO;

        // Get camera's forward and right vectors, but keep them horizontal for ground movement
        //let forward = camera_transform.forward();
        //let right = -camera_transform.right();

        let forward = Vec3::Z;
        let right = Vec3::X;

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

        let speed =
            apply_modifier_if_present(modifier_stack, PLAYER_SPEED_MODIFIER, player.base_speed);

        linear_velocity.0 = velocity.normalize() * speed;
    }
}

pub fn handle_camera(
    player_query: Query<(&Transform, &Player)>,
    mut camera_query: Query<(&mut Transform, &PlayerCamera), Without<Player>>,
    window: Single<&Window>,
    time: Res<Time>,
) {
    if let (Ok((player_transform, _player)), Ok((mut camera_transform, camera))) =
        (player_query.single(), camera_query.single_mut())
    {
        let mouse_pos = window
            .cursor_position()
            .map(|pos| pos / window.size() - Vec2::splat(0.5))
            .map(|pos| Vec3::new(pos.x, 0.0, pos.y))
            .unwrap_or(Vec3::ZERO);

        // Fixed camera offset - 60 degree downward angle (10 units up, 5.77 units back)
        let camera_offset = (-Vec3::Z * camera.ground_offset).with_y(camera.height_offset);

        let target_position = player_transform.translation + camera_offset
            - camera.height_offset * mouse_pos * camera.pan_factor * camera.pan_ratio;

        let lerp_factor = (camera.lerp_factor * time.delta_secs()).min(1.0);

        // Smoothly move camera towards target position
        camera_transform.translation = camera_transform
            .translation
            .lerp(target_position, lerp_factor);

        let target_transform = camera_transform.looking_at(
            player_transform.translation - camera.height_offset * mouse_pos * camera.pan_factor,
            Vec3::Z,
        );

        camera_transform.rotation = camera_transform
            .rotation
            .slerp(target_transform.rotation, lerp_factor);
    }
}

/// System to handle shooting using ability system
pub fn shoot_gun(
    mouse_input: Res<ButtonInput<MouseButton>>,
    player_query: Query<(Entity, &Transform), With<Player>>,
    cursor_query: Query<&Transform, With<Cursor>>,
    _time: Res<Time>,
) {
    if let Ok((_player, _player_transform)) = player_query.single()
        && mouse_input.pressed(MouseButton::Left)
        && let Ok(_cursor_transform) = cursor_query.single()
    {}
}

/*pub fn player_vfx(
    mut player_query: Query<(&mut Transform, &AbilitySlot<DynamicAbility>), With<Player>>,
    mut dash_cast_events: EventReader<CastEvent<Dash>>,
    time: Res<Time>,
) {
    for (mut transform, ability) in &mut player_query {
        transform.scale = transform.scale.lerp(
            Vec3::splat(1.0 - ability.cooldown.fraction_remaining() + 0.2),
            time.delta_secs() * 10.0,
        );
    }

    for event in dash_cast_events.read() {
        if let Ok((mut transform, _ability)) = player_query.get_mut(event.caster) {
            if let DashParams::Directional(direction) = event.params {
                transform.scale += direction.abs();
            }
        }
    }
}*/

pub fn plugin(app: &mut App) {
    app.add_systems(GameInit, spawn_player);
    app.add_systems(Startup, spawn_camera);
    // app.add_systems(Update, player_vfx);
}
