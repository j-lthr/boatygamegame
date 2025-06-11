use super::*;
use crate::fx;
use crate::init::DespawnOnReset;
use crate::projectile;
use crate::utils::normal_dist_1d;
use bevy::audio::Volume;
use std::time::Duration;

#[derive(Clone)]
pub struct BasicProjectileAttack {
    pub bullet_count: i32,
    pub spread: f32,
    pub speed: f32,
    pub lifetime: f32,
    pub damage: i32,
    pub color: Color,
}

#[derive(Copy, Clone)]
pub struct BasicProjectileAttackParams {
    pub target_position: Vec3,
}

pub fn cast_basic_prjectile_attack(
    mut cast_events: EventReader<CastEvent<BasicProjectileAttack>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut shoot_sounds: ResMut<Assets<fx::fm::FMSound>>,
    caster_query: Query<&Transform, Without<crate::projectile::Projectile>>,
) {
    for cast_event in cast_events.read() {
        if let Ok(caster_transform) = caster_query.get(cast_event.caster) {
            // Play shooting sound (FM synthesis)
            let shoot_sound_handle = shoot_sounds.add(fx::fm::FMSound {
                config: fx::fm::GUN_SOUND,
                duration: Duration::from_millis(1000),
            });

            let bullet_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.2, 0.2, 0.2),
                metallic: 0.5,
                perceptual_roughness: 0.5,
                emissive: cast_event.ability.color.into(),
                ..default()
            });

            for i in 0..cast_event.ability.bullet_count {
                // Calculate bullet direction with spread
                let direction =
                    Quat::from_rotation_y(normal_dist_1d(0.0, 1.0) * cast_event.ability.spread)
                        * (cast_event.params.target_position - caster_transform.translation)
                            .normalize()
                            .with_y(0.0);

                // Spawn projectile
                let mut projectile = commands.spawn((
                    Mesh3d(meshes.add(Sphere::new(0.05 + 0.05 * fastrand::f32()))),
                    MeshMaterial3d(bullet_mat.clone()),
                    Transform::from_translation(caster_transform.translation),
                    projectile::Projectile {
                        direction,
                        speed: cast_event.ability.speed,
                        lifetime: cast_event.ability.lifetime,
                        damage: cast_event.ability.damage,
                        source: cast_event.caster,
                    },
                    DespawnOnReset
                ));

                if i == 0 {
                    projectile.insert((
                        AudioPlayer(shoot_sound_handle.clone()),
                        PlaybackSettings::ONCE
                            .with_spatial(true)
                            .with_volume(Volume::Decibels(24.0)),
                    ));
                }
            }
        }
    }
}

impl Ability for BasicProjectileAttack {
    type CastParams = BasicProjectileAttackParams;

    fn add_systems(app: &mut bevy::app::App) {
        app.add_systems(Update, cast_basic_prjectile_attack);
    }
}
