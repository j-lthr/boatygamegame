use super::*;
use crate::fx;
use crate::init::DespawnOnReset;
use crate::projectile;
use crate::utils::normal_dist_1d;
use bevy::audio::Volume;
use std::time::Duration;


#[derive(Clone)]
pub struct DynamicAbility {
    entity_commands: Arc<Vec<Box<dyn EntityCommand>>>,
}

impl DynamicAbility {
    pub fn from_bundle(bundle: impl Bundle) -> Self {
        Self {
            entity_commands: Arc::new(vec!(Box::new(|mut entity: EntityWorldMut| {
                entity.insert(bundle);
            })))
        }
    }
}

#[derive(Copy, Clone, Component)]
pub struct CastInfo {
    pub caster: Entity,
    pub target_position: Vec3,
    pub target_entity: Option<Entity>,
}

pub fn cast_dynamic_ability(
    mut cast_events: EventReader<CastEvent<DynamicAbility>>,
    mut commands: Commands,
) {
    for event in cast_events.read() {
        let mut entity_commands = commands.spawn(event.params);

        for entity_command in event.ability.entity_commands {
            entity_commands.queue(entity_command);
        }
    }
}

impl Ability for DynamicAbility {
    type CastParams = CastInfo;

    fn add_systems(app: &mut bevy::app::App) {
        app.add_systems(Update, cast_dynamic_ability);
    }
}
