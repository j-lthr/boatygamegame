use std::f32;

use crate::{
    ability::{CastDynamicAbility, DynamicAbility},
    common::{Faction, Health},
    enemy::spawn::SpawnInfo,
    event::DamageEvent,
    utils::normal_dist_1d,
};
use avian3d::prelude::*;
use bevy::prelude::*;

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
    look_at_target: bool,
}

impl FollowTarget {
    pub fn ranged(preferred_distance: f32, rotation_speed: f32) -> Self {
        Self {
            mode: FollowMovementMode::Ranged {
                preferred_distance,
                rotation_speed,
            },
            look_at_target: true,
        }
    }

    pub fn melee() -> Self {
        Self {
            mode: FollowMovementMode::ToMeleeRange,
            look_at_target: true,
        }
    }
}

#[derive(Event)]
pub struct MoveEvent {
    velocity: Vec3,
    entity: Entity,
    rotation: Option<Quat>,
}

pub fn handle_follow_target(
    follower_query: Query<(Entity, &FollowTarget, &Transform, &SpawnInfo)>,
    target_query: Query<&Transform>,
    mut move_events: EventWriter<MoveEvent>,
    time: Res<Time>,
) {
    for (follower_entity, follow_target, follower_transform, spawn_info) in follower_query {
        if let Ok(target_transform) = target_query.get(spawn_info.target) {
            let delta = target_transform.translation - follower_transform.translation;

            move_events.write(MoveEvent {
                velocity: match follow_target.mode {
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
                rotation: if follow_target.look_at_target {
                    Some(
                        follower_transform
                            .looking_at(target_transform.translation, Vec3::Y)
                            .rotation,
                    )
                } else {
                    None
                },
            });
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct FirstOrderMovement {
    pub speed: f32,
    pub jitter: f32,
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

            if let Some(rotation) = move_event.rotation {
                transform.rotation = rotation;
            }
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct ForceMovement {
    pub speed: f32,
    pub omega: f32,
    pub gamma: f32,
}

pub fn handle_force_movement(
    mut move_events: EventReader<MoveEvent>,
    mut query: Query<(
        &Transform,
        &ForceMovement,
        &mut ExternalForce,
        &mut ExternalTorque,
        &LinearVelocity,
        &AngularVelocity,
    )>,
    time: Res<Time>,
) {
    for move_event in move_events.read() {
        if let Ok((transform, movement, mut force, mut torque, vel, ang_vel)) =
            query.get_mut(move_event.entity)
        {
            let target_vel = movement.speed * move_event.velocity;

            force.apply_force(movement.omega * (target_vel - vel.0));

            force.persistent = false;

            // --- Angular Movement (New) ---
            if let Some(target_rotation) = move_event.rotation {
                // 1. Calculate the difference between target and current rotation
                let mut delta_rot = target_rotation * transform.rotation.inverse();

                // 2. "Shortest Path" check:
                // Quaternions represent the same rotation at q and -q.
                // If w is negative, we are taking the "long way" around. Flip it to take the shortest path.
                if delta_rot.w < 0.0 {
                    delta_rot = -delta_rot;
                }

                // 3. Decompose into Axis-Angle (The "Lie Algebra" magic part)
                // This gives us a vector where direction is the axis to spin around,
                // and magnitude is how much we need to spin (in radians).
                let angle_axis = delta_rot.to_scaled_axis();

                torque.apply_torque(movement.gamma * angle_axis);
                torque.persistent = false;
            }
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

pub fn single_ability_timed(
    mut commands: Commands,
    query: Query<(Entity, &mut SingleAbilityTimed, &SpawnInfo)>,
    mut events: EventWriter<CastDynamicAbility>,
    time: Res<Time>,
) {
    for (caster, mut sat, spawn_info) in query {
        sat.timer.tick(time.delta());

        if sat.timer.just_finished() {
            commands.entity(caster).trigger(
                CastDynamicAbility::at_caster(sat.ability.clone(), caster)
                    .with_target_entity(spawn_info.target),
            );
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct ContactDamage {
    pub damage: i32,
    pub radius: f32,
    pub cooldown: Timer,
    pub self_knockback: f32,
    pub self_damage: i32,
}

impl ContactDamage {
    pub fn new(damage: i32, radius: f32, cooldown_seconds: f32) -> Self {
        Self {
            damage,
            radius,
            cooldown: Timer::from_seconds(cooldown_seconds, TimerMode::Once),
            self_knockback: 0.0,
            self_damage: 0,
        }
    }

    pub fn with_self_knockback(mut self, knockback: f32) -> Self {
        self.self_knockback = knockback;
        self
    }

    pub fn with_self_damage(mut self, damage: i32) -> Self {
        self.self_damage = damage;
        self
    }
}

pub fn handle_contact_damage(
    mut contact_query: Query<(Entity, &mut ContactDamage, &mut Transform, &Faction, Option<&LinearVelocity>)>,
    target_query: Query<(Entity, &Transform, &Faction, &Health), Without<ContactDamage>>,
    mut damage_events: EventWriter<DamageEvent>,
    time: Res<Time>,
) {
    for (contact_entity, mut contact_damage, mut contact_transform, contact_faction, vel) in
        &mut contact_query
    {
        contact_damage.cooldown.tick(time.delta());

        if contact_damage.cooldown.finished() {
            for (target_entity, target_transform, target_faction, _) in &target_query {
                if contact_entity != target_entity && contact_faction != target_faction {
                    let distance = contact_transform
                        .translation
                        .distance(target_transform.translation);
                    if distance < contact_damage.radius {
                        info!("Contact Damage");

                        damage_events.write(DamageEvent {
                            target: target_entity,
                            source: Some(contact_entity),
                            damage: contact_damage.damage,
                            position: target_transform.translation,
                            impact_velocity: vel.map(|x|x.0),
                        });

                        if contact_damage.self_damage > 0 {
                            damage_events.write(DamageEvent {
                            target: contact_entity,
                            source: Some(contact_entity),
                            damage: contact_damage.self_damage,
                            position: target_transform.translation,
                            impact_velocity: vel.map(|x|x.0),
                        });
                        }

                        let knockback_direction = (contact_transform.translation
                            - target_transform.translation)
                            .normalize_or_zero();

                        contact_transform.translation +=
                            contact_damage.self_knockback * knockback_direction;

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

pub fn update_despawn_timer(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DespawnTimer, &Transform)>,
    time: Res<Time>,
) {
    for (entity, mut timer, _transform) in &mut query.iter_mut() {
        timer.timer.tick(time.delta());

        if timer.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            handle_follow_target,
            handle_kinematic_move_events,
            single_ability_timed,
            handle_contact_damage,
            update_despawn_timer,
            handle_force_movement,
        ),
    );
    app.add_event::<MoveEvent>();
}
