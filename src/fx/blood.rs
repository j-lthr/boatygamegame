use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{
    common::HealthPool,
    event::DeathEvent,
    init::DespawnOnReset,
};

// Enhanced blood particle component with realistic properties
#[derive(Component)]
pub struct Particle {
    velocity: Vec3,
    lifetime: f32,
    max_lifetime: f32,
    size_factor: f32,
    is_stuck: bool,
    stick_surface: Vec3, // Normal of the surface it's stuck to
}

/// Enhanced function to spawn realistic blood explosion particles
pub fn spawn_particles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials_: ResMut<Assets<StandardMaterial>>,
    mut death_events: EventReader<DeathEvent>,
    transform_query: Query<(&Transform, &MeshMaterial3d<StandardMaterial>, &HealthPool)>,
) {
    let mut spawn_particles =
        |position: Vec3,
         particle_count: usize,
         explosion_force_base: f32,
         base_velocity: Vec3,
         blood_color: Color,
         materials: &mut ResMut<Assets<StandardMaterial>>| {
            // Create varied particle meshes for realism
            let small_sphere = meshes.add(Sphere::new(0.02));
            let medium_sphere = meshes.add(Sphere::new(0.04));
            let large_sphere = meshes.add(Sphere::new(0.07));
            let particle_meshes = [small_sphere, medium_sphere, large_sphere];

            for _ in 0..particle_count {
                // Random direction with bias toward horizontal spread
                let angle_y = fastrand::f32() * 2.0 * PI;
                let angle_x = (fastrand::f32() - 0.5) * PI * 0.4; // Biased toward horizontal

                let direction = Vec3::new(
                    angle_y.cos() * angle_x.cos(),
                    0.3 + fastrand::f32() * 0.4, // Upward but not too high
                    angle_y.sin() * angle_x.cos(),
                )
                .normalize();

                // Varied explosion force for realistic spread
                let force_multiplier = 0.5 + fastrand::f32() * 1.5;
                let initial_velocity =
                    direction * explosion_force_base * force_multiplier + base_velocity;

                // Small random offset from explosion center
                let offset = Vec3::new(
                    (fastrand::f32() - 0.5) * 0.3,
                    0.1 + fastrand::f32() * 0.1,
                    (fastrand::f32() - 0.5) * 0.3,
                );

                // Choose random particle size and corresponding material
                let size_type = fastrand::usize(0..3);
                let particle_mesh = particle_meshes[size_type].clone();

                // Determine initial blood properties
                let size_factor = match size_type {
                    0 => 0.7 + fastrand::f32() * 0.3, // Small particles
                    1 => 1.0 + fastrand::f32() * 0.3, // Medium particles
                    _ => 1.3 + fastrand::f32() * 0.4, // Large particles
                };

                let max_lifetime = 1.0 + fastrand::f32() * 0.5;

                commands.spawn((
                    Mesh3d(particle_mesh),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgba(1.0, 1.0, 1.0, 1.0),
                        emissive: blood_color.into(),
                        ..Default::default()
                    })),
                    Transform::from_translation(position + offset),
                    Particle {
                        velocity: initial_velocity,
                        lifetime: 0.0,
                        max_lifetime,
                        size_factor,
                        is_stuck: false,
                        stick_surface: Vec3::ZERO,
                    },
                    DespawnOnReset,
                ));
            }
        };

    for death_event in death_events.read() {
        if let Ok((transform, material, living)) = transform_query.get(death_event.entity) {
            let color: Color = materials_.get(material.id()).unwrap().emissive.into();
            spawn_particles(
                transform.translation,
                living.max_health as usize,
                4.0,
                Vec3::ZERO,
                color,
                &mut materials_,
            );
            info!("Spawning particles at {}", transform.translation);
        }
    }
}

