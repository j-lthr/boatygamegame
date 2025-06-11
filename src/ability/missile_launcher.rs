use super::*;
use crate::fx;

// Missile Launcher System
//
// Features:
// - Missiles track entities for most of their lifetime (lock_time_ratio)
// - Near end of lifetime, switches to position lock to allow dodging
// - Area damage explosions with visual and audio effects
// - Configurable tracking strength, explosion radius, and damage falloff
//
// Required additions to fx module:
// - fx::fm::MISSILE_LAUNCH_SOUND
// - fx::fm::EXPLOSION_SOUND

#[derive(Clone)]
pub struct MissileLauncher {
    pub missile_count: i32,
    pub spread: f32,
    pub speed: f32,
    pub lifetime: f32,
    pub damage: i32,
    pub color: Color,
    pub tracking_strength: f32, // How aggressively missiles track targets (0.0 - 1.0)
    pub explosion_radius: f32,  // Radius of explosion on impact
    pub lock_distance: f32, // Portion of lifetime spent tracking entity (0.7 = 70% tracking, 30% locked)
}

#[derive(Component)]
pub struct ExplosionEffect {
    pub position: Vec3,
    pub radius: f32,
    pub damage: i32,
    pub source: Entity,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

use crate::common;
use crate::event;

#[derive(Copy, Clone)]
pub struct MissileParams {
    pub target_entity: Entity,
}

#[derive(Clone)]
pub enum MissileTarget {
    Entity(Entity),
    Locked(Vec3),
}

#[derive(Component)]
pub struct Missile {
    target: MissileTarget,
    tracking_strength: f32,
    explosion_radius: f32,
    lifetime_remaining: f32,
    max_lifetime: f32,
    lock_distance: f32,
    damage: i32,
    source: Entity,
    speed: f32,
}

pub fn cast_missiles(
    mut cast_events: EventReader<CastEvent<MissileLauncher>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    shoot_sounds: ResMut<Assets<fx::fm::FMSound>>,
    caster_query: Query<&Transform, Without<crate::projectile::Projectile>>,
) {
    for cast_event in cast_events.read() {
        if let Ok(caster_transform) = caster_query.get(cast_event.caster) {
            // Create missile mesh (elongated cylinder for rocket shape)
            let missile_mesh = meshes.add(Capsule3d::new(0.03, 0.15));
            let missile_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 1.0, 1.0),
                emissive: cast_event.ability.color.into(),
                ..Default::default()
            });

            for i in 0..cast_event.ability.missile_count {
                // Calculate initial direction with spread

                let spread_offset = Vec3::new(
                    (fastrand::f32() - 0.5) * cast_event.ability.spread,
                    (fastrand::f32() - 0.5) * cast_event.ability.spread * 0.5, // Less vertical spread
                    (fastrand::f32() - 0.5) * cast_event.ability.spread,
                );

                // Determine target for this missile
                let target = MissileTarget::Entity(cast_event.params.target_entity);

                // Spawn missile
                let missile_entity = commands
                    .spawn((
                        Mesh3d(missile_mesh.clone()),
                        MeshMaterial3d(missile_mat.clone()),
                        Transform::from_translation(
                            caster_transform.translation + Vec3::Y * 0.5 + spread_offset,
                        ),
                        Missile {
                            target: target.clone(),
                            tracking_strength: cast_event.ability.tracking_strength,
                            explosion_radius: cast_event.ability.explosion_radius,
                            lifetime_remaining: cast_event.ability.lifetime,
                            max_lifetime: cast_event.ability.lifetime,
                            lock_distance: cast_event.ability.lock_distance,
                            damage: cast_event.ability.damage,
                            source: cast_event.caster,
                            speed: cast_event.ability.speed,
                        },
                    ))
                    .id();
            }
        }
    }
}

pub fn update_missiles(
    mut commands: Commands,
    mut missile_query: Query<(Entity, &mut Transform, &mut Missile)>,
    target_query: Query<&Transform, (Without<Missile>, Without<crate::projectile::Projectile>)>,
    time: Res<Time>,
) {
    for (missile_entity, mut missile_transform, mut missile) in missile_query.iter_mut() {
        // Update lifetime
        missile.lifetime_remaining -= time.delta_secs();
        if missile.lifetime_remaining <= 0.0 {
            explode_missile(
                &mut commands,
                missile_entity,
                missile_transform.translation,
                &missile,
            );
            continue;
        }

        // Calculate how much of lifetime has passed
        let lifetime_progress = 1.0 - (missile.lifetime_remaining / missile.max_lifetime);
        if let MissileTarget::Entity(target_entity) = missile.target {
            // Switch from entity tracking to position lock when we've used up the lock_time_ratio
            if let Ok(target_transform) = target_query.get(target_entity) {
                if target_transform
                    .translation
                    .distance(missile_transform.translation)
                    < missile.lock_distance
                {
                    // Lock onto the current position of the target
                    missile.target = MissileTarget::Locked(target_transform.translation);
                }
            }
        }

        // Get target position
        let target_position = match &missile.target {
            MissileTarget::Entity(entity) => {
                if let Ok(target_transform) = target_query.get(*entity) {
                    target_transform.translation
                } else {
                    // Target entity no longer exists, continue in current direction
                    missile_transform.translation + missile_transform.forward() * 10.0
                }
            }
            MissileTarget::Locked(pos) => *pos,
        };

        // Calculate desired direction towards target
        let to_target = (target_position - missile_transform.translation).normalize();
        let current_forward = missile_transform.forward();

        // Blend current direction with target direction based on tracking strength
        let new_direction = current_forward
            .lerp(
                to_target,
                missile.tracking_strength * time.delta_secs() * 2.0,
            )
            .normalize();

        // Update missile rotation to face new direction
        missile_transform.look_to(new_direction, Vec3::Y);

        // Move missile forward
        let speed = missile.speed; // You might want to make this configurable
        missile_transform.translation += new_direction * speed * time.delta_secs();

        // Check for collision with target (simple distance check)
        let distance_to_target = missile_transform.translation.distance(target_position);
        if distance_to_target < 0.5 {
            // Collision threshold
            explode_missile(
                &mut commands,
                missile_entity,
                missile_transform.translation,
                &missile,
            );
        }
    }
}

