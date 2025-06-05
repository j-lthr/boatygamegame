use bevy::prelude::*;

use super::*;

#[derive(Clone)]
pub struct Dash {
    pub range: f32,
}

#[derive(Copy, Clone)]
pub enum DashParams {
    Directional(Vec3),
    ToPosition(Vec3)
}

pub fn cast_dash(mut cast_events: EventReader<CastEvent<Dash>>, mut caster_query: Query<&mut Transform>) {
    for cast_event in cast_events.read() {
        if let Ok(mut caster_transform) = caster_query.get_mut(cast_event.caster) {
            let delta = match cast_event.params {
                DashParams::Directional(direction) => {
                    direction.normalize() * cast_event.ability.range
                },
                DashParams::ToPosition(target_position) => {
                    (target_position - caster_transform.translation).clamp_length_max(cast_event.ability.range)    
                },
            };

            caster_transform.translation += delta;
        }
    }
}

impl Ability for Dash {
    type CastParams = DashParams;

    fn add_systems(app: &mut bevy::app::App) {
        app.add_systems(Update, cast_dash);
    }
}