use bevy::prelude::*;

use std::sync::Arc;

pub mod common;
pub mod dash;
pub mod missile_launcher;
pub mod slam;
pub mod components;

pub trait Ability: Clone + Send + Sync + 'static {
    type CastParams: Clone + Send + Sync + 'static;

    fn add_systems(app: &mut App);
}

#[derive(Event)]
pub struct AttemptCastEvent<T: Ability> {
    pub caster: Entity,
    pub params: T::CastParams,
}

#[derive(Event)]
pub struct CastEvent<T: Ability> {
    pub ability: T,
    pub caster: Entity,
    pub params: T::CastParams,
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

pub trait BundleInjector {
    fn add_to_entity(&self, entity: &mut EntityCommands);
}

#[derive(Clone, Debug)]
pub struct BundleWrapper<B: Bundle + Clone>(pub B);

impl<B: Bundle + Clone> BundleInjector for BundleWrapper<B> {
    fn add_to_entity(&self, entity: &mut EntityCommands) {
        entity.insert(self.0.clone());
    }
}

#[derive(Clone)]
pub struct DynamicAbility {
    components: Arc<dyn BundleInjector + Send + Sync>,
}

impl DynamicAbility {
    pub fn with_components(bundle: impl Bundle + Clone) -> Self {
        Self {
            components: Arc::new(
                BundleWrapper(bundle)
            ),
        }
    }
}

#[derive(Copy, Clone, Component)]
pub struct CastInfo {
    pub caster: Entity,
    pub cast_position: Vec3,
    pub target_position: Vec3,
    pub target_entity: Option<Entity>,
    pub cast_time: f64,
}

pub fn handle_dynamic_ability_casts(
    mut cast_events: EventReader<CastEvent<DynamicAbility>>,
    mut commands: Commands,
) {
    for event in cast_events.read() {
        let mut default_cast_entity = commands.spawn(event.params);

        event.ability.components.add_to_entity(&mut default_cast_entity);
    }
}

impl Ability for DynamicAbility {
    type CastParams = CastInfo;

    fn add_systems(app: &mut bevy::app::App) {
        app.add_systems(Update, handle_dynamic_ability_casts);
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
        register_ability::<slam::Slam>,
        register_ability::<missile_launcher::MissileLauncher>,
        register_ability::<DynamicAbility>,
        components::plugin,
    ));
}
