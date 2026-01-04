use bevy::prelude::*;

use crate::ability::{CastBy, components::subcast::SubCastInfo};
use crate::enemy::registry::Enemy;
use crate::enemy::spawn::{SpawnEnemyEvent, SpawnInfo};
use crate::utils::{normal_dist_1d, normal_dist_2d};

#[derive(Clone, Debug)]
pub enum RadialSubCastType {
    TotalAngle(f32),
    AnglePerCast(f32),
}

#[derive(Component, Clone, Debug)]
pub struct RadialSubCastOffset {
    /// Radius from the center of the cast position
    pub radius: f32,
    /// Total angle spread of the sub-casts in radians
    /// This is the angle between the first and last sub-cast in the radial spread
    pub ty: RadialSubCastType,
}

impl RadialSubCastOffset {
    pub fn new(radius: f32, spread_angle: f32) -> Self {
        Self {
            radius,
            ty: RadialSubCastType::TotalAngle(spread_angle),
        }
    }

    pub fn from_degrees(radius: f32, spread_degrees: f32) -> Self {
        Self::new(radius, spread_degrees.to_radians())
    }

    pub fn from_radius_360(radius: f32) -> Self {
        Self::new(radius, 2.0 * std::f32::consts::PI)
    }

    pub fn from_degrees_per_cast(radius: f32, degrees_per_cast: f32) -> Self {
        Self {
            radius,
            ty: RadialSubCastType::AnglePerCast(degrees_per_cast.to_radians()),
        }
    }
}

pub fn handle_radial_sub_cast_offset(
    mut commands: Commands,
    query: Query<(Entity, &mut Transform, &RadialSubCastOffset, &SubCastInfo)>,
) {
    for (entity, mut transform, radial_offset, subcast_info) in query {
        let total_angle = match radial_offset.ty {
            RadialSubCastType::TotalAngle(a) => a,
            RadialSubCastType::AnglePerCast(a) => a * subcast_info.num_casts() as f32,
        };

        let angle = ((subcast_info.index() as f32 - (subcast_info.num_casts() - 1) as f32 / 2.0)
            / subcast_info.num_casts() as f32)
            * total_angle;

        transform.rotation = Quat::from_rotation_y(angle) * transform.rotation;
        let delta_p = transform.forward() * radial_offset.radius;
        transform.translation += delta_p;

        commands.entity(entity).remove::<RadialSubCastOffset>();
    }
}

#[derive(Component, Copy, Clone, Debug)]
pub struct RandomSpawnOffset {
    pub position_stddev: f32,
    pub rotation_stddev: f32,
}

impl RandomSpawnOffset {
    pub fn new(position_stddev: f32, rotation_stddev: f32) -> Self {
        Self {
            position_stddev,
            rotation_stddev,
        }
    }
}

pub fn handle_random_spawn_offset(
    mut commands: Commands,
    query: Query<(Entity, &mut Transform, &RandomSpawnOffset)>,
) {
    for (entity, mut transform, random_spawn_offset) in query {
        let new_position = transform.translation
            + normal_dist_2d(Vec2::ZERO, random_spawn_offset.position_stddev)
                .xxy()
                .with_y(0.0);
        let new_rotation = Quat::from_axis_angle(
            Vec3::Y,
            normal_dist_1d(0.0, random_spawn_offset.rotation_stddev),
        );

        transform.translation = new_position;
        transform.rotation *= new_rotation;

        commands.entity(entity).remove::<RandomSpawnOffset>();
    }
}

#[derive(Component, Clone, Debug)]
pub struct SpawnEnemyAtCastPosition {
    pub enemy: Enemy,
}

impl SpawnEnemyAtCastPosition {
    pub fn new(enemy: Enemy) -> Self {
        Self { enemy }
    }
}

pub fn handle_spawn_enemy_at_cast_position(
    mut commands: Commands,
    mut spawn_events: EventWriter<SpawnEnemyEvent>,
    query: Query<(Entity, &Transform, &CastBy, &SpawnEnemyAtCastPosition)>,
    spawn_info_query: Query<&SpawnInfo>,
) {
    for (entity, transform, cast_by, spawn_enemy) in query {
        // Get the target from the original caster (summoner)
        if let Ok(spawn_info) = spawn_info_query.get(cast_by.entity) {
            spawn_events.write(SpawnEnemyEvent {
                enemy: spawn_enemy.enemy.clone(),
                position: transform.translation,
                target: spawn_info.target,
            });
        }

        commands.entity(entity).despawn();
    }
}

pub fn plugin(app: &mut bevy::app::App) {
    app.add_systems(
        Update,
        (
            handle_radial_sub_cast_offset,
            handle_spawn_enemy_at_cast_position,
            handle_random_spawn_offset,
        )
            .chain(),
    );
}
