use std::{any::Any, marker::PhantomData};

use bevy::{ecs::{query::QueryData, system::{StaticSystemParam, SystemParam}}, prelude::*};

pub trait Ability : Clone + Send + Sync + 'static {
    type TargetType : Clone + Send + Sync + 'static;
    
    fn add_systems(app: &mut App);
}

#[derive(Event)]
pub struct AttemptCastEvent<T: Ability> {
    pub ability: T,
    pub caster: Entity,
    pub target: T::TargetType,
    pub _marker: std::marker::PhantomData<T>,
}

#[derive(Event)]
pub struct CastEvent<T: Ability> {
    pub ability: T,
    pub caster: Entity,
    pub target: T::TargetType,
    pub _marker: std::marker::PhantomData<T>,
}

#[derive(Component)]
pub struct AbilitySlot<T: Ability> {
    pub cooldown: Timer,
    pub name: &'static str,
    pub _ability: std::marker::PhantomData<T>,
}

pub fn handle_cast_attempts<T: Ability>(mut commands: Commands, mut cast_attempts: EventReader<AttemptCastEvent<T>>, mut cast_events: EventWriter<CastEvent<T>>, mut query: Query<&mut AbilitySlot<T>>) {
    for cast_attempt in cast_attempts.read() {
        if let Ok(mut ability_slot) = query.get_mut(cast_attempt.caster) {
            if ability_slot.cooldown.finished() {
                cast_events.write(
                    CastEvent {
                        ability: cast_attempt.ability.clone(),
                            caster: cast_attempt.caster,
                            target: cast_attempt.target.clone(),
                            _marker: PhantomData
                    }
                );
                ability_slot.cooldown.reset();
            }
        } else {
            warn!("Entity {} attempted to cast an ability it doesn't own.", cast_attempt.caster);
        }
    }
}


#[derive(Default)]
pub struct AbilityPlugin<T: Ability> {
    _marker: PhantomData<T>
}

impl<T: Ability> Plugin for AbilityPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_event::<AttemptCastEvent<T>>();       
        app.add_event::<CastEvent<T>>();
        app.add_systems(
            Update,
            handle_cast_attempts::<T>
        );
        T::add_systems(app);
    }
}

