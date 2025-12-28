use std::f32;

use avian3d::prelude::ShapeCaster;
use bevy::{ecs::spawn, prelude::*};
use bevy::reflect::TupleFieldIter;
use crate::{ability::{CastDynamicAbility, DynamicAbility}, common::Faction, enemy::spawn::SpawnInfo, event::DamageEvent, utils::normal_dist_1d};

#[derive(Clone, Debug)]
pub enum FollowMovementMode {
    ToMeleeRange,
    Ranged {
        preferred_distance: f32,
        rotation_speed: f32,
    },
}

#[derive(Component, Clone, Debug)]
pub struct FollowTarget {
    pub mode: FollowMovementMode,
}

impl FollowTarget {
    pub fn ranged(preferred_distance: f32, rotation_speed: f32) -> Self {
        Self {
            mode: FollowMovementMode::Ranged {
                preferred_distance,
                rotation_speed,
            }
        }
    }
}

#[derive(Event)]
pub struct MoveEvent {
    velocity: Vec3,
    entity: Entity,
}

#[derive(Component, Clone, Debug)]
pub struct FirstOrderMovement {
    pub speed: f32,
    pub jitter: f32,
}

pub fn handle_follow_movement(
    follower_query: Query<(Entity, &FollowTarget, &Transform, &SpawnInfo)>,
    target_query: Query<&Transform>,
    mut move_events: EventWriter<MoveEvent>,
    time: Res<Time>,
) {
    for (follower_entity, follower_movement, follower_transform, spawn_info) in follower_query {
        if let Ok(target_transform) = target_query.get(spawn_info.target) {
            let delta = target_transform.translation - follower_transform.translation;

            move_events.write(MoveEvent {
                velocity: match follower_movement.mode {
                    FollowMovementMode::ToMeleeRange => delta,
                    FollowMovementMode::Ranged {
                        preferred_distance,
                        rotation_speed,
                    } => {
                        let mut optimal_position =
                            target_transform.translation - delta.normalize() * preferred_distance;

                        optimal_position += Quat::from_rotation_y(0.5 * f32::consts::PI)
                            * (optimal_position - target_transform.translation)
                            * time.delta_secs()
                            * rotation_speed;

                        optimal_position - follower_transform.translation
                    }
                }
                .normalize(),
                entity: follower_entity,
            });
        }
    }
}

pub fn handle_kinematic_move_events(
    mut move_events: EventReader<MoveEvent>,
    mut query: Query<(&mut Transform, &FirstOrderMovement)>,
    time: Res<Time>,
) {
    for move_event in move_events.read() {
        if let Ok((mut transform, movement)) = query.get_mut(move_event.entity) {
            transform.translation += move_event.velocity * time.delta_secs() * movement.speed;

            transform.translation.x += normal_dist_1d(0.0, 1.0) * movement.jitter;
            transform.translation.z += normal_dist_1d(0.0, 1.0) * movement.jitter;
        }
    }
}


#[derive(Component, Clone, Debug)]
pub struct SingleAbilityTimed {
    ability: DynamicAbility,
    timer: Timer, 
}

impl SingleAbilityTimed {
    pub fn new(ability: DynamicAbility, interval: f32) -> Self {
        Self {
            ability,
            timer: Timer::from_seconds(interval, TimerMode::Repeating),
        }
    }
}

pub fn single_ability_timed(query: Query<(Entity, &mut SingleAbilityTimed, &SpawnInfo)>, mut events: EventWriter<CastDynamicAbility>, time: Res<Time>) {
    for (caster, mut sat, spawn_info) in query {
        sat.timer.tick(time.delta());

        if sat.timer.just_finished() {
            events.write(CastDynamicAbility::at_caster(sat.ability.clone(), caster).with_target_entity(spawn_info.target));
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct ContactDamage {
    pub damage: i32,
    pub radius: f32,
    pub cooldown: Timer,
    pub self_knockback: f32,
}

impl ContactDamage {
    pub fn new(damage: i32, radius: f32, cooldown_seconds: f32) -> Self {
        Self {
            damage,
            radius,
            cooldown: Timer::from_seconds(cooldown_seconds, TimerMode::Once),
            self_knockback: 0.0,
        }
    }

    pub fn with_self_knockback(mut self, knockback: f32) -> Self {
        self.self_knockback = knockback;
        self
    }
}

pub fn handle_contact_damage(
    mut contact_query: Query<(Entity, &mut ContactDamage, &mut Transform, &Faction)>,
    target_query: Query<(Entity, &Transform, &Faction), Without<ContactDamage>>,
    mut damage_events: EventWriter<DamageEvent>,
    time: Res<Time>,
) {
    for (contact_entity, mut contact_damage, mut contact_transform, contact_faction) in &mut contact_query {
        contact_damage.cooldown.tick(time.delta());
        
        if contact_damage.cooldown.finished() {
            for (target_entity, target_transform, target_faction) in &target_query {
                if contact_entity != target_entity && contact_faction != target_faction {
                    let distance = contact_transform.translation.distance(target_transform.translation);
                    if distance < contact_damage.radius {
                        damage_events.write(DamageEvent {
                            target: target_entity,
                            source: Some(contact_entity),
                            damage: contact_damage.damage,
                            position: target_transform.translation,
                            impact_velocity: None,
                        });

                        let knockback_direction = (contact_transform.translation - target_transform.translation).normalize_or_zero();

                        contact_transform.translation += contact_damage.self_knockback * knockback_direction;
                        
                        contact_damage.cooldown.reset();
                        break;
                    }
                }
            }
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct DespawnTimer {
    pub timer: Timer,
}

impl DespawnTimer {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

pub fn update_despawn_timer(mut commands: Commands, mut query: Query<(Entity, &mut DespawnTimer, &Transform)>, time: Res<Time>) {

    for (entity, mut timer, transform) in &mut query.iter_mut() {
        timer.timer.tick(time.delta());

        if timer.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}


pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (handle_follow_movement, handle_kinematic_move_events, single_ability_timed, handle_contact_damage, update_despawn_timer),
    );
    app.add_event::<MoveEvent>();
}
