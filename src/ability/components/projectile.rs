use avian3d::prelude::*;
use bevy::prelude::*;

use crate::ability::components::common::{DynamicTarget, Lifetime};
use crate::ability::components::events::{OnActiveDespawn, OnCollision};
use crate::ability::components::spawn::{RadialSubCastOffset, handle_radial_sub_cast_offset};
use crate::ability::components::visual::LifetimeFadeout;
use crate::ability::{CastBy, IntendedTarget};
use crate::common::Faction;
use crate::event;
use crate::modifiers::*;

// Component for linear movement
#[derive(Component, Clone, Debug)]
pub struct InitialVelocity {
    pub direction: Dir3,
    pub base_speed: f32,
}

impl InitialVelocity {
    pub fn forward(base_speed: f32) -> Self {
        Self {
            direction: -Dir3::Z,
            base_speed,
        }
    }
}

pub fn handle_initial_velocity(
    mut commands: Commands,
    mut movement_query: Query<
        (
            Entity,
            &Transform,
            &InitialVelocity,
            &CastBy,
            &mut LinearVelocity,
        ),
        Without<RadialSubCastOffset>,
    >,
    modifiers: Query<&ModifierStack>,
) {
    for (entity, transform, movement, cast_by, mut velocity) in &mut movement_query {
        let speed = apply_modifier_if_present(
            modifiers.get(cast_by.entity).ok(),
            PROJECTILE_SPEED_MODIFIER,
            movement.base_speed,
        );

        velocity.0 += speed * (transform.rotation * movement.direction);

        commands.entity(entity).remove::<InitialVelocity>();
    }
}

// Component for damage on collision
#[derive(Component, Clone, Debug)]
pub struct DamageOnCollision {
    pub base_damage: f32,
}

// Component for despawning on collision
#[derive(Component, Clone, Debug)]
#[require(CollisionEventsEnabled)]
pub struct DespawnOnCollision;

// Component for homing movement - needs DynamicTarget to work
#[derive(Component, Clone, Debug)]
pub struct Homing {
    pub base_turn_speed: f32,
}

/// System to rotate entities with HomingMovement toward target_position
pub fn handle_homing_movement(
    transforms: Query<&Transform, Without<Homing>>,
    mut homing_query: Query<(
        &mut Transform,
        &Homing,
        &CastBy,
        &IntendedTarget,
        &mut LinearVelocity,
    )>,
    modifiers: Query<&ModifierStack>,
    time: Res<Time>,
) {
    for (mut transform, homing, cast_info, target, mut velocity) in &mut homing_query {
        /*let turn_speed = apply_modifier_if_present(
            modifiers.get(cast_info.entity).ok(),
            HOMING_STRENGTH_MODIFIER,
            homing.base_turn_speed,
        );

        if let Some(target_pos) = target
            .target
            .and_then(|e| transforms.get(e).ok())
            .map(|t| t.translation)
        {
            // Calculate rotation needed
            let target_direction = (target_pos - transform.translation).normalize();

            let current_direction = velocity.0.normalize();

            // Slerp toward target rotation
            let max_rotation = turn_speed * time.delta_secs();

            velocity.0 = current_direction
                .lerp(target_direction, max_rotation)
                .normalize()
                * velocity.0.length();
            transform.look_to(velocity.0, Vec3::Y);
        }*/

        // TODO
    }
}

/// Observer system to handle collision damage
pub fn handle_collision_damage(
    trigger: Trigger<OnCollisionStart>,
    damage_on_collision_query: Query<(
        &DamageOnCollision,
        &CastBy,
        &Transform,
        &LinearVelocity,
        &Faction,
    )>,
    mut damage_events: EventWriter<event::DamageEvent>,
    modifiers: Query<&ModifierStack>,
    target_transforms: Query<(&Transform, &Faction), Without<DamageOnCollision>>,
) {
    let damage_source_entity = trigger.target();
    let damaged_entity = trigger.event().collider;

    if let Ok((damage_component, cast_by, collider_transform, movement, source_faction)) =
        damage_on_collision_query.get(damage_source_entity)
        && let Ok((target_transform, target_faction)) = target_transforms.get(damaged_entity)
    {
        if target_faction != source_faction {
            damage_events.write(event::DamageEvent {
                target: damaged_entity,
                source: Some(cast_by.entity),
                damage: apply_modifier_if_present(
                    modifiers.get(cast_by.entity).ok(),
                    DAMAGE_MODIFIER,
                    damage_component.base_damage,
                ) as i32,
                position: target_transform.translation,
                impact_velocity: Some(movement.0),
            });
        }
    }
}

/// Observer system to handle despawning on collision
pub fn handle_collision_despawn(
    trigger: Trigger<OnCollisionStart>,
    mut commands: Commands,
    despawn_query: Query<&DespawnOnCollision>,
) {
    let collider_entity = trigger.target();

    if despawn_query.get(collider_entity).is_ok() {
        commands.entity(collider_entity).trigger(OnActiveDespawn);
        commands.entity(collider_entity).despawn();
    }
}

pub fn plugin(app: &mut bevy::app::App) {
    app.add_systems(
        Update,
        (
            handle_initial_velocity.after(handle_radial_sub_cast_offset),
            handle_homing_movement,
        ),
    );
    app.add_observer(handle_collision_damage);
    app.add_observer(handle_collision_despawn);
}

#[derive(Component, Debug, Clone)]
pub struct Projectile;

#[derive(Bundle, Clone, Debug)]
pub struct BasicProjectileBundle {
    pub rigidbody: RigidBody,
    pub lifetime: Lifetime,
    pub collider: Collider,
    pub initial_velocity: InitialVelocity,
    pub damage_on_collision: DamageOnCollision,
    pub despawn_on_collision: DespawnOnCollision,
    pub subcast_offset: RadialSubCastOffset,
    pub mesh: Mesh3d,
    pub mat: MeshMaterial3d<StandardMaterial>,
    pub fade: LifetimeFadeout,
    pub hooks: ActiveCollisionHooks,
    pub projectile: Projectile,
    pub lock_axes: LockedAxes,
}

impl BasicProjectileBundle {
    pub fn new(
        lifetime: f32,
        radius: f32,
        base_vel: f32,
        base_damage: f32,
        color: Color,
        meshes: &mut Assets<Mesh>,
        mats: &mut Assets<StandardMaterial>,
    ) -> Self {
        let mat = mats.add(StandardMaterial {
            base_color: color,
            emissive: color.into(),
            ..default()
        });

        let mesh = meshes.add(Sphere::new(radius));

        Self {
            rigidbody: RigidBody::Dynamic,
            lifetime: Lifetime::fixed_with_modifier(lifetime, PROJECTILE_DURATION_MODIFIER),
            collider: Collider::sphere(radius),
            initial_velocity: InitialVelocity::forward(base_vel),
            damage_on_collision: DamageOnCollision { base_damage },
            despawn_on_collision: DespawnOnCollision,
            subcast_offset: RadialSubCastOffset::from_degrees_per_cast(1.5, 5.0),
            mesh: Mesh3d(mesh),
            mat: MeshMaterial3d(mat),
            fade: LifetimeFadeout::new(0.1),
            hooks: ActiveCollisionHooks::FILTER_PAIRS,
            projectile: Projectile,
            lock_axes: LockedAxes::new().lock_translation_y(),
        }
    }
}
