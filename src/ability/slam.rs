use std::time::Duration;
use bevy::audio::Volume;
use bevy::prelude::*;
use crate::fx;
use crate::common;
use crate::event;
use super::*;

#[derive(Clone)]
pub struct Slam {
    pub range: f32,
    pub damage: i32,
    pub knockback_force: f32,
}

#[derive(Copy, Clone)]
pub struct SlamParams {
    pub target_position: Vec3,
}

#[derive(Component)]
pub struct SlamFadeEffect {
    pub radius: f32,
    pub alpha: f32,
}

pub fn cast_slam(
    mut cast_events: EventReader<CastEvent<Slam>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut slam_sounds: ResMut<Assets<fx::fm::FMSound>>,
    mut entity_query: Query<(Entity, &mut Transform, Option<&common::Living>)>,
    mut damage_events: EventWriter<event::DamageEvent>,
) {
    for cast_event in cast_events.read() {
        if let Ok((_, caster_transform, _)) = entity_query.get(cast_event.caster) {
            let caster_pos = caster_transform.translation;
            
            // Find all entities within range, excluding the caster
            for (entity, mut target_transform, living_opt) in entity_query.iter_mut() {
                // Skip self-damage
                if entity == cast_event.caster {
                    continue;
                }
                
                let distance = caster_pos.distance(target_transform.translation);
                
                if distance <= cast_event.ability.range {
                    // Apply damage if target has Living component
                    if living_opt.is_some() {
                        // Emit damage event instead of directly modifying health
                        damage_events.write(event::DamageEvent {
                            target: entity,
                            source: Some(cast_event.caster),
                            damage: cast_event.ability.damage,
                            position: target_transform.translation,
                        });
                    }
                    
                    // Apply knockback
                    let knockback_vector = (target_transform.translation - caster_pos)
                        .normalize_or_zero()
                        .with_y(0.0)
                        * cast_event.ability.knockback_force;
                    
                    target_transform.translation += knockback_vector;
                }
            }
            
            // Spawn visual effect
            spawn_slam_effect(
                &mut commands,
                &mut meshes,
                &mut materials,
                caster_pos,
                cast_event.ability.range,
            );
            
            // Play sound
            let slam_sound_handle = slam_sounds.add(fx::fm::FMSound {
                config: fx::fm::SLAM,
                duration: Duration::from_millis(400),
            });
            
            commands.spawn((
                AudioPlayer(slam_sound_handle),
                PlaybackSettings::DESPAWN
                    .with_spatial(true)
                    .with_volume(Volume::Decibels(18.0)),
                Transform::from_translation(caster_pos),
            ));
        }
    }
}

pub fn spawn_slam_effect(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    radius: f32,
) {
    let circle_mesh = meshes.add(Cylinder::new(2.0, 0.1));
    
    let slam_material = materials.add(StandardMaterial {
        base_color: Color::srgba(2.0, 1.0, 1.0, 0.8),
        emissive: Color::srgb(2.0, 1.0, 1.0).into(),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    
    commands.spawn((
        Mesh3d(circle_mesh),
        MeshMaterial3d(slam_material),
        Transform::from_translation(position + Vec3::new(0.0, 0.05, 0.0)),
        SlamFadeEffect {
            radius: radius * 1.1,
            alpha: 1.0,
        },
    ));
}

pub fn animate_fade_effects(
    mut commands: Commands,
    mut slam_effect_query: Query<(Entity, &mut Transform, &mut SlamFadeEffect)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut slam_effect) in &mut slam_effect_query {
        let fade_speed = 10.0;
        slam_effect.alpha -= fade_speed * time.delta_secs();
        
        slam_effect.radius += 3.0 * time.delta_secs();
        transform.scale = Vec3::splat(slam_effect.radius / 2.0);
        
        if let Ok(material_handle) = material_query.get(entity) {
            if let Some(material) = materials.get_mut(&material_handle.0) {
                material.base_color.set_alpha(slam_effect.alpha.max(0.0));
                let emissive_intensity = slam_effect.alpha.max(0.0) * 5.0;
                material.emissive =
                    Color::srgb(emissive_intensity, emissive_intensity * 0.3, 0.0).into();
            }
        }
        
        if slam_effect.alpha <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

impl Ability for Slam {
    type CastParams = SlamParams;

    fn add_systems(app: &mut bevy::app::App) {
        app.add_systems(Update, (cast_slam, animate_fade_effects));
    }
}