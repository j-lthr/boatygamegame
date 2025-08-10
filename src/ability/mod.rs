use bevy::prelude::*;

use std::sync::Arc;
use std::time::Duration;
use avian3d::prelude::LinearVelocity;
use bevy::ecs::system::QueryLens;
use crate::{ability::components::events::OnSpawn, init::DespawnOnReset};
use crate::ability::components::subcast::SubCastInfo;
use crate::common::{EntityModifier, BundleInjector};
use crate::modifiers::*;

pub mod common;
pub mod dash;
pub mod missile_launcher;
pub mod slam;
pub mod slots;
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
    Source,
    Target
}
#[derive(Copy, Clone)]
pub struct CastConfig {
    pub spawn_location: SpawnLocation,
    pub inherit_velocity: bool,
}

impl Default for CastConfig {
    fn default() -> Self {
        Self { spawn_location: SpawnLocation::Source, inherit_velocity: true }
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


#[derive(Clone, Copy, Component)]
pub struct CastBy {
    pub entity: Entity,
}

#[derive(Clone, Copy, Component)]
pub enum IntendedTarget {
    Entity(Entity),
    Position(Vec3),
    None
}

pub struct ResolvedTarget {
    pub entity: Option<Entity>,
    pub position: Vec3,
}

pub fn resolve_target(transforms: QueryLens<&GlobalTransform>, target: IntendedTarget) -> Result<ResolvedTarget> {



    match target {
        IntendedTarget::Entity(entity) => {
            Ok(ResolvedTarget {
                entity: Some(entity),
                position: transforms.query_inner().get(entity)?.translation()
            })
        },

        IntendedTarget::Position(position) => {
            Ok(ResolvedTarget {
                entity: None,
                position
            })
        }
        IntendedTarget::None => Err("no target specified".into()),
    }
}


#[derive(Clone, Copy, Component)]
pub struct SubCast {
    parent: Entity,
    num_casts: i32,
    cast_index: i32,
}

#[derive(Event)]
pub struct CastDynamicAbility {
    ability: DynamicAbility,
    caster: Entity,
    target: IntendedTarget,
    sub_cast: Option<SubCast>,
}

impl CastDynamicAbility {
    pub fn at_caster(ability: DynamicAbility, caster: Entity) -> Self {
        Self {
            ability,
            caster,
            target: IntendedTarget::None,
            sub_cast: None,
        }
    }

    pub fn with_sub_cast(mut self, parent_ability: Entity, num_casts: i32, cast_index: i32) -> Self {
        self.sub_cast = Some (SubCast {
            parent: parent_ability,
            num_casts,
            cast_index,
        });
        self
    }

    pub fn with_target_entity(mut self, target: Entity) -> Self {
        self.target = IntendedTarget::Entity(target);
        self
    }

    pub fn with_target_position(mut self, position: Vec3) -> Self {
        self.target = IntendedTarget::Position(position);
        self
    }
}

pub fn event_handler_dynamic_ability_casts(
    mut cast_events: EventReader<CastDynamicAbility>,
    query: Query<(&Transform, Option<&LinearVelocity>)>,
    mut commands: Commands,
) -> Result<()> {
    for event in cast_events.read() {

        let root_entity = if let Some(sub_cast) = event.sub_cast {
            sub_cast.parent
        } else {
            event.caster
        };

        let (root_transform, root_velocity) = query.get(root_entity)?;

        let (target_position) = match event.target {
            IntendedTarget::Entity(target) => {
                let (target_transform, _) = query.get(target)?;

                target_transform.translation
            },
            IntendedTarget::Position(pos) => {
                pos
            },
            IntendedTarget::None => {
                root_transform.translation
            }
        };

        let mut entity = commands.spawn((
            event.target, 
            DespawnOnReset,
            CastBy { entity: event.caster }
        ));

        // Add SubCastInfo if this is a sub-cast
        if let Some(sub_cast) = event.sub_cast {
            entity.insert(SubCastInfo::new(sub_cast.cast_index, sub_cast.num_casts));
        }

        let config = event.ability.config;

        match config.spawn_location {
            SpawnLocation::Source => entity.insert(Transform::from_translation(root_transform.translation)),
            SpawnLocation::Target => entity.insert(Transform::from_translation(target_position)),
        };

        if config.inherit_velocity {
            if let Some(root_velocity) = root_velocity {
                entity.insert(root_velocity.clone());
            }
        }

        event.ability.components.add_to_entity(&mut entity);
    }

    Ok(())
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
        components::plugin,
        slots::plugin,
    ));

    app.add_event::<CastDynamicAbility>();

    app.add_systems(Update, event_handler_dynamic_ability_casts);
}
