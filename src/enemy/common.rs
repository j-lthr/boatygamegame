use std::f32;

use bevy::prelude::*;

use crate::utils::normal_dist_1d;

pub enum FollowMovementMode {
    ToMeleeRange,
    Ranged {
        preferred_distance: f32,
        rotation_speed: f32,
    },
}

#[derive(Component)]
pub struct FollowTarget {
    pub target: Entity,
    pub mode: FollowMovementMode,
}

#[derive(Event)]
pub struct MoveEvent {
    velocity: Vec3,
    entity: Entity,
}

#[derive(Component)]
pub struct FirstOrderMovement {
    pub speed: f32,
    pub jitter: f32,
}

pub fn handle_follow_movement(
    follower_query: Query<(Entity, &FollowTarget, &Transform)>,
    target_query: Query<&Transform>,
    mut move_events: EventWriter<MoveEvent>,
    time: Res<Time>,
) {
    for (follower_entity, follower_movement, follower_transform) in follower_query {
        if let Ok(target_transform) = target_query.get(follower_movement.target) {
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

pub fn register(app: &mut App) {
    app.add_systems(
        Update,
        (handle_follow_movement, handle_kinematic_move_events),
    );
    app.add_event::<MoveEvent>();
}
