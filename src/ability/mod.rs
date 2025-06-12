use std::marker::PhantomData;

use bevy::prelude::*;

pub mod basic_projectile_attack;
pub mod common;
pub mod dash;
pub mod missile_launcher;
pub mod slam;

pub trait Ability: Clone + Send + Sync + 'static {
    type CastParams: Clone + Send + Sync + 'static;

    fn add_systems(app: &mut App);
}

#[derive(Event)]
pub struct AttemptCastEvent<T: Ability> {
    pub caster: Entity,
    pub params: T::CastParams,
    pub _marker: std::marker::PhantomData<T>,
}

#[derive(Event)]
pub struct CastEvent<T: Ability> {
    pub ability: T,
    pub caster: Entity,
    pub params: T::CastParams,
    pub _marker: std::marker::PhantomData<T>,
}

#[derive(Component)]
pub struct AbilitySlot<T: Ability> {
    pub cooldown: Timer,
    pub name: &'static str,
    pub ability: T,
}

pub fn handle_cast_attempts<T: Ability>(
    mut cast_attempts: EventReader<AttemptCastEvent<T>>,
    mut cast_events: EventWriter<CastEvent<T>>,
    mut query: Query<&mut AbilitySlot<T>>,
    time: Res<Time>,
) {
    for mut ability_slot in query.iter_mut() {
        ability_slot.cooldown.tick(time.delta());
    }
    for cast_attempt in cast_attempts.read() {
        if let Ok(mut ability_slot) = query.get_mut(cast_attempt.caster) {
            if ability_slot.cooldown.finished() {
                info!(
                    "Entity {} used ability '{}'.",
                    cast_attempt.caster, ability_slot.name
                );
                cast_events.write(CastEvent {
                    ability: ability_slot.ability.clone(),
                    caster: cast_attempt.caster,
                    params: cast_attempt.params.clone(),
                    _marker: PhantomData,
                });
                ability_slot.cooldown.reset();
            }
        } else {
            warn!(
                "Entity {} attempted to cast an ability it doesn't own.",
                cast_attempt.caster
            );
        }
    }
}

fn register_ability<T: Ability>(app: &mut App) {
    app.add_event::<AttemptCastEvent<T>>();
    app.add_event::<CastEvent<T>>();
    app.add_systems(Update, handle_cast_attempts::<T>);
    T::add_systems(app);
}

pub fn plugin(app: &mut App) {
    app.add_plugins((
        register_ability::<dash::Dash>,
        register_ability::<basic_projectile_attack::BasicProjectileAttack>,
        register_ability::<slam::Slam>,
        register_ability::<missile_launcher::MissileLauncher>,
    ));
}
