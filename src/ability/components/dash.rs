use avian3d::prelude::*;
use bevy::prelude::*;

use crate::ability::{
    CastBy,
    components::common::{Lifetime, handle_lifetime},
};

#[derive(Clone, Debug)]
pub enum DashState {
    Start,
    InProgress(f32),
    Stop,
}

#[derive(Clone, Debug, Component)]
pub struct TransportCaster;

impl TransportCaster {
    pub fn new() -> TransportCaster {
        TransportCaster
    }
}

pub fn handle_transport_caster(
    ability_query: Query<(Entity, &Transform, &mut TransportCaster, &CastBy, &Lifetime)>,
    mut caster_query: Query<&mut Transform, Without<TransportCaster>>,
    mut commands: Commands,
) {
    for (entity, dash_transform, mut transport_caster, caster, lifetime) in ability_query {
        if let Ok(mut caster_transform) = caster_query.get_mut(caster.entity) {
            caster_transform.translation = dash_transform.translation;
        }
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update, handle_transport_caster.before(handle_lifetime));
}
