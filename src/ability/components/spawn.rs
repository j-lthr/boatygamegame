use bevy::prelude::*;

use crate::ability::{components::subcast::SubCastInfo, CastInfo};

#[derive(Component, Clone)]
pub struct SpawnAtCastPosition;

pub fn handle_spawn_at_cast_position(
    mut commands: Commands,
    query: Query<(Entity, &mut Transform, &CastInfo), With<SpawnAtCastPosition>>,
) {
    for (entity, mut transform, cast_info) in query {
        transform.translation = cast_info.cast_position;
        commands.entity(entity).remove::<SpawnAtCastPosition>();

        info!("Spawned entity at cast position: {:?}", transform.translation);
    }
}


#[derive(Component, Clone)]
pub struct RadialSubCastOffset {
    /// Radius from the center of the cast position
    pub radius: f32,
    /// Total angle spread of the sub-casts in radians
    /// This is the angle between the first and last sub-cast in the radial spread
    pub spread_angle: f32,
}

impl RadialSubCastOffset {
    pub fn new(radius: f32, spread_angle: f32) -> Self {
        Self { radius, spread_angle }
    }

    pub fn from_degrees(radius: f32, spread_degrees: f32) -> Self {
        Self {
            radius,
            spread_angle: spread_degrees.to_radians(),
        }
    }

    pub fn from_radius_360(radius: f32) -> Self {
        Self {
            radius,
            spread_angle: std::f32::consts::PI * 2.0, // Full circle
        }
    }
}

pub fn handle_radial_sub_cast_offset(
    mut commands: Commands,
    query: Query<(Entity, &mut Transform, &RadialSubCastOffset, &SubCastInfo, &CastInfo)>,
) {
    for (entity, mut transform, radial_offset, subcast_info, cast_info) in query {

        let direction = cast_info.target_position - transform.translation;

        let angle = direction.z.atan2(direction.x) + ((subcast_info.index() - subcast_info.num_casts() / 2) as f32 / subcast_info.num_casts() as f32) * radial_offset.spread_angle;
        let offset = Vec3::new(radial_offset.radius * angle.cos(), 0.0, radial_offset.radius * angle.sin());

        let new_position = transform.translation + offset;

        info!("Looking at new position: {:?}", new_position);

        transform.look_at(new_position, Vec3::Y);
        transform.translation = new_position;

        commands.entity(entity).remove::<RadialSubCastOffset>();
    }
}

pub fn plugin(app: &mut bevy::app::App) {
    app.add_systems(Update, (handle_radial_sub_cast_offset, handle_spawn_at_cast_position).chain());
}