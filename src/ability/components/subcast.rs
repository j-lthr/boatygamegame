use bevy::prelude::*;
use crate::ability::components::events::OnActiveDespawn;

use crate::{
    ability::{CastBy, CastDynamicAbility, DynamicAbility},
    modifiers::{ModifierID, ModifierStack, apply_modifier_if_present},
};



#[derive(Component)]
pub struct SubCastInfo {
    index: i32,
    num_casts: i32,
}

impl SubCastInfo {
    pub fn new(index: i32, num_casts: i32) -> Self {
        Self { index, num_casts }
    }

    pub fn index(&self) -> i32 {
        self.index
    }

    pub fn num_casts(&self) -> i32 {
        self.num_casts
    }
}

#[derive(Component, Clone, Debug)]
pub struct SubCastOnce {
    pub ability: DynamicAbility,
    pub num_casts: i32,
    pub modified_by: Option<ModifierID>,
}

impl SubCastOnce {
    pub fn new(ability: DynamicAbility, num_casts: i32) -> Self {
        Self {
            ability,
            num_casts,
            modified_by: None,
        }
    }

    pub fn modified_by(mut self, modifier_id: ModifierID) -> Self {
        self.modified_by = Some(modifier_id);
        self
    }
}

pub fn handle_sub_cast_once(
    mut commands: Commands,
    sub_casts: Query<(Entity, &SubCastOnce, &CastBy)>,
    modifiers: Query<&ModifierStack>,
    transforms: Query<&Transform>,
    mut cast_events: EventWriter<CastDynamicAbility>,
) -> Result {
    for (entity, sub_cast_once, cast_by) in sub_casts.iter() {
        let num_casts = if let Some(modifier_id) = sub_cast_once.modified_by {
            apply_modifier_if_present(
                modifiers.get(cast_by.entity).ok(),
                modifier_id,
                sub_cast_once.num_casts as f32,
            ) as i32
        } else {
            sub_cast_once.num_casts
        };

        let transform = transforms.get(entity)?;

        for index in 0..num_casts {
            cast_events.write(
                CastDynamicAbility::at_caster(sub_cast_once.ability.clone(), cast_by.entity)
                    .with_target_position(transform.translation)
                    .with_sub_cast(entity, num_casts, index)
            );
        }

        commands.entity(entity).despawn();
    }

    Ok(())
}

#[derive(Component, Clone, Debug)]
pub struct TimedSubCast {
    pub ability: DynamicAbility,
    pub num_casts_per_interval: i32,
    pub timer: Timer,
    pub num_repeats: i32,
}

impl TimedSubCast {
    pub fn new_once(ability: DynamicAbility, num_casts_per_interval: i32, delay: f32) -> Self {
        Self {
            ability,
            num_casts_per_interval,
            timer: Timer::from_seconds(delay, TimerMode::Once),
            num_repeats: 0,
        }
    }

    pub fn new_repeating(
        ability: DynamicAbility,
        num_casts_per_interval: i32,
        interval: f32,
        num_repeats: i32,
    ) -> Self {
        Self {
            ability,
            num_casts_per_interval,
            timer: Timer::from_seconds(interval, TimerMode::Once),
            num_repeats,
        }
    }
}

pub fn handle_timed_sub_cast(
    mut commands: Commands,
    time: Res<Time>,
    mut sub_casts: Query<(Entity, &mut TimedSubCast, &CastBy, &Transform)>,
    mut cast_events: EventWriter<CastDynamicAbility>,
) {
    for (entity, mut timed_sub_cast, cast_by, transform) in sub_casts.iter_mut() {
        timed_sub_cast.timer.tick(time.delta());

        if timed_sub_cast.timer.finished() {
            for index in 0..timed_sub_cast.num_casts_per_interval {
                cast_events.write(
                    CastDynamicAbility::at_caster(timed_sub_cast.ability.clone(), cast_by.entity)
                        .with_target_position(transform.translation)
                        .with_sub_cast(entity, timed_sub_cast.num_casts_per_interval, index)
                );
            }

            if timed_sub_cast.num_repeats > 0 {
                timed_sub_cast.num_repeats -= 1;
                timed_sub_cast.timer.reset();
            } else {
                commands.entity(entity).despawn();
            }
        }
    }
}

#[derive(Component, Clone)]
pub struct CastOnDespawn {
    pub ability: DynamicAbility,
    pub num_casts: i32,
    pub modified_by: Option<ModifierID>,
}

impl CastOnDespawn {
    pub fn new(ability: DynamicAbility, num_casts: i32) -> Self {
        Self {
            ability,
            num_casts,
            modified_by: None,
        }
    }

    pub fn modified_by(mut self, modifier_id: ModifierID) -> Self {
        self.modified_by = Some(modifier_id);
        self
    }
}

pub fn handle_cast_on_despawn(
    trigger: Trigger<OnActiveDespawn>,
    _commands: Commands,
    cast_on_despawn: Query<(Entity, &CastOnDespawn, &CastBy, &Transform)>,
    modifiers: Query<&ModifierStack>,
    transforms: Query<&Transform>,
    mut cast_events: EventWriter<CastDynamicAbility>,
) -> Result {
    if let Ok((entity, cast_on_despawn, cast_by, _transform)) = cast_on_despawn.get(trigger.target()) {
        let num_casts = if let Some(modifier_id) = cast_on_despawn.modified_by {
            apply_modifier_if_present(
                modifiers.get(cast_by.entity).ok(),
                modifier_id,
                cast_on_despawn.num_casts as f32,
            ) as i32
        } else {
            cast_on_despawn.num_casts
        };

        let transform = transforms.get(entity)?;

        for index in 0..num_casts {
            cast_events.write(
                CastDynamicAbility::at_caster(cast_on_despawn.ability.clone(), cast_by.entity)
                    .with_target_position(transform.translation)
                    .with_sub_cast(entity, num_casts, index)
            );
        }
    }

    Ok(())
}

pub fn plugin(app: &mut bevy::app::App) {
    app.add_systems(Update, handle_sub_cast_once);
    app.add_systems(Update, handle_timed_sub_cast);
    app.add_observer(handle_cast_on_despawn);
}
