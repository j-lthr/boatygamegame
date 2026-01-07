use crate::ability::components::blast::BlastBundle;
use crate::ability::components::common::{
    DynamicTarget, Lifetime, LifetimeFromCursor, SelectNearestTargetOnSpawn,
};
use crate::ability::components::dash::TransportCaster;
use crate::ability::components::projectile::{
    BasicProjectileBundle, DamageOnCollision, DespawnOnCollision, Homing, InitialVelocity,
};
use crate::ability::components::spawn::{RadialSubCastOffset, RandomSpawnOffset};
use crate::ability::components::subcast::{CastOnDespawn, SubCastOnce, TimedSubCast};
use crate::ability::slots::{
    AbilityKeymap, AbilitySlots, AbilityTargeting, SlotId, SlottedAbility,
};
use crate::ability::{CastConfig, DynamicAbility};
use crate::common::{HealthBundle, Targetable};
use crate::event::SpawnEvent;
use crate::init::DespawnOnReset;
use crate::init::GameInit;
use crate::input::Cursor;
use crate::modifiers::*;
use crate::rune::Collector;

use avian3d::prelude::*;
use avian3d::prelude::{
    Collider, ColliderConstructor, CollisionEventsEnabled, LinearVelocity, LockedAxes, RigidBody,
};
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
    mut assets: ResMut<AssetServer>,
) {
    let player_color = Color::srgb(20.0, 20.0, 20.0);

    let player_material = MeshMaterial3d(materials.add(StandardMaterial {
        emissive: player_color.into(),
        ..default()
    }));

    let projectile = DynamicAbility::from_components((BasicProjectileBundle::new(
        2.0,
        0.25,
        100.0,
        10.0,
        player_color,
        meshes.as_mut(),
        materials.as_mut(),
    ),));

    let projectile_ability = DynamicAbility::from_components((
        SubCastOnce::new(projectile, 1).modified_by(PROJECTILE_COUNT_MODIFIER),
    ));

    let dash_ability = DynamicAbility::from_components((
        RigidBody::Dynamic,
        LifetimeFromCursor::new().with_max_distance(50.0),
        TransportCaster,
        InitialVelocity::forward(1000.0),
    ));

    // Create abilities for the slots
    let abilities = vec![
        SlottedAbility::new(
            projectile_ability,
            AbilityTargeting::Cursor,
            0.2, // 0.5 second cooldown
        ),
        SlottedAbility::new(dash_ability, AbilityTargeting::Cursor, 2.0),
    ];

    // Create custom keymap
    let mut keymap = AbilityKeymap::new();
    keymap
        .bind_mouse(MouseButton::Left, SlotId(0))
        //.bind_mouse(MouseButton::Right, SlotId(1))
        .bind_key(KeyCode::Space, SlotId(1));

    // Player spawn point (invisible, camera will follow this)
    let player = commands
        .spawn((
            RigidBody::Dynamic,
            (
                Transform::from_xyz(0.0, 0.0, 0.0), // Eye level height
                Player { base_speed: 50.0 },
                Mesh3d(meshes.add(Sphere::new(0.7))),
                player_material,
                CollisionEventsEnabled,
                ColliderConstructor::ConvexHullFromMesh,
                AbilitySlots::with_abilities(abilities),
                keymap,
                HealthBundle::new(100, 0),
                Collector {
                    collect_radius: 1.0,
                    magnet_radius: 100.0,
                    magnet_force: 4000.0,
                },
                ActiveCollisionHooks::FILTER_PAIRS,
                LinearDamping(0.7),
            ),
            ExternalForce::ZERO.with_persistence(false),
            ModifierStack::default(),
            SpawnerTarget,
            Faction::Friendly,
            DespawnOnReset,
            StarEffect { spawn_rate: 40.0 },
            Targetable,
            LockedAxes::new().lock_translation_y().lock_rotation_x().lock_rotation_y().lock_rotation_z(),
            LookAtCursor
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
            clear_color: ClearColorConfig::Custom(Color::srgb(0.07, 0.04, 0.04)),
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
        Bloom {
            intensity: 0.05,
            ..Bloom::ANAMORPHIC
        },
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
        }
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
        &mut ExternalForce,
        &Player,
        Option<&ModifierStack>,
        &LinearVelocity
    )>,
    mut camera_query: Query<(&GlobalTransform, &mut PlayerCamera), With<Camera3d>>,
    cursor_query: Query<(&Cursor, &GlobalTransform)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    _scroll_wheel: Res<AccumulatedMouseScroll>,
    _time: Res<Time>,
) {
    if let (
        Ok((player_entity, mut player_transform, mut force, player, modifier_stack, vel)),
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

        force.apply_force(50.0*(velocity.normalize_or_zero() * speed - vel.0));


        if let Ok(cursor) = cursor_query.single() {
            player_transform.look_at(cursor.1.translation(), Vec3::Y);
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct LookAtCursor;





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
}