fn explode_missile(
    commands: &mut Commands,
    missile_entity: Entity,
    explosion_position: Vec3,
    missile: &Missile,
) {
    // Remove the missile
    commands.entity(missile_entity).despawn();

    // Spawn explosion effect
    commands.spawn(ExplosionEffect {
        position: explosion_position,
        radius: missile.explosion_radius,
        damage: missile.damage,
        source: missile.source,
        lifetime: 0.5, // Explosion lasts 0.5 seconds
        max_lifetime: 0.5,
    });
}

const EXPLOSION_EMISSIVE: Color = Color::srgb(100.0, 0.0, 0.0);

pub fn handle_explosions(
    mut commands: Commands,
    mut explosion_query: Query<(Entity, &mut ExplosionEffect)>,
    target_query: Query<
        (Entity, &Transform, &common::Living),
        (Without<ExplosionEffect>, Without<Missile>),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    explosion_sounds: ResMut<Assets<fx::fm::FMSound>>,
    mut damage_events: EventWriter<event::DamageEvent>,
    faction_query: Query<&common::Faction>,
    time: Res<Time>,
) {
    for (explosion_entity, mut explosion) in explosion_query.iter_mut() {
        // First frame: apply damage and spawn visual effects
        if explosion.lifetime >= explosion.max_lifetime {
            // Apply area damage
            for (target_entity, target_transform, _living) in target_query.iter() {
                // Skip self-damage
                if target_entity == explosion.source {
                    continue;
                }

                // Check faction compatibility to prevent friendly fire
                if let (Ok(source_faction), Ok(target_faction)) = (
                    faction_query.get(explosion.source),
                    faction_query.get(target_entity),
                ) {
                    if source_faction == target_faction {
                        continue;
                    }
                }

                let distance = explosion.position.distance(target_transform.translation);
                if distance <= explosion.radius {
                    // Calculate damage falloff (full damage at center, 25% at edge)
                    let damage_multiplier = (1.0 - (distance / explosion.radius) * 0.75).max(0.25);
                    let actual_damage = (explosion.damage as f32 * damage_multiplier) as i32;

                    // Emit damage event instead of directly modifying health
                    damage_events.write(event::DamageEvent {
                        target: target_entity,
                        source: Some(explosion.source),
                        damage: actual_damage,
                        position: target_transform.translation,
                        impact_velocity: None,
                    });
                }
            }

            // Spawn visual explosion effect
            let explosion_material = materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 0.5, 0.0, 0.8), // Orange with transparency
                emissive: EXPLOSION_EMISSIVE.into(),
                alpha_mode: AlphaMode::Blend,
                ..default()
            });

            // Create expanding sphere for explosion visual
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(1.0))),
                MeshMaterial3d(explosion_material),
                Transform::from_translation(explosion.position).with_scale(Vec3::splat(0.1)), // Start small
                ExplosionVisual {
                    max_scale: explosion.radius,
                    lifetime: 0.5,
                    max_lifetime: 0.5,
                },
            ));
        }

        // Update explosion lifetime
        explosion.lifetime -= time.delta_secs();

        // Remove explosion effect when lifetime expires
        if explosion.lifetime <= 0.0 {
            commands.entity(explosion_entity).despawn();
        }
    }
}

#[derive(Component)]
pub struct ExplosionVisual {
    max_scale: f32,
    lifetime: f32,
    max_lifetime: f32,
}

pub fn update_explosion_visuals(
    mut commands: Commands,
    mut explosion_visual_query: Query<(
        Entity,
        &mut Transform,
        &mut ExplosionVisual,
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

        let emissive_rgb = LinearRgba::from(EXPLOSION_EMISSIVE);

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

impl Ability for MissileLauncher {
    type CastParams = MissileParams;

    fn add_systems(app: &mut bevy::app::App) {
        app.add_systems(
            Update,
            (
                cast_missiles,
                update_missiles,
                handle_explosions,
                update_explosion_visuals,
            ),
        );
    }
}

impl Default for MissileLauncher {
    fn default() -> Self {
        Self {
            missile_count: 3,
            spread: 0.3,
            speed: 15.0,
            lifetime: 4.0,
            damage: 25,
            color: Color::srgb(1.0, 0.3, 0.0), // Orange glow
            tracking_strength: 0.8,
            explosion_radius: 2.0,
            lock_distance: 0.7, // Track entity for 70% of lifetime, then lock position
        }
    }
}
