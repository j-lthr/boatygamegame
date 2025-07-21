use bevy::prelude::*;

use super::components::*;
use super::registry::*;
use crate::ability::components::subcast::TimedSubCast;
use crate::ability::DynamicAbility;
use crate::ability::components::blast::BlastBundle;
use crate::ability::components::common::Lifetime;
use crate::ability::components::projectile::{
    DamageOnCollision, DespawnOnCollision, LinearMovement, SimpleCollider,
};
use crate::ability::components::spawn::RadialSubCastOffset;
use crate::ability::components::subcast::{CastOnDespawn, SubCastOnce};
use crate::common::HealthBundle;
use crate::loot::DropTableBuilder;
use crate::rune::*;

pub fn register_enemies(
    mut registry: ResMut<EnemyRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let normal_drop_table = DropTableBuilder::new()
        .add_rune(2.0, SPEED_RUNE)
        .add_rune(0.5, MULTISHOT_RUNE)
        .add_rune(2.0, DAMAGE_RUNE)
        .add_rune(1.5, AOE_RUNE)
        .add_rune(2.5, MAX_HEALTH_RUNE)
        .add_rune(1.2, HEALTH_REGEN_RUNE)
        .add_rune(0.8, COOLDOWN_RECOVERY_RATE_RUNE)
        //    .add_rune(0.6, HOMING_RUNE)
        .with_chance(1.0)
        .build();

    {
        let material = materials.add(StandardMaterial {
            emissive: LinearRgba::rgb(10.0, 1.0, 1.0),
            ..Default::default()
        });

        let projectile = DynamicAbility::from_components((
            LinearMovement { base_speed: 30.0 },
            Lifetime::fixed(10.0),
            SimpleCollider { radius: 1.0 },
            DespawnOnCollision,
            DamageOnCollision { base_damage: 10.0 },
            RadialSubCastOffset::from_degrees_per_cast(1.0, 5.0),
            Mesh3d(meshes.add(Sphere::new(0.25))),
            MeshMaterial3d(material.clone()),
        ));

        let ability = DynamicAbility::from_components((SubCastOnce::new(projectile.clone(), 3),));

        let ranger = Enemy::from_components(
            "small-ranger",
            (
                Mesh3d(meshes.add(Sphere::new(1.0))),
                MeshMaterial3d(material),
                HealthBundle::new(30, 0),
                FollowTarget::ranged(10.0, 0.0),
                FirstOrderMovement {
                    speed: 10.0,
                    jitter: 0.1,
                },
                SingleAbilityTimed::new(ability, 1.0),
                normal_drop_table.clone(),
            ),
        );

        registry.register_enemy(ranger);
    }

    {
        let rusher_material = materials.add(StandardMaterial {
            emissive: LinearRgba::rgb(5.0, 5.0, 1.0),
            ..Default::default()
        });

        let rusher = Enemy::from_components(
            "rusher",
            (
                Mesh3d(meshes.add(Sphere::new(0.5))),
                MeshMaterial3d(rusher_material),
                HealthBundle::new(50, 0),
                FollowTarget {
                    mode: FollowMovementMode::ToMeleeRange,
                },
                FirstOrderMovement {
                    speed: 15.0,
                    jitter: 0.2,
                },
                ContactDamage::new(25, 2.0, 1.0).with_self_knockback(10.0),
                normal_drop_table.clone(),
            ),
        );

        registry.register_enemy(rusher);
    }

    {
        let material = materials.add(StandardMaterial {
            emissive: LinearRgba::rgb(1.0, 1.0, 10.0),
            ..Default::default()
        });

        let projectile = DynamicAbility::from_components((
            LinearMovement { base_speed: 30.0 },
            Lifetime::fixed(10.0),
            SimpleCollider { radius: 1.0 },
            DespawnOnCollision,
            DamageOnCollision { base_damage: 10.0 },
            RadialSubCastOffset::from_radius_360(1.0),
            Mesh3d(meshes.add(Sphere::new(0.25))),
            MeshMaterial3d(material.clone()),
        ));

        let ability = DynamicAbility::from_components((SubCastOnce::new(projectile.clone(), 5),));

        let spiral_shooter = Enemy::from_components(
            "spiral-shooter",
            (
                Mesh3d(meshes.add(Sphere::new(1.0))),
                MeshMaterial3d(material),
                HealthBundle::new(100, 0),
                SingleAbilityTimed::new(ability, 0.5),
                normal_drop_table.clone(),
            ),
        );

        registry.register_enemy(spiral_shooter);
    }

    {
        let color = LinearRgba::rgb(1.0, 10.0, 10.0);
        let material = materials.add(StandardMaterial {
            emissive: color,
            ..Default::default()
        });

        let blast = DynamicAbility::from_components(
            BlastBundle::new(
                &mut meshes, 
                &mut materials,
                color.into(),
                4.0,
                100,
                0.5
            )
        );

        let projectile = DynamicAbility::from_components((
            LinearMovement { base_speed: 50.0 },
            Lifetime::fixed(0.5),
            SimpleCollider { radius: 1.0 },
            DespawnOnCollision,
            DamageOnCollision { base_damage: 10.0 },
            RadialSubCastOffset::from_radius_360(1.0),
            Mesh3d(meshes.add(Sphere::new(0.25))),
            MeshMaterial3d(material.clone()),
            TimedSubCast::new_repeating(blast, 1, 0.1, 5)
        ));

        let ability = DynamicAbility::from_components((SubCastOnce::new(projectile.clone(), 5),));

        let spiral_blaster = Enemy::from_components(
            "spiral-shooter",
            (
                Mesh3d(meshes.add(Sphere::new(1.0))),
                MeshMaterial3d(material),
                FollowTarget {
                    mode: FollowMovementMode::ToMeleeRange,
                },
                FirstOrderMovement {
                    speed: 3.0,
                    jitter: 0.2,
                },
                HealthBundle::new(30, 0),
                SingleAbilityTimed::new(ability, 2.0),
                normal_drop_table.clone(),
            ),
        );


        registry.register_enemy(spiral_blaster);
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, register_enemies);
}
