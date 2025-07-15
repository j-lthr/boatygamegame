use std::time::Duration;

use bevy::audio::Volume;
use bevy::prelude::*;

use crate::ability::components::events::{OnActiveDespawn, OnCollision};
use crate::ability::CastInfo;
use crate::common;
use crate::common::Faction;
use crate::event;
use crate::fx;
use crate::modifiers::*;

// Component for linear movement
#[derive(Component, Clone)]
pub struct LinearMovement {
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

/// System to move entities with LinearMovement
pub fn handle_linear_movement(
    mut movement_query: Query<(&mut Transform, &LinearMovement, &CastInfo)>,
    modifiers: Query<&ModifierStack>,
    time: Res<Time>,
) {
    for (mut transform, movement, cast_info) in &mut movement_query {
        let speed = apply_modifier_if_present(modifiers.get(cast_info.caster).ok(), PROJECTILE_SPEED_MODIFIER, movement.base_speed);
        let fwd = transform.forward();
        transform.translation += fwd * speed * time.delta_secs();
    }
}

/// System for collision detection with SimpleCollider
pub fn handle_simple_collision(
    mut commands: Commands,
    collider_query: Query<(Entity, &Transform, &SimpleCollider, &CastInfo)>,
    target_query: Query<(Entity, &Transform, &common::Living), Without<SimpleCollider>>,
    faction_query: Query<&Faction>,
) {
    for (collider_entity, collider_transform, collider, cast_info) in collider_query {
        for (entity, target_transform, _living_opt) in &target_query {
            // Skip self-damage
            if entity == cast_info.caster {
                continue;
            }

            if let (Ok(source_faction), Ok(target_faction)) = (
                faction_query.get(cast_info.caster),
                faction_query.get(entity),
            ) {
                if source_faction == target_faction {
                    continue;
                }
            }

            let distance = collider_transform
                .translation
                .distance(target_transform.translation);

            if distance < collider.radius {
                commands.entity(collider_entity).trigger(OnCollision { target: entity });
                break;
            }
        }
    }
}

/// Observer system to handle collision damage
pub fn handle_collision_damage(
    trigger: Trigger<OnCollision>,
    damage_on_collision_query: Query<(&DamageOnCollision, &CastInfo, &Transform, &LinearMovement)>,
    mut damage_events: EventWriter<event::DamageEvent>,
    modifiers: Query<&ModifierStack>,
    target_transforms: Query<&Transform, Without<DamageOnCollision>>,
) {
    let collider_entity = trigger.target();
    let collision_event = trigger.event();
    
    if let Ok((damage_component, cast_info, collider_transform, movement)) = damage_on_collision_query.get(collider_entity) {
        if let Ok(target_transform) = target_transforms.get(collision_event.target) {
            
            damage_events.write(event::DamageEvent {
                target: collision_event.target,
                source: Some(cast_info.caster),
                damage: apply_modifier_if_present(modifiers.get(cast_info.caster).ok(), DAMAGE_MODIFIER, damage_component.base_damage) as i32,
                position: target_transform.translation,
                impact_velocity: None,
            });
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
            handle_linear_movement,
            handle_simple_collision,
        )
    );
    app.add_observer(handle_collision_damage);
    app.add_observer(handle_collision_despawn);
}