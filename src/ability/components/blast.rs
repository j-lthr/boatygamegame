use bevy::prelude::*;
use bevy::render::mesh;

use crate::ability::CastInfo;
use crate::common;
use crate::event;
use crate::fx;

#[derive(Component, Clone)]
pub struct BlastDamage {
    pub radius: f32,
    pub damage: i32,
}

pub fn handle_blast_damage(
    mut commands: Commands,
    mut explosion_query: Query<(Entity, &BlastDamage, &CastInfo, &Transform)>,
    target_query: Query<(Entity, &Transform, &common::Living)>,
    mut damage_events: EventWriter<event::DamageEvent>,
    faction_query: Query<&common::Faction>,
) {
    for (explosion_entity, blast, cast_info, explosion_transform) in explosion_query.iter_mut() {
        // Apply area damage
        for (target_entity, target_transform, _living) in target_query.iter() {
            // Skip self-damage
            if target_entity == cast_info.caster {
                continue;
            }

            // Check faction compatibility to prevent friendly fire
            if let (Ok(source_faction), Ok(target_faction)) = (
                faction_query.get(cast_info.caster),
                faction_query.get(target_entity),
            ) {
                if source_faction == target_faction {
                    continue;
                }
            }

            let distance = explosion_transform
                .translation
                .distance(target_transform.translation);
            if distance <= blast.radius {
                // Calculate damage falloff (full damage at center, 25% at edge)
                let damage_multiplier = (1.0 - (distance / blast.radius) * 0.75).max(0.25);
                let actual_damage = (blast.damage as f32 * damage_multiplier) as i32;

                // Emit damage event instead of directly modifying health
                damage_events.write(event::DamageEvent {
                    target: target_entity,
                    source: Some(cast_info.caster),
                    damage: actual_damage,
                    position: target_transform.translation,
                    impact_velocity: None,
                });
            }
        }

        commands.entity(explosion_entity).remove::<BlastDamage>();
    }
}

#[derive(Component, Clone)]
pub struct BlastVisual {
    max_scale: f32,
    lifetime: f32,
    max_lifetime: f32,
    emissive_color: Color,
}

#[derive(Bundle, Clone)]
pub struct BlastBundle {
    mesh: Mesh3d,
    mesh_material: MeshMaterial3d<StandardMaterial>,
    blast_visual: BlastVisual,
    blast_damage: BlastDamage,
}

impl BlastBundle {
    pub fn new(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        color: Color,
        radius: f32,
        damage: i32,
        fade_duration: f32,
    ) -> Self {
        let explosion_material = materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.5, 0.0, 0.8), // Orange with transparency
            emissive: color.into(),
            alpha_mode: AlphaMode::Blend,
            ..default()
        });

        Self {
            mesh: Mesh3d(meshes.add(Sphere::new(1.0))),
            mesh_material: MeshMaterial3d(explosion_material),
            blast_visual: BlastVisual {
                max_scale: radius,
                lifetime: fade_duration,
                max_lifetime: fade_duration,
                emissive_color: color,
            },
            blast_damage: BlastDamage { radius, damage },
        }
    }
}

pub fn handle_blast_visual(
    mut commands: Commands,
    mut explosion_visual_query: Query<(
        Entity,
        &mut Transform,
        &mut BlastVisual,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut visual, material) in explosion_visual_query.iter_mut() {
        visual.lifetime -= time.delta_secs();

        if visual.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Calculate animation progress (0.0 = start, 1.0 = end)
        let progress = 1.0 - (visual.lifetime / visual.max_lifetime);

        // Scale up quickly, then fade
        let scale_progress = (progress * 3.0).min(1.0);
        let fade_progress = if progress > 0.7 {
            (progress - 0.7) / 0.3
        } else {
            0.0
        };

        let material = materials.get_mut(material.id()).unwrap();
        material.base_color.set_alpha(1.0 - fade_progress);

        let emissive_rgb = LinearRgba::from(visual.emissive_color);

        let brightness = 1.0 / (1e3 * progress + 1e-3) * (1.0 - fade_progress);

        material.emissive = LinearRgba::rgb(
            emissive_rgb.red * (brightness),
            emissive_rgb.green * (brightness),
            emissive_rgb.blue * (brightness),
        );

        transform.scale = Vec3::splat(scale_progress * visual.max_scale);

        // Fade out the explosion (you might need to update the material alpha)
        // This is a simplified approach - you may want to create separate materials
        // or use a shader for better control
    }
}

pub fn plugin(app: &mut bevy::app::App) {
    app.add_systems(Update, (handle_blast_damage, handle_blast_visual));
}
