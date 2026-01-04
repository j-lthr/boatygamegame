use bevy::prelude::*;

use crate::ability::components::subcast::SubCastInfo;
use crate::common::{BundleInjector, EntityModifier, Faction};
use crate::init::DespawnOnReset;
use crate::modifiers::*;
use avian3d::prelude::{CollisionEventsEnabled, LinearVelocity};
use bevy::ecs::system::QueryLens;
use std::sync::Arc;
use std::time::Duration;

pub mod components;
pub mod slots;

#[derive(Copy, Clone, Debug)]
pub enum SpawnLocation {
    Source,
    Target,
}
#[derive(Copy, Clone, Debug)]
pub struct CastConfig {
    pub spawn_location: SpawnLocation,
    pub inherit_transform: bool,
    pub inherit_velocity: bool,
    pub inherit_faction: bool,
}

impl Default for CastConfig {
    fn default() -> Self {
        Self {
            spawn_location: SpawnLocation::Source,
            inherit_velocity: true,
            inherit_transform: true,
            inherit_faction: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DynamicAbility {
    components: Arc<dyn EntityModifier + Send + Sync>,
    config: CastConfig,
}

impl DynamicAbility {
    pub fn from_components(bundle: impl Bundle + Clone + std::fmt::Debug) -> Self {
        Self {
            components: Arc::new(BundleInjector(bundle)),
            config: Default::default(),
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

#[derive(Clone, Copy, Component, Debug)]
pub enum IntendedTarget {
    Entity(Entity),
    Position(Vec3),
    None,
}

pub struct ResolvedTarget {
    pub entity: Option<Entity>,
    pub position: Vec3,
}

pub fn resolve_target(
    transforms: QueryLens<&GlobalTransform>,
    target: IntendedTarget,
) -> Result<ResolvedTarget> {
    match target {
        IntendedTarget::Entity(entity) => Ok(ResolvedTarget {
            entity: Some(entity),
            position: transforms.query_inner().get(entity)?.translation(),
        }),

        IntendedTarget::Position(position) => Ok(ResolvedTarget {
            entity: None,
            position,
        }),
        IntendedTarget::None => Err("no target specified".into()),
    }
}

#[derive(Clone, Copy, Component, Debug)]
pub struct SubCast {
    parent: Entity,
    num_casts: i32,
    cast_index: i32,
}

#[derive(Event, Debug)]
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

    pub fn with_sub_cast(
        mut self,
        parent_ability: Entity,
        num_casts: i32,
        cast_index: i32,
    ) -> Self {
        self.sub_cast = Some(SubCast {
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

pub fn ability_cast_observer(
    cast_event: Trigger<CastDynamicAbility>,
    query: Query<(&Transform, Option<&LinearVelocity>, Option<&Faction>)>,
    mut commands: Commands,
) -> Result<()> {
    let event = cast_event.event();

    let root_entity = if let Some(sub_cast) = event.sub_cast {
        sub_cast.parent
    } else {
        event.caster
    };

    let (root_transform, root_velocity, root_faction) = query.get(root_entity)?;

    let target_position = match event.target {
        IntendedTarget::Entity(target) => {
            let (target_transform, _, _) = query.get(target)?;
            target_transform.translation
        }
        IntendedTarget::Position(pos) => pos,
        IntendedTarget::None => root_transform.translation,
    };

    let mut entity = commands.spawn((
        event.target,
        DespawnOnReset,
        CastBy {
            entity: event.caster,
        },
    ));

    // Add SubCastInfo if this is a sub-cast
    if let Some(sub_cast) = event.sub_cast {
        entity.insert(SubCastInfo::new(sub_cast.cast_index, sub_cast.num_casts));
    }

    let config = event.ability.config;

    let position = match config.spawn_location {
        SpawnLocation::Source => root_transform.translation,
        SpawnLocation::Target => target_position,
    };

    let mut transform = Transform::from_translation(position);

    //transform.look_to(target_position - root_transform.translation, Vec3::Y);

    if config.inherit_transform {
        transform.rotation = root_transform.rotation;
    }

    entity.insert(transform);

    entity.insert(CollisionEventsEnabled);

    if config.inherit_faction
        && let Some(root_faction) = root_faction
    {
        entity.insert(*root_faction);
    }

    if config.inherit_velocity
        && let Some(root_velocity) = root_velocity
    {
        //dbg!(root_velocity);
        entity.insert(*root_velocity);
    } else {
        entity.insert(LinearVelocity::ZERO);
    }

    event.ability.components.add_to_entity(&mut entity);

    Ok(())
}


pub fn plugin(app: &mut App) {
    app.add_plugins((components::plugin, slots::plugin));

    app.add_event::<CastDynamicAbility>();

    app.add_observer(ability_cast_observer);
}
