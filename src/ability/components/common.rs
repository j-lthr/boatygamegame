use bevy::prelude::*;

use crate::ability::CastInfo;


#[derive(Component, Clone)]
pub struct AttachToCaster;

pub fn handle_attach_to_caster(
    caster_query: Query<&Transform, Without<AttachToCaster>>,
    query: Query<(&mut Transform, &CastInfo), With<AttachToCaster>>,
) {
    for (mut transform, cast_info) in query {
        transform.translation = caster_query.get(cast_info.caster)
            .map(|caster_transform| caster_transform.translation)
            .unwrap_or(Vec3::ZERO);
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            handle_attach_to_caster,
        )
    );
}