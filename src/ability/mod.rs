use std::any::Any;

use bevy::prelude::*;

pub trait TargetFilter: Send + Sync {
    fn is_valid_target(world: &World, target: Entity) -> bool;
}

pub struct MarkerTargetFilter<Marker : Component + Sync + Send> {
    _marker: std::marker::PhantomData<Marker>,
}

impl<Marker : Component + Sync + Send> TargetFilter for MarkerTargetFilter<Marker> {
    fn is_valid_target(world: &World, target: Entity) -> bool {
        world.get::<Marker>(target).is_some()
    }
}

#[derive(Clone, Copy)]
pub struct Target {
    pub target_position: Vec3,
}

pub struct Damage {}

pub struct CastEvent<T: Send + Sync> {
    pub caster: Entity,
    pub target: Target,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Send + Sync> CastEvent<T> {
    pub fn new(caster: Entity, target: Target) -> Self {
        Self {
            target,
            caster,
            _marker: std::marker::PhantomData,
        }
    }
}

#[derive(Default)]
pub struct ProjectileAbility<TF: TargetFilter> {
    marker: std::marker::PhantomData<TF>,
}





