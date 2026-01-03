use super::events::OnActiveDespawn;
use bevy::prelude::*;

use super::projectile::InitialVelocity;
use crate::ability::{resolve_target, CastBy, IntendedTarget};
use crate::common::{Faction, Targetable};
use crate::modifiers::*;

#[derive(Component, Clone)]
pub struct AttachToCaster;

pub fn handle_attach_to_caster(
    caster_query: Query<&Transform, Without<AttachToCaster>>,
    query: Query<(&mut Transform, &CastBy), With<AttachToCaster>>,
) {
    for (mut transform, cast_by) in query {
        transform.translation = caster_query
            .get(cast_by.entity)
            .map(|caster_transform| caster_transform.translation)
            .unwrap_or(Vec3::ZERO);
    }
}

#[derive(Copy, Clone, Debug)]
pub enum LifetimeSource {
    Fixed {
        base_duration: f32,
        modified_by: Option<ModifierID>,
    },
    Dynamic,
}

#[derive(Copy, Clone, Debug)]
pub enum LifetimePhase {
    JustSpawned(LifetimeSource),
    Alive { duration: f32, elapsed: f32 },
    JustDied,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Lifetime {
    phase: LifetimePhase,
}

impl Lifetime {
    pub fn fixed(base_duration: f32) -> Self {
        Self {
            phase: LifetimePhase::JustSpawned(LifetimeSource::Fixed {
                base_duration,
                modified_by: None,
            }),
        }
    }

    pub fn fixed_with_modifier(base_duration: f32, modifier: ModifierID) -> Self {
        Self {
            phase: LifetimePhase::JustSpawned(LifetimeSource::Fixed {
                base_duration,
                modified_by: Some(modifier),
            }),
        }
    }

    pub fn dynamic() -> Self {
        Self {
            phase: LifetimePhase::JustSpawned(LifetimeSource::Dynamic),
        }
    }

    pub fn just_spawned(&self) -> bool {
        if let LifetimePhase::JustSpawned(_) = self.phase {
            true
        } else {
            false
        }
    }

    pub fn just_died(&self) -> bool {
        if let LifetimePhase::JustDied = self.phase {
            true
        } else {
            false
        }
    }

    pub fn elapsed_ratio(&self) -> f32 {
        match self.phase {
            LifetimePhase::JustSpawned(_) => 0.0,
            LifetimePhase::Alive { duration, elapsed } => elapsed / duration,
            LifetimePhase::JustDied => 1.0,
        }
    }

    pub fn set_dynamic_duration(&mut self, duration: f32) -> Result {
        match self.phase {
            LifetimePhase::JustSpawned(LifetimeSource::Dynamic) => {
                self.phase = LifetimePhase::Alive {
                    duration,
                    elapsed: 0.0,
                };
                Ok(())
            }
            _ => Err("Can only set duration for dynamic lifetime sources".into()),
        }
    }
}

pub fn handle_lifetime(
    mut commands: Commands,
    query: Query<(Entity, &mut Lifetime, &CastBy)>,
    modifiers: Query<&ModifierStack>,
    time: Res<Time>,
) {
    for (entity, mut lifetime, cast_by) in query {
        match &mut lifetime.phase {
            LifetimePhase::JustSpawned(lifetime_source) => {
                if let LifetimeSource::Fixed {
                    base_duration,
                    modified_by,
                } = lifetime_source
                {
                    let duration = if let Some(modified_by) = modified_by {
                        apply_modifier_if_present(
                            modifiers.get(cast_by.entity).ok(),
                            *modified_by,
                            *base_duration,
                        )
                    } else {
                        *base_duration
                    };

                    lifetime.phase = LifetimePhase::Alive {
                        duration,
                        elapsed: 0.0,
                    };
                }
            }
            LifetimePhase::Alive { elapsed, duration } => {
                *elapsed += time.delta_secs();

                if elapsed > duration {
                    lifetime.phase = LifetimePhase::JustDied
                }
            }
            LifetimePhase::JustDied => {
                commands.entity(entity).trigger(OnActiveDespawn);
                commands.entity(entity).despawn();
            }
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct LifetimeFromCursor;

pub fn handle_lifetime_from_cursor(
    mut query: Query<
        (&mut Lifetime, &IntendedTarget, &Transform, &InitialVelocity),
        Added<LifetimeFromCursor>,
    >,
    mut transforms: Query<&Transform>,
) -> Result {

    for (mut lifetime, target, transform, linear_movement) in query.iter_mut() {

        let position = resolve_target(transforms.transmute_lens(), *target)?.position;
        let distance = transform.translation.distance(position);
        let duration = distance / linear_movement.base_speed;

        lifetime.set_dynamic_duration(duration)?;
    }

    Ok(())
}

#[derive(Component, Clone, Debug)]
pub struct DynamicTarget {
    pub target: Option<Entity>,
}

impl DynamicTarget {
    pub fn new() -> Self {
        Self { target: None }
    }
}

#[derive(Component, Clone, Debug)]
pub struct SelectNearestTargetOnSpawn {
    max_distance: f32,
}

impl SelectNearestTargetOnSpawn {
    pub fn new(max_distance: f32) -> Self {
        Self { max_distance }
    }
}

fn handle_select_nearest_target_on_spawn(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, Option<&Faction>, &mut DynamicTarget, &IntendedTarget, &SelectNearestTargetOnSpawn)>,
    targets: Query<(Entity, &Transform, Option<&Faction>), (With<Targetable>, Without<SelectNearestTargetOnSpawn>)>,
    mut transforms: Query<&GlobalTransform>
) -> Result {
    for (entity, _transform, faction, mut dynamic_target, intended_target, selector) in query.iter_mut() {
        // Find nearest target
        let mut nearest_entity = None;
        let mut nearest_distance = f32::INFINITY;
        
        let target = resolve_target(transforms.transmute_lens(), *intended_target)?;
        
        for (target_entity, target_transform, target_faction) in targets.iter() {
            let distance = target.position.distance(target_transform.translation);
            if distance < selector.max_distance && distance < nearest_distance && (faction.is_none() || target_faction.is_none() || faction != target_faction) {
                nearest_distance = distance;
                nearest_entity = Some(target_entity);
            }
        }
        
        dynamic_target.target = nearest_entity;
        commands.entity(entity).remove::<SelectNearestTargetOnSpawn>();
    }
    
    Ok(())
}


pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            handle_attach_to_caster,
            handle_lifetime,
            handle_lifetime_from_cursor,
            handle_select_nearest_target_on_spawn,
        ),
    );
}

