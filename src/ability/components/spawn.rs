use bevy::prelude::*;

use crate::ability::{CastInfo, components::subcast::SubCastInfo};
use crate::utils::{normal_dist_1d, normal_dist_2d};

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

#[derive(Component)]
pub struct RandomSpawnOffset {
    pub position_stddev: f32,
    pub rotation_stddev: f32,
}

pub fn handle_random_spawn_offset(
    mut commands: Commands,
    query: Query<(
        Entity,
        &mut Transform,
        &RandomSpawnOffset,
    )>,
) {
    for (entity, mut transform, radial_offset) in query {


        let new_position = transform.translation + normal_dist_2d(Vec2::ZERO, radial_offset.position_stddev).xxy().with_y(0.0);
        let new_rotation = Quat::from_axis_angle(Vec3::Y, normal_dist_1d(0.0,radial_offset.rotation_stddev));

        transform.look_at(new_position, Vec3::Y);
        transform.translation = new_position;

        commands.entity(entity).remove::<RadialSubCastOffset>();
    }
}

pub fn plugin(app: &mut bevy::app::App) {
    app.add_systems(
        Update,
        (handle_radial_sub_cast_offset, handle_spawn_at_cast_position).chain(),
    );
    app.add_systems(Update, (handle_spawn_at_target_position));
}