/// Enhanced physics system for blood particles with realistic behavior
pub fn handle_particle_physics(
    mut particle_query: Query<(&mut Transform, &mut Particle)>,
    time: Res<Time>,
) {
    //let particles: Vec<_> = particle_query.iter().map(|(t, p)| (t.translation, p.velocity)).collect();

    for (mut transform, mut particle) in &mut particle_query {
        particle.lifetime += time.delta_secs();

        // Apply gravity
        let gravity = Vec3::new(0.0, 0.0, 0.0);
        particle.velocity += gravity * time.delta_secs();

        // Apply viscosity - blood gets thicker over time
        let age_factor = (particle.lifetime / particle.max_lifetime).clamp(0.0, 1.0);

        // Air resistance
        let air_resistance =
            particle.velocity * particle.velocity.length() * 0.01 * time.delta_secs();
        particle.velocity -= air_resistance;

        if particle.is_stuck {
            continue; // Skip physics for stuck particles
        }

        // Integrate position
        transform.translation += particle.velocity * time.delta_secs();

        // Enhanced collision detection and sticking
        let ground_level = -10.0;
        if transform.translation.y <= ground_level {
            transform.translation.y = ground_level;

            // Determine if particle should stick based on velocity and blood properties
            let stick_threshold = 2.0 * (1.0 - age_factor * 0.5); // Older blood sticks easier
            let impact_speed = particle.velocity.length();

            if impact_speed < stick_threshold {
                // Particle sticks to surface
                particle.is_stuck = true;
                particle.velocity = Vec3::ZERO;
                particle.stick_surface = Vec3::Y;

                // Create a small splatter decal for visual effect
                // (This would be implemented with a separate system)
            } else {
                // Bounce with energy loss and splatter
                particle.velocity.y = -particle.velocity.y * 0.3;
                particle.velocity.x *= 0.6;
                particle.velocity.z *= 0.6;
            }
        }

        /*// Boundary collision with sticking
        let boundary = 15.0;
        if transform.translation.x.abs() > boundary {
            if particle.velocity.length() < 3.0 {
                particle.is_stuck = true;
                particle.velocity = Vec3::ZERO;
                particle.stick_surface = Vec3::new(-transform.translation.x.signum(), 0.0, 0.0);
            } else {
                particle.velocity.x = -particle.velocity.x * 0.4;
                transform.translation.x = transform.translation.x.signum() * boundary;
            }
        }
        if transform.translation.z.abs() > boundary {
            if particle.velocity.length() < 3.0 {
                particle.is_stuck = true;
                particle.velocity = Vec3::ZERO;
                particle.stick_surface = Vec3::new(0.0, 0.0, -transform.translation.z.signum());
            } else {
                particle.velocity.z = -particle.velocity.z * 0.4;
                transform.translation.z = transform.translation.z.signum() * boundary;
            }
        }*/
    }
}

/// System to update blood particle rendering based on age and state
pub fn blood_particle_rendering(
    mut particle_query: Query<(
        &mut Transform,
        &Particle,
    )>,
) {
    for ( mut transform, particle) in &mut particle_query {
        let age_ratio = (particle.lifetime / particle.max_lifetime).clamp(0.0, 1.0);

        // Scale particles slightly based on size factor and age
        let age_scale = 1.0 + age_ratio * 0.2; // Slight expansion as blood coagulates
        let final_scale = particle.size_factor * age_scale;
        transform.scale = Vec3::splat(final_scale);

        // Stuck particles might flatten slightly
        if particle.is_stuck {
            let stick_direction = particle.stick_surface;
            if stick_direction.y.abs() > 0.5 {
                // Stuck to ground - flatten vertically
                transform.scale.y *= 0.3;
                transform.scale.x *= 1.2;
                transform.scale.z *= 1.2;
            } else {
                // Stuck to wall - flatten in the direction of the normal
                if stick_direction.x.abs() > 0.5 {
                    transform.scale.x *= 0.3;
                } else {
                    transform.scale.z *= 0.3;
                }
            }
        }
    }
}

/// System to cleanup old blood particles
pub fn cleanup_blood_particles(
    mut commands: Commands,
    particle_query: Query<(Entity, &Particle)>,
) {
    for (entity, particle) in &particle_query {
        if particle.lifetime > particle.max_lifetime {
            commands.entity(entity).despawn();
        }
    }
}
