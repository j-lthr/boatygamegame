use std::time::Duration;

use bevy::audio::Volume;
use bevy::prelude::*;

use crate::ability::components::events::{OnActiveDespawn, OnCollision};
use crate::ability::CastBy;
use crate::ability::components::common::DynamicTarget;
use crate::common;
use crate::common::Faction;
use crate::event;
use crate::fx;
use crate::modifiers::*;

// Component for linear movement
#[derive(Component, Clone)]
pub struct MoveForward {
    pub base_speed: f32,
}

// Component for simple collision detection
#[derive(Component, Clone)]
pub struct SimpleCollider {
    pub radius: f32,
}

// Component for damage on collision
#[derive(Component, Clone)]
pub struct DamageOnCollision {
    pub base_damage: f32,
}

// Component for despawning on collision
#[derive(Component, Clone)]
pub struct DespawnOnCollision;

// Component for homing movement - needs DynamicTarget to work
#[derive(Component, Clone)]
pub struct Homing {
    pub base_turn_speed: f32,
}

/// System to move entities with MoveForward
pub fn handle_forward_movement(
    mut movement_query: Query<(&mut Transform, &MoveForward, &CastBy)>,
    modifiers: Query<&ModifierStack>,
    time: Res<Time>,
) {
    for (mut transform, movement, cast_by) in &mut movement_query {
        let speed = apply_modifier_if_present(modifiers.get(cast_by.entity).ok(), PROJECTILE_SPEED_MODIFIER, movement.base_speed);
        let fwd = transform.forward();
        transform.translation += fwd * speed * time.delta_secs();
    }
}

/// System to rotate entities with HomingMovement toward target_position
pub fn handle_homing_movement(
    transforms: Query<&Transform, Without<Homing>>,
    mut homing_query: Query<(&mut Transform, &Homing, &CastBy, &DynamicTarget)>,
    modifiers: Query<&ModifierStack>,
    time: Res<Time>,
) {
    for (mut transform, homing, cast_info, target) in &mut homing_query {
        let turn_speed = apply_modifier_if_present(modifiers.get(cast_info.entity).ok(), HOMING_STRENGTH_MODIFIER, homing.base_turn_speed);

        if let Some(target_pos) = target.target.and_then(|e| transforms.get(e).ok()).map(|t|t.translation) {
            // Calculate rotation needed
            let target_rotation = Transform::from_translation(transform.translation)
                .looking_at(target_pos, Vec3::Y)
                .rotation;

            // Slerp toward target rotation
            let max_rotation = turn_speed * time.delta_secs();
            transform.rotation = transform.rotation.slerp(target_rotation, max_rotation);
        }
    }
}

/// Observer system to handle collision damage
pub fn handle_collision_damage(
    trigger: Trigger<OnCollision>,
    damage_on_collision_query: Query<(&DamageOnCollision, &CastBy, &Transform, &MoveForward)>,
    mut damage_events: EventWriter<event::DamageEvent>,
    modifiers: Query<&ModifierStack>,
    target_transforms: Query<&Transform, Without<DamageOnCollision>>,
) {
    let collider_entity = trigger.target();
    let collision_event = trigger.event();
    
    if let Ok((damage_component, cast_info, collider_transform, movement)) = damage_on_collision_query.get(collider_entity) {
        if let Ok(target_transform) = target_transforms.get(collision_event.target) {
            
            /*damage_events.write(event::DamageEvent {
                target: collision_event.target,
                source: Some(cast_info.caster),
                damage: apply_modifier_if_present(modifiers.get(cast_info.caster).ok(), DAMAGE_MODIFIER, damage_component.base_damage) as i32,
                position: target_transform.translation,
                impact_velocity: None,
            });*/
        }
    }
}

/// Observer system to handle despawning on collision
pub fn handle_collision_despawn(
    trigger: Trigger<OnCollision>,
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
            handle_forward_movement,
            handle_homing_movement,
        )
    );
    app.add_observer(handle_collision_damage);
    app.add_observer(handle_collision_despawn);
}