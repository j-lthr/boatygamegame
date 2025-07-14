use std::time::Duration;

use bevy::audio::Volume;
use bevy::prelude::*;

use crate::ability::CastInfo;
use crate::common;
use crate::common::Faction;
use crate::event;
use crate::fx;
use crate::modifiers::apply_modifier_if_present;
use crate::modifiers::ModifierID;
use crate::modifiers::ModifierStack;
use crate::modifiers::DAMAGE_MODIFIER;

// Component for projectiles
#[derive(Component, Clone)]
pub struct Projectile {
    pub base_speed: f32,
    pub base_damage: f32,
}

pub fn collide(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Transform, &Projectile, &CastInfo)>,
    target_query: Query<(Entity, &mut Transform, &common::Living), Without<Projectile>>,
    mut shoot_sounds: ResMut<Assets<fx::fm::FMSound>>,
    mut damage_events: EventWriter<event::DamageEvent>,
    modifiers: Query<&ModifierStack>,
    faction_query: Query<&Faction>,
) {
    for (projectile_entity, projectile_transform, projectile, cast_info) in projectile_query {
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

            let distance = projectile_transform
                .translation
                .distance(target_transform.translation);

            if distance < 1.0 {

                commands.entity(projectile_entity).despawn();

                let projectile_velocity = projectile_transform.forward() * projectile.base_speed;

                damage_events.write(event::DamageEvent {
                    target: entity,
                    source: Some(cast_info.caster),
                    damage: apply_modifier_if_present(modifiers.get(cast_info.caster).ok(), DAMAGE_MODIFIER, projectile.base_damage) as i32,
                    position: target_transform.translation,
                    impact_velocity: Some(projectile_velocity),
                });

                break;
            }
        }
    }
}

/// System to move projectiles
pub fn handle_movement(
    mut projectile_query: Query<(&mut Transform, &Projectile)>,
    time: Res<Time>,
) {
    for (mut transform, projectile) in &mut projectile_query {
        let fwd = transform.forward();
        transform.translation += fwd * projectile.base_speed * time.delta_secs();
    }
}


pub fn plugin(app: &mut bevy::app::App) {
    app.add_systems(
        Update,
        (
            collide,
            handle_movement,
        )
    );
}