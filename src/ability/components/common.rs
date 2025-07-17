use super::events::OnActiveDespawn;
use bevy::prelude::*;

use super::projectile::LinearMovement;
use crate::ability::CastInfo;
use crate::ability::components::events::OnSpawn;
use crate::modifiers::*;

#[derive(Component, Clone)]
pub struct AttachToCaster;

pub fn handle_attach_to_caster(
    caster_query: Query<&Transform, Without<AttachToCaster>>,
    query: Query<(&mut Transform, &CastInfo), With<AttachToCaster>>,
) {
    for (mut transform, cast_info) in query {
        transform.translation = caster_query
            .get(cast_info.caster)
            .map(|caster_transform| caster_transform.translation)
            .unwrap_or(Vec3::ZERO);
    }
}

#[derive(Copy, Clone)]
pub enum LifetimeSource {
    Fixed {
        base_duration: f32,
        modified_by: Option<ModifierID>,
    },
    Dynamic,
}

#[derive(Copy, Clone)]
pub enum LifetimePhase {
    JustSpawned(LifetimeSource),
    Alive { duration: f32, elapsed: f32 },
    JustDied,
}

#[derive(Component, Clone, Copy)]
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

    pub fn set_dynamic_duration(&mut self, duration: f32) -> Result<(), &'static str> {
        match self.phase {
            LifetimePhase::JustSpawned(LifetimeSource::Dynamic) => {
                self.phase = LifetimePhase::Alive {
                    duration,
                    elapsed: 0.0,
                };
                Ok(())
            }
            _ => Err("Can only set duration for dynamic lifetime sources"),
        }
    }
}

pub fn handle_lifetime(
    mut commands: Commands,
    query: Query<(Entity, &mut Lifetime, &CastInfo)>,
    modifiers: Query<&ModifierStack>,
    time: Res<Time>,
) {
    for (entity, mut lifetime, cast_info) in query {
        match &mut lifetime.phase {
            LifetimePhase::JustSpawned(lifetime_source) => {
                if let LifetimeSource::Fixed {
                    base_duration,
                    modified_by,
                } = lifetime_source
                {
                    let duration = if let Some(modified_by) = modified_by {
                        apply_modifier_if_present(
                            modifiers.get(cast_info.caster).ok(),
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
        (&mut Lifetime, &CastInfo, &Transform, &LinearMovement),
        Added<LifetimeFromCursor>,
    >,
) -> Result {

    for (mut lifetime, cast_info, transform, linear_movement) in query.iter_mut() {
        let distance = transform.translation.distance(cast_info.target_position);
        let duration = distance / linear_movement.base_speed;
        lifetime.set_dynamic_duration(duration)?;
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
        ),
    );
}

