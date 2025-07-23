use bevy::prelude::*;

use crate::ability::{CastInfo, components::subcast::SubCastInfo};
use crate::utils::{normal_dist_1d, normal_dist_2d};
use crate::enemy::spawn::{SpawnEnemyEvent, SpawnInfo};

#[derive(Component, Clone)]
pub struct SpawnAtCastPosition;

pub fn handle_spawn_at_cast_position(
    mut commands: Commands,
    query: Query<(Entity, &mut Transform, &CastInfo), With<SpawnAtCastPosition>>,
) {
    for (entity, mut transform, cast_info) in query {
        transform.translation = cast_info.cast_position;
        commands.entity(entity).remove::<SpawnAtCastPosition>();

        info!(
            "Spawned entity at cast position: {:?}",
            transform.translation
        );
    }
}

#[derive(Component, Clone)]
pub struct SpawnAtTargetPosition;

pub fn handle_spawn_at_target_position(
    mut commands: Commands,
    query: Query<(Entity, &mut Transform, &CastInfo), With<SpawnAtTargetPosition>>,
) {
    for (entity, mut transform, cast_info) in query {
        transform.translation = cast_info.target_position;
        commands.entity(entity).remove::<SpawnAtTargetPosition>();

        info!(
            "Spawned entity at target position: {:?}",
            transform.translation
        );
    }
}

#[derive(Clone)]
pub enum RadialSubCastType {
    TotalAngle(f32),
    AnglePerCast(f32),
}

#[derive(Component, Clone)]
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
    query: Query<(
        Entity,
        &mut Transform,
        &RadialSubCastOffset,
        &SubCastInfo,
        &CastInfo,
    )>,
) {
    for (entity, mut transform, radial_offset, subcast_info, cast_info) in query {
        let direction = cast_info.target_position - transform.translation;

        let total_angle = match radial_offset.ty {
            RadialSubCastType::TotalAngle(a) => a,
            RadialSubCastType::AnglePerCast(a) => a * subcast_info.num_casts() as f32,
        };

        let angle = direction.z.atan2(direction.x)
            + ((subcast_info.index() as f32 - (subcast_info.num_casts() - 1) as f32 / 2.0)
                / subcast_info.num_casts() as f32)
                * total_angle;

        let offset = Vec3::new(
            radial_offset.radius * angle.cos(),
            0.0,
            radial_offset.radius * angle.sin(),
        );

        let new_position = transform.translation + offset;

        transform.look_at(new_position, Vec3::Y);
        transform.translation = new_position;

        commands.entity(entity).remove::<RadialSubCastOffset>();
    }
}

#[derive(Component, Copy, Clone)]
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
    query: Query<(
        Entity,
        &mut Transform,
        &RandomSpawnOffset,
    )>,
) {
    for (entity, mut transform, random_spawn_offset) in query {


        let new_position = transform.translation + normal_dist_2d(Vec2::ZERO, random_spawn_offset.position_stddev).xxy().with_y(0.0);
        let new_rotation = Quat::from_axis_angle(Vec3::Y, normal_dist_1d(0.0, random_spawn_offset.rotation_stddev));

        transform.translation = new_position;
        transform.rotation *= new_rotation;

        commands.entity(entity).remove::<RandomSpawnOffset>();
    }
}

#[derive(Component, Clone)]
pub struct SpawnEnemyAtCastPosition {
    pub enemy_id: String,
}

impl SpawnEnemyAtCastPosition {
    pub fn new(enemy_id: impl Into<String>) -> Self {
        Self {
            enemy_id: enemy_id.into(),
        }
    }
}

pub fn handle_spawn_enemy_at_cast_position(
    mut commands: Commands,
    mut spawn_events: EventWriter<SpawnEnemyEvent>,
    query: Query<(Entity, &Transform, &CastInfo, &SpawnEnemyAtCastPosition)>,
    spawn_info_query: Query<&SpawnInfo>,
) {
    for (entity, transform, cast_info, spawn_enemy) in query {
        // Get the target from the original caster (summoner)
        if let Ok(spawn_info) = spawn_info_query.get(cast_info.caster) {
            spawn_events.write(SpawnEnemyEvent {
                enemy_id: spawn_enemy.enemy_id.clone(),
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
            handle_spawn_at_target_position, 
            handle_radial_sub_cast_offset, 
            handle_spawn_at_cast_position,
            handle_spawn_enemy_at_cast_position,
            handle_random_spawn_offset,
        ).chain(),
    );
}
