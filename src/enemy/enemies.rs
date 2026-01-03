use bevy::prelude::*;

use super::components::*;
use super::registry::*;
use crate::ability::components::subcast::TimedSubCast;
use crate::ability::DynamicAbility;
use crate::ability::components::blast::BlastBundle;
use crate::ability::components::common::Lifetime;
use crate::ability::components::projectile::{DamageOnCollision, DespawnOnCollision, InitialVelocity};
use crate::ability::components::spawn::{RadialSubCastOffset, SpawnEnemyAtCastPosition, RandomSpawnOffset};
use crate::ability::components::subcast::SubCastOnce;
use crate::ability::components::visual::LifetimeFadeout;
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
        .add_rune(0.6, PROJECTILE_SPEED_RUNE)
        .add_rune(0.6, HOMING_RUNE)
        .with_chance(0.3)
        .build();

    {
        let material = materials.add(StandardMaterial {
            emissive: LinearRgba::rgb(100.0, 1.0, 1.0),
            ..Default::default()
        });

        let projectile = DynamicAbility::from_components((
            InitialVelocity::forward(30.0),
            Lifetime::fixed(2.0),
            LifetimeFadeout::new(0.2),
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
                HealthBundle::new(50, 0),
                FollowTarget::ranged(10.0, 0.0),
                FirstOrderMovement {
                    speed: 10.0,
                    jitter: 0.1,
                },
                SingleAbilityTimed::new(ability, 1.0),
                normal_drop_table.clone(),
            ),
        ).with_num_slots(2);

        registry.register_enemy(ranger);
    }

    {
        let rusher_material = materials.add(StandardMaterial {
            emissive: LinearRgba::rgb(70.0, 70.0, 1.0),
            ..Default::default()
        });

        let rusher = Enemy::from_components(
            "rusher",
            (
                Mesh3d(meshes.add(Sphere::new(0.5))),
                MeshMaterial3d(rusher_material),
                HealthBundle::new(30, 0),
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
        ).with_num_slots(1);

        registry.register_enemy(rusher);
    }

    {
        let material = materials.add(StandardMaterial {
            emissive: LinearRgba::rgb(1.0, 1.0, 70.0),
            ..Default::default()
        });

        let projectile = DynamicAbility::from_components((
            InitialVelocity::forward(30.0),
            Lifetime::fixed(2.0),
            LifetimeFadeout::new(0.2),
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
                HealthBundle::new(40, 5),
                SingleAbilityTimed::new(ability, 0.1),
                FollowTarget {
                    mode: FollowMovementMode::Ranged { preferred_distance: 30.0, rotation_speed: 0.0 },
                },
                FirstOrderMovement {
                    speed: 3.0,
                    jitter: 0.0,
                },
                normal_drop_table.clone(),
            ),
        ).with_min_level(5).with_num_slots(5);

        registry.register_enemy(spiral_shooter);
    }

    {
        let material = materials.add(StandardMaterial {
            emissive: LinearRgba::rgb(1.0, 60.0, 55.0),
            ..Default::default()
        });

        let blast = DynamicAbility::from_components(
            BlastBundle::new(
                &mut meshes,
                &mut materials,
                LinearRgba::rgb(1000.0, 1000.0, 1000.0).into(),
                4.0,
                100,
                0.5
            )
        );

        let projectile = DynamicAbility::from_components((
            InitialVelocity::forward(70.0),
            Lifetime::fixed(0.5),
            DespawnOnCollision,
            DamageOnCollision { base_damage: 10.0 },
            RadialSubCastOffset::from_radius_360(1.0),
            Mesh3d(meshes.add(Sphere::new(0.25))),
            MeshMaterial3d(material.clone()),
            TimedSubCast::new_repeating(blast, 1, 0.1, 5)
        ));

        let ability = DynamicAbility::from_components((SubCastOnce::new(projectile.clone(), 5),));

        let star_blaster = Enemy::from_components(
            "star-blaster",
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
        ).with_min_level(5).with_num_slots(5);


        registry.register_enemy(star_blaster);
    }

    // Minion enemy - spawned by summoner
    {
        let minion_material = materials.add(StandardMaterial {
            emissive: LinearRgba::rgb(20.0, 20.0, 80.0),
            ..Default::default()
        });

        let minion = Enemy::from_components(
            "minion",
            (
                Mesh3d(meshes.add(Sphere::new(0.3))),
                MeshMaterial3d(minion_material),
                HealthBundle::new(10, 0),
                FollowTarget {
                    mode: FollowMovementMode::ToMeleeRange,
                },
                FirstOrderMovement {
                    speed: 20.0,
                    jitter: 0.15,
                },
                ContactDamage::new(15, 1.5, 1.0).with_self_knockback(8.0),
                DespawnTimer::new(5.0),
            ),
        ).with_num_slots(1);


        let summoner_material = materials.add(StandardMaterial {
            emissive: LinearRgba::rgb(20.0, 20.0, 80.0),
            ..Default::default()
        });

        let summoning_ability = DynamicAbility::from_components((
            SubCastOnce::new(
                DynamicAbility::from_components((
                    SpawnEnemyAtCastPosition::new(minion),
                    RadialSubCastOffset::from_radius_360(2.0),
                    RandomSpawnOffset::new(1.0, 0.5),
                )),
                1
            ),
        ));

        let summoner = Enemy::from_components(
            "summoner",
            (
                Mesh3d(meshes.add(Sphere::new(2.0))),
                MeshMaterial3d(summoner_material),
                HealthBundle::new(100, 2),
                FollowTarget {
                    mode: FollowMovementMode::Ranged { preferred_distance: 15.0, rotation_speed: 0.0 },
                },
                FirstOrderMovement {
                    speed: 5.0,
                    jitter: 0.1,
                },
                SingleAbilityTimed::new(summoning_ability, 0.75),
                normal_drop_table.clone(),
            ),
        ).with_min_level(7).with_num_slots(7);

        {
            let material = materials.add(StandardMaterial {
                emissive: LinearRgba::rgb(100.0, 70.0, 40.0),
                ..Default::default()
            });

            let projectile = DynamicAbility::from_components((
                InitialVelocity::forward(70.0),
                Lifetime::fixed(2.0),
                DespawnOnCollision,
                DamageOnCollision { base_damage: 40.0 },
                RadialSubCastOffset::from_degrees_per_cast(1.0, 5.0),
                Mesh3d(meshes.add(Sphere::new(0.25))),
                MeshMaterial3d(material.clone()),
            ));

            let ability = DynamicAbility::from_components((SubCastOnce::new(projectile.clone(), 1),));

            let spinner = Enemy::from_components(
                "spinner",
                (
                    Mesh3d(meshes.add(Sphere::new(0.7))),
                    MeshMaterial3d(material),
                    HealthBundle::new(100, 0),
                    FollowTarget::ranged(30.0, 30.0),
                    FirstOrderMovement {
                        speed: 20.0,
                        jitter: 0.0,
                    },
                    SingleAbilityTimed::new(ability, 2.0),
                    normal_drop_table.clone(),
                ),
            ).with_num_slots(4).with_min_level(10);

            registry.register_enemy(spinner);
        }

        {
            let material = materials.add(StandardMaterial {
                emissive: LinearRgba::rgb(100.0, 1.0, 1.0),
                ..Default::default()
            });

            let projectile = DynamicAbility::from_components((
                InitialVelocity::forward(30.0),
                Lifetime::fixed(2.0),
                LifetimeFadeout::new(0.2),
                DespawnOnCollision,
                DamageOnCollision { base_damage: 50.0 },
                RandomSpawnOffset::new(0.0, 0.5),
                Mesh3d(meshes.add(Sphere::new(3.0))),
                MeshMaterial3d(material.clone()),
            ));

            let ability = DynamicAbility::from_components((SubCastOnce::new(projectile.clone(), 5),));

            let ranger = Enemy::from_components(
                "ranger-boss",
                (
                    Mesh3d(meshes.add(Sphere::new(10.0))),
                    MeshMaterial3d(material),
                    HealthBundle::new(700, 0),
                    FollowTarget::ranged(30.0, 0.0),
                    FirstOrderMovement {
                        speed: 2.0,
                        jitter: 0.1,
                    },
                    SingleAbilityTimed::new(ability, 0.2),
                    normal_drop_table.clone(),
                ),
            ).with_num_slots(20);

            registry.register_enemy(ranger);
        }

        registry.register_enemy(summoner);
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, register_enemies);
}
