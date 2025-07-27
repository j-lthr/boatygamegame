use bevy::prelude::*;

use std::sync::Arc;
use std::time::Duration;
use avian3d::prelude::LinearVelocity;
use crate::{ability::components::events::OnSpawn, init::DespawnOnReset};
use crate::common::{EntityModifier, BundleInjector};
use crate::modifiers::*;

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
    mut query: Query<(&mut AbilitySlot<T>, Option<&ModifierStack>)>,
    time: Res<Time>,
) {
    for (mut ability_slot, modifiers) in query.iter_mut() {

        let delta = time.delta_secs();
        ability_slot.cooldown.tick(Duration::from_secs_f32(apply_modifier_if_present(modifiers, COOLDOWN_RECOVERY_RATE_MODIFIER, delta)));
    }
    
    for cast_attempt in cast_attempts.read() {
        if let Ok((mut ability_slot, _)) = query.get_mut(cast_attempt.caster) {
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

#[derive(Copy, Clone)]
pub enum SpawnLocation {
    Caster,
    Target
}
#[derive(Copy, Clone)]
pub struct CastConfig {
    pub spawn_location: SpawnLocation,
    pub inherit_velocity: bool,
}

impl Default for CastConfig {
    fn default() -> Self {
        Self { spawn_location: SpawnLocation::Caster, inherit_velocity: true }
    }
}

#[derive(Clone)]
pub struct DynamicAbility {
    components: Arc<dyn EntityModifier + Send + Sync>,
    config: CastConfig,
}

impl DynamicAbility {
    pub fn from_components(bundle: impl Bundle + Clone) -> Self {
        Self {
            components: Arc::new(
                BundleInjector(bundle)
            ),
            config: Default::default()
        }
    }

    pub fn with_config(mut self, config: CastConfig) -> Self {
        self.config = config;
        self
    }
}

#[derive(Clone, Copy)]
pub enum AbilityTarget {
    Entity(Entity),
    Position(Vec3),
    None
}

#[derive(Clone, Copy, Component)]
pub struct SubCast {
    pub parent: Entity,
    num_casts: i32,
}

#[derive(Event)]
pub struct CastDynamicAbility {
    ability: DynamicAbility,
    caster: Entity,
    target: AbilityTarget,
    sub_cast: Option<SubCast>,
}

impl CastDynamicAbility {
    pub fn at_caster(ability: DynamicAbility, caster: Entity) -> Self {
        Self {
            ability,
            caster,
            target: AbilityTarget::None,
            sub_cast: None,
        }
    }

    pub fn with_sub_cast(mut self, parent_ability: Entity, num_casts: i32) -> Self {
        self.sub_cast = Some (SubCast {
            parent: parent_ability,
            num_casts,
        });
        self
    }

    pub fn with_target_entity(mut self, target: Entity) -> Self {
        self.target = AbilityTarget::Entity(target);
        self
    }

    pub fn with_target_position(mut self, position: Vec3) -> Self {
        self.target = AbilityTarget::Position(position);
        self
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
        let mut entity = commands.spawn((event.params, DespawnOnReset));

        let config = event.ability.config;

        match config.spawn_location {
            SpawnLocation::Caster => entity.insert(Transform::from_translation(event.params.cast_position).looking_at(event.params.target_position, Vec3::Y)),
            SpawnLocation::Target => entity.insert(Transform::from_translation(event.params.target_position)),
        };

        event.ability.components.add_to_entity(&mut entity);
    }
}


pub fn event_handler_dynamic_ability_casts(
    mut cast_events: EventReader<CastDynamicAbility>,
    time: Res<Time>,
    query: Query<(&Transform, Option<&LinearVelocity>)>,
    mut commands: Commands,
) -> Result<()> {
    for event in cast_events.read() {

        let (caster_transform, caster_velocity) = query.get(event.caster)?;

        let (target_position, target_entity) = match event.target {
            AbilityTarget::Entity(target) => {
                let (target_transform, target_velocity) = query.get(target)?;

                (target_transform.translation, Some(target))
            },
            AbilityTarget::Position(pos) => {
                (pos, None)
            },
            AbilityTarget::None => {
                (caster_transform.translation, None)
            }
        };


        let cast_params = CastInfo {
            caster: event.caster,
            cast_position: caster_transform.translation,
            target_position,
            target_entity,
            cast_time: time.elapsed_secs_f64(),
        };

        let mut entity = commands.spawn((cast_params, DespawnOnReset));

        let config = event.ability.config;

        match config.spawn_location {
            SpawnLocation::Caster => entity.insert(Transform::from_translation(cast_params.cast_position).looking_at(cast_params.target_position, Vec3::Y)),
            SpawnLocation::Target => entity.insert(Transform::from_translation(cast_params.target_position)),
        };

        if config.inherit_velocity {
            if let Some(caster_velocity) = caster_velocity {
                entity.insert(caster_velocity.clone());
            }
        }

        event.ability.components.add_to_entity(&mut entity);
    }

    Ok(())
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

    app.add_event::<CastDynamicAbility>();

    app.add_systems(Update, event_handler_dynamic_ability_casts);
}
