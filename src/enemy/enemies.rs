use avian3d::prelude::*;
use bevy::prelude::*;

use super::components::*;
use super::registry::*;
use crate::ability::CastConfig;
use crate::ability::DynamicAbility;
use crate::ability::components::blast::BlastBundle;
use crate::ability::components::common::Lifetime;
use crate::ability::components::projectile::BasicProjectileBundle;
use crate::ability::components::projectile::{
    DamageOnCollision, DespawnOnCollision, InitialVelocity,
};
use crate::ability::components::spawn::{
    RadialSubCastOffset, RandomSpawnOffset, SpawnEnemyAtCastPosition,
};
use crate::ability::components::subcast::SubCastOnce;
use crate::ability::components::subcast::TimedSubCast;
use crate::ability::components::visual::LifetimeFadeout;
use crate::common::HealthBundle;
use crate::loot::DropTableBuilder;
use crate::modifiers::COOLDOWN_RECOVERY_RATE_MODIFIER;
use crate::modifiers::Modifier;
use crate::rune::*;

pub fn register_enemies(
    mut registry: ResMut<EnemyRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let normal_drop_table = DropTableBuilder::new()
        .add_rune(2.0, SPEED_RUNE)
        .add_rune(0.5, HealRune {
            amount: 100,
            color: Color::srgb(5.0, 5.0, 5.0)
        })
        .add_rune(
            0.025,
            ModifierRune {
                modifier: Modifier::multiplicative(COOLDOWN_RECOVERY_RATE_MODIFIER, 2.0),
                color: Color::srgb(100.0, 100.0, 0.0),
            },
        )
        .add_rune(0.5, MULTISHOT_RUNE)
        .add_rune(2.0, DAMAGE_RUNE)
        //.add_rune(1.5, AOE_RUNE)
        .add_rune(2.5, MAX_HEALTH_RUNE)
        .add_rune(1.2, HEALTH_REGEN_RUNE)
        .add_rune(0.8, COOLDOWN_RECOVERY_RATE_RUNE)
        .add_rune(0.6, PROJECTILE_SPEED_RUNE)
        //.add_rune(0.6, HOMING_RUNE)
        .with_chance(0.5)
        .build();

    {
        let rusher_material = materials.add(StandardMaterial {
            emissive: LinearRgba::rgb(0.0, 70.0, 70.0),
            ..Default::default()
        });

        let rusher = Enemy::from_components(
            "rusher",
            (
                Mesh3d(meshes.add(Cuboid::new(2.5, 1.3, 2.5))),
                MeshMaterial3d(rusher_material),
                Collider::cuboid(2.5, 1.3, 2.5),
                HealthBundle::new(70, 5),
                FollowTarget::melee(),
                ForceMovement {
                    speed: 60.0,
                    omega: 15.0,
                    gamma: 20.0,
                },
                ContactDamage::new(25, 4.0, 1.0).with_self_damage(70),
                normal_drop_table.clone(),
            ),
        )
        .with_num_slots(1);

        registry.register_enemy(rusher);
    }

    {
        let color = LinearRgba::rgb(60.0, 30.0, 40.0);
        let material = materials.add(StandardMaterial {
            emissive: color,
            ..Default::default()
        });

        let projectile = DynamicAbility::from_components((BasicProjectileBundle {
            subcast_offset: RadialSubCastOffset::from_degrees_per_cast(2.0, 15.0),
            ..BasicProjectileBundle::new(
                2.0,
                0.25,
                80.0,
                10.0,
                color.into(),
                meshes.as_mut(),
                materials.as_mut(),
            )
        },));

        let ability = DynamicAbility::from_components((SubCastOnce::new(projectile.clone(), 3),));

        let ranger = Enemy::from_components(
            "small-ranger",
            (
                Mesh3d(meshes.add(Sphere::new(1.0))),
                Collider::sphere(1.1),
                MeshMaterial3d(material),
                HealthBundle::new(50, 0),
                FollowTarget::ranged(10.0, 5.0),
                ForceMovement {
                    speed: 30.0,
                    omega: 50.0,
                    gamma: 20.0,
                },
                SingleAbilityTimed::new(ability, 0.5),
                normal_drop_table.clone(),
            ),
        )
        .with_num_slots(2);

        registry.register_enemy(ranger);
    }

    {
        let color = LinearRgba::rgb(30.0, 30.0, 70.0);
        let material = materials.add(StandardMaterial {
            emissive: color,
            ..Default::default()
        });

        let projectile = DynamicAbility::from_components((BasicProjectileBundle {
            subcast_offset: RadialSubCastOffset::from_radius_360(2.0),
            ..BasicProjectileBundle::new(
                2.0,
                0.7,
                120.0,
                15.0,
                color.into(),
                meshes.as_mut(),
                materials.as_mut(),
            )
        },));

        let ability = DynamicAbility::from_components((SubCastOnce::new(projectile.clone(), 5),));

        let spiral_shooter = Enemy::from_components(
            "spiral-shooter",
            (
                Mesh3d(meshes.add(Sphere::new(2.0))),
                Collider::sphere(2.0),
                MeshMaterial3d(material),
                HealthBundle::new(100, 0),
                SingleAbilityTimed::new(ability, 0.1),
                ExternalTorque::new(Vec3::Y * 70.0).with_persistence(true),
                LockedAxes::new()
                    .lock_rotation_x()
                    .lock_rotation_z()
                    .lock_translation_y(),
                normal_drop_table.clone(),
            ),
        )
        .with_num_slots(7);

        registry.register_enemy(spiral_shooter);
    }

    {
        let color = LinearRgba::rgb(60.0, 10.0, 15.0);

        let material = materials.add(StandardMaterial {
            emissive: color,
            ..Default::default()
        });

        
        let sub_projectile = DynamicAbility::from_components((
            BasicProjectileBundle {
                subcast_offset: RadialSubCastOffset::from_radius_360(2.0),
                ..BasicProjectileBundle::new(
                    2.0,
                    0.2,
                    30.0,
                    15.0,
                    color.into(),
                    meshes.as_mut(),
                    materials.as_mut(),
                )
            },
        ));


      

        let projectile = DynamicAbility::from_components((
            BasicProjectileBundle {
                subcast_offset: RadialSubCastOffset::from_radius_360(2.0),
                ..BasicProjectileBundle::new(
                    6.0,
                    0.7,
                    15.0,
                    30.0,
                    color.into(),
                    meshes.as_mut(),
                    materials.as_mut(),
                )
            },
            TimedSubCast::new_repeating(sub_projectile, 5, 1.0, 20),
        ));

        let ability = DynamicAbility::from_components((SubCastOnce::new(projectile.clone(), 5),));

        let star_blaster = Enemy::from_components(
            "star-blaster",
            (
                Mesh3d(meshes.add(Sphere::new(3.0))),
                Collider::sphere(3.0),
                MeshMaterial3d(material),
                FollowTarget::melee(),
                ForceMovement {
                    speed: 100.0,
                    omega: 15.0,
                    gamma: 15.0,
                },
                HealthBundle::new(500, 0),
                SingleAbilityTimed::new(ability, 1.0),
                normal_drop_table.clone(),
            ),
        )
        .with_min_level(15)
        .with_num_slots(10);

        registry.register_enemy(star_blaster);
    }

    // // Minion enemy - spawned by summoner
    // {
    //     let minion_material = materials.add(StandardMaterial {
    //         emissive: LinearRgba::rgb(20.0, 20.0, 80.0),
    //         ..Default::default()
    //     });

    //     let minion = Enemy::from_components(
    //         "minion",
    //         (
    //             Mesh3d(meshes.add(Sphere::new(0.3))),
    //             MeshMaterial3d(minion_material),
    //             HealthBundle::new(10, 0),
    //             FollowTarget {
    //                 mode: FollowMovementMode::ToMeleeRange,
    //             },
    //             FirstOrderMovement {
    //                 speed: 20.0,
    //                 jitter: 0.15,
    //             },
    //             ContactDamage::new(15, 1.5, 1.0).with_self_knockback(8.0),
    //             DespawnTimer::new(5.0),
    //         ),
    //     ).with_num_slots(1);

    //     let summoner_material = materials.add(StandardMaterial {
    //         emissive: LinearRgba::rgb(20.0, 20.0, 80.0),
    //         ..Default::default()
    //     });

    //     let summoning_ability = DynamicAbility::from_components((
    //         SubCastOnce::new(
    //             DynamicAbility::from_components((
    //                 SpawnEnemyAtCastPosition::new(minion),
    //                 RadialSubCastOffset::from_radius_360(2.0),
    //                 RandomSpawnOffset::new(1.0, 0.5),
    //             )),
    //             1
    //         ),
    //     ));

    //     let summoner = Enemy::from_components(
    //         "summoner",
    //         (
    //             Mesh3d(meshes.add(Sphere::new(2.0))),
    //             MeshMaterial3d(summoner_material),
    //             HealthBundle::new(100, 2),
    //             FollowTarget {
    //                 mode: FollowMovementMode::Ranged { preferred_distance: 15.0, rotation_speed: 0.0 },
    //             },
    //             FirstOrderMovement {
    //                 speed: 5.0,
    //                 jitter: 0.1,
    //             },
    //             SingleAbilityTimed::new(summoning_ability, 0.75),
    //             normal_drop_table.clone(),
    //         ),
    //     ).with_min_level(7).with_num_slots(7);

    //     {
    //         let material = materials.add(StandardMaterial {
    //             emissive: LinearRgba::rgb(100.0, 70.0, 40.0),
    //             ..Default::default()
    //         });

    //         let projectile = DynamicAbility::from_components((
    //             InitialVelocity::forward(70.0),
    //             Lifetime::fixed(2.0),
    //             DespawnOnCollision,
    //             DamageOnCollision { base_damage: 40.0 },
    //             RadialSubCastOffset::from_degrees_per_cast(1.0, 5.0),
    //             Mesh3d(meshes.add(Sphere::new(0.25))),
    //             MeshMaterial3d(material.clone()),
    //         ));

    //         let ability = DynamicAbility::from_components((SubCastOnce::new(projectile.clone(), 1),));

    //         let spinner = Enemy::from_components(
    //             "spinner",
    //             (
    //                 Mesh3d(meshes.add(Sphere::new(0.7))),
    //                 MeshMaterial3d(material),
    //                 HealthBundle::new(100, 0),
    //                 FollowTarget::ranged(30.0, 30.0),
    //                 FirstOrderMovement {
    //                     speed: 20.0,
    //                     jitter: 0.0,
    //                 },
    //                 SingleAbilityTimed::new(ability, 2.0),
    //                 normal_drop_table.clone(),
    //             ),
    //         ).with_num_slots(4).with_min_level(10);

    //         registry.register_enemy(spinner);
    //     }

    //     {
    //         let material = materials.add(StandardMaterial {
    //             emissive: LinearRgba::rgb(100.0, 1.0, 1.0),
    //             ..Default::default()
    //         });

    //         let projectile = DynamicAbility::from_components((
    //             InitialVelocity::forward(30.0),
    //             Lifetime::fixed(2.0),
    //             LifetimeFadeout::new(0.2),
    //             DespawnOnCollision,
    //             DamageOnCollision { base_damage: 50.0 },
    //             RandomSpawnOffset::new(0.0, 0.5),
    //             Mesh3d(meshes.add(Sphere::new(3.0))),
    //             MeshMaterial3d(material.clone()),
    //         ));

    //         let ability = DynamicAbility::from_components((SubCastOnce::new(projectile.clone(), 5),));

    //         let ranger = Enemy::from_components(
    //             "ranger-boss",
    //             (
    //                 Mesh3d(meshes.add(Sphere::new(10.0))),
    //                 MeshMaterial3d(material),
    //                 HealthBundle::new(700, 0),
    //                 FollowTarget::ranged(30.0, 0.0),
    //                 FirstOrderMovement {
    //                     speed: 2.0,
    //                     jitter: 0.1,
    //                 },
    //                 SingleAbilityTimed::new(ability, 0.2),
    //                 normal_drop_table.clone(),
    //             ),
    //         ).with_num_slots(20);

    //         registry.register_enemy(ranger);
    //     }

    //     registry.register_enemy(summoner);
    // }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, register_enemies);
}
