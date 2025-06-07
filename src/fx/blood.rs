use std::f32::consts::PI;

use bevy::prelude::*;

use crate::enemy::boulder::BOULDER_COLOR;

// Enhanced blood particle component with realistic properties
#[derive(Component)]
pub struct BloodParticle {
    velocity: Vec3,
    initial_velocity: Vec3,
    density: f32,
    pressure: f32,
    lifetime: f32,
    max_lifetime: f32,
    size_factor: f32,
    viscosity_factor: f32,
    initial_color: Color,
    is_stuck: bool,
    stick_surface: Vec3, // Normal of the surface it's stuck to
}

// Component for blood splatter decals
#[derive(Component)]
pub struct BloodSplatter {
    fade_timer: f32,
    max_fade_time: f32,
}


// Resource to hold shared blood materials for performance
#[derive(Resource)]
pub struct BloodMaterials {
    fresh_blood: Handle<StandardMaterial>,
    medium_blood: Handle<StandardMaterial>,
    old_blood: Handle<StandardMaterial>,
    splatter_material: Handle<StandardMaterial>,
}


/// Setup blood materials with realistic properties
pub fn setup_blood_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let fresh_blood = materials.add(StandardMaterial {
        base_color: Color::srgba(0.8, 0.1, 0.1, 0.9), // Bright red, slightly transparent
        metallic: 0.0,
        alpha_mode: AlphaMode::Blend,
        unlit: false, // Keep lighting for realism
        emissive: BOULDER_COLOR.into(), // Slight glow
        ..default()
    });

    let medium_blood = materials.add(StandardMaterial {
        base_color: Color::srgba(0.6, 0.08, 0.08, 0.8), // Darker red
        metallic: 0.0,
        alpha_mode: AlphaMode::Blend,
        unlit: false,
        emissive: BOULDER_COLOR.into(),
        ..default()
    });

    let old_blood = materials.add(StandardMaterial {
        base_color: Color::srgba(0.4, 0.05, 0.02, 0.7), // Dark brown-red
        metallic: 0.0,
        alpha_mode: AlphaMode::Blend,
        unlit: false,
        emissive: BOULDER_COLOR.into(),
        ..default()
    });

    let splatter_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.5, 0.05, 0.05, 0.6),
        metallic: 0.0,
        alpha_mode: AlphaMode::Blend,
        unlit: true, // Splatters don't need complex lighting
        ..default()
    });

    commands.insert_resource(BloodMaterials {
        fresh_blood,
        medium_blood,
        old_blood,
        splatter_material,
    });
}


/// Enhanced function to spawn realistic blood explosion particles
pub fn spawn_blood_explosion(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    position: Vec3,
    blood_materials: &BloodMaterials,
    particle_count: usize,
    explosion_force_base: f32,
    base_velocity: Vec3,
    blood_color: Vec3,
) {

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
        ).normalize();
        
        // Varied explosion force for realistic spread
        let force_multiplier = 0.5 + fastrand::f32() * 1.5;
        let initial_velocity = direction * explosion_force_base * force_multiplier + base_velocity;
        
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
        
        let viscosity_factor = 0.8 + fastrand::f32() * 0.4;
        let max_lifetime = 5.0 + fastrand::f32() * 3.0;
        
        // Vary initial blood color slightly
        let color_variation = 0.9 + fastrand::f32() * 0.2;
        let initial_color = Color::srgba(
            blood_color.x * color_variation,
            blood_color.y * color_variation,
            blood_color.z * color_variation,
            0.9
        );
        
        commands.spawn((
            Mesh3d(particle_mesh),
            MeshMaterial3d(blood_materials.fresh_blood.clone()),
            Transform::from_translation(position + offset),
            BloodParticle {
                velocity: initial_velocity,
                initial_velocity,
                density: 1200.0,
                pressure: 0.0,
                lifetime: 0.0,
                max_lifetime,
                size_factor,
                viscosity_factor,
                initial_color,
                is_stuck: false,
                stick_surface: Vec3::ZERO,
            },
        ));
    }
}

/// Enhanced physics system for blood particles with realistic behavior
pub fn blood_particle_physics(
    mut particle_query: Query<(&mut Transform, &mut BloodParticle)>,
    time: Res<Time>,
) {
    //let particles: Vec<_> = particle_query.iter().map(|(t, p)| (t.translation, p.velocity)).collect();
    
    for (mut transform, mut particle) in &mut particle_query {

        particle.lifetime += time.delta_secs();
     
      
        
        // Apply gravity
        let gravity = Vec3::new(0.0, -9.81, 0.0);
        particle.velocity += gravity * time.delta_secs();
        
        // Apply viscosity - blood gets thicker over time
        let age_factor = (particle.lifetime / particle.max_lifetime).clamp(0.0, 1.0);

        // Air resistance
        let air_resistance = particle.velocity * particle.velocity.length() * 0.01 * time.delta_secs();
        particle.velocity -= air_resistance;

        if particle.is_stuck {
            continue; // Skip physics for stuck particles
        }
        
        
        // Integrate position
        transform.translation += particle.velocity * time.delta_secs();
        
        // Enhanced collision detection and sticking
        let ground_level = 0.02;
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
    mut particle_query: Query<(&mut MeshMaterial3d<StandardMaterial>, &mut Transform, &BloodParticle)>,
    blood_materials: Res<BloodMaterials>,
) {
    for (mut material, mut transform, particle) in &mut particle_query {
        let age_ratio = (particle.lifetime / particle.max_lifetime).clamp(0.0, 1.0);
        
        // Change material based on blood age
        let new_material = if age_ratio < 0.3 {
            blood_materials.fresh_blood.clone()
        } else if age_ratio < 0.7 {
            blood_materials.medium_blood.clone()
        } else {
            blood_materials.old_blood.clone()
        };
        
        if material.0 != new_material {
            material.0 = new_material;
        }
        
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
    particle_query: Query<(Entity, &BloodParticle)>,
) {
    for (entity, particle) in &particle_query {
        if particle.lifetime > particle.max_lifetime {
            commands.entity(entity).despawn();
        }
    }
}

/// System to handle blood splatter fading (placeholder for now)
pub fn fade_blood_splatters(
    mut commands: Commands,
    mut splatter_query: Query<(Entity, &mut BloodSplatter)>,
    time: Res<Time>,
) {
    for (entity, mut splatter) in &mut splatter_query {
        splatter.fade_timer += time.delta_secs();
        if splatter.fade_timer > splatter.max_fade_time {
            commands.entity(entity).despawn();
        }
    }
}