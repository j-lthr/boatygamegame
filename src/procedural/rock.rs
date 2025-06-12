// src/procedural/rock.rs
use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};
use noise::{NoiseFn, Perlin, Seedable};
use std::collections::HashMap;

/// Component to mark entities as procedural rocks
#[derive(Component)]
pub struct ProceduralRock {
    pub config: RockConfig,
    pub regeneration_timer: Option<Timer>,
}

/// Configuration for procedural rock generation
#[derive(Clone, Debug)]
pub struct RockConfig {
    /// Base radius of the rock
    pub radius: f32,
    /// Number of subdivisions (higher = more detailed, but still low-poly)
    pub subdivisions: usize,
    /// Strength of the noise displacement
    pub noise_strength: f32,
    /// Scale of the noise (smaller = more detailed features)
    pub noise_scale: f32,
    /// Random seed for the noise
    pub seed: u32,
    /// Number of octaves for fractal noise
    pub octaves: usize,
    /// Frequency multiplier between octaves
    pub lacunarity: f32,
    /// Amplitude multiplier between octaves  
    pub persistence: f32,
}

impl Default for RockConfig {
    fn default() -> Self {
        Self {
            radius: 1.0,
            subdivisions: 2,
            noise_strength: 0.3,
            noise_scale: 2.0,
            seed: 42,
            octaves: 3,
            lacunarity: 2.0,
            persistence: 0.5,
        }
    }
}

impl RockConfig {
    /// Create a small pebble configuration
    pub fn pebble(seed: u32) -> Self {
        Self {
            radius: 0.3,
            subdivisions: 1,
            noise_strength: 0.2,
            seed,
            ..Default::default()
        }
    }

    /// Create a medium boulder configuration
    pub fn boulder(seed: u32) -> Self {
        Self {
            radius: 2.0,
            subdivisions: 2,
            noise_strength: 0.5,
            octaves: 4,
            seed,
            ..Default::default()
        }
    }

    /// Create a large cliff rock configuration
    pub fn cliff_rock(seed: u32) -> Self {
        Self {
            radius: 3.0,
            subdivisions: 2,
            noise_strength: 0.8,
            noise_scale: 1.5,
            octaves: 5,
            seed,
            ..Default::default()
        }
    }

    /// Randomize the configuration with a base seed
    pub fn randomize(&mut self, base_seed: u32) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        base_seed.hash(&mut hasher);
        let hash = hasher.finish();

        self.seed = hash as u32;
        self.noise_strength *= 0.7 + (hash % 60) as f32 / 100.0; // 0.7-1.3x variation
        self.noise_scale *= 0.8 + (hash >> 16) as f32 / 100.0; // 0.8-1.2x variation
    }
}

/// Generate a procedural low-poly rock mesh
pub fn generate_rock_mesh(config: &RockConfig) -> Mesh {
    let mut positions = Vec::new();
    let mut indices = Vec::new();

    // Generate icosphere vertices and triangles
    generate_icosphere(&mut positions, &mut indices, config.subdivisions);

    // Initialize noise generator
    let perlin = Perlin::new(config.seed);

    // Displace vertices using fractal noise
    for position in &mut positions {
        let noise_value = fractal_noise(
            &perlin,
            *position,
            config.octaves,
            config.lacunarity,
            config.persistence,
            config.noise_scale,
        );

        // Normalize the position to get direction, then apply displacement
        let direction = position.normalize();
        let displacement = direction * noise_value * config.noise_strength;

        // Apply base radius and displacement
        *position = direction * config.radius + displacement;
    }

    // Calculate normals (flat shading for low-poly look)
    let normals = calculate_flat_normals(&positions, &indices);

    // Generate UVs (simple spherical mapping)
    let uvs = generate_spherical_uvs(&positions);

    // Create the mesh
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

// Private helper functions...

fn generate_icosphere(positions: &mut Vec<Vec3>, indices: &mut Vec<u32>, subdivisions: usize) {
    let phi = (1.0 + 5.0_f32.sqrt()) / 2.0;
    let inv_norm = 1.0 / (1.0 + phi * phi).sqrt();

    let initial_vertices = vec![
        Vec3::new(-1.0, phi, 0.0) * inv_norm,
        Vec3::new(1.0, phi, 0.0) * inv_norm,
        Vec3::new(-1.0, -phi, 0.0) * inv_norm,
        Vec3::new(1.0, -phi, 0.0) * inv_norm,
        Vec3::new(0.0, -1.0, phi) * inv_norm,
        Vec3::new(0.0, 1.0, phi) * inv_norm,
        Vec3::new(0.0, -1.0, -phi) * inv_norm,
        Vec3::new(0.0, 1.0, -phi) * inv_norm,
        Vec3::new(phi, 0.0, -1.0) * inv_norm,
        Vec3::new(phi, 0.0, 1.0) * inv_norm,
        Vec3::new(-phi, 0.0, -1.0) * inv_norm,
        Vec3::new(-phi, 0.0, 1.0) * inv_norm,
    ];

    positions.extend_from_slice(&initial_vertices);

    let initial_faces = vec![
        [0, 11, 5],
        [0, 5, 1],
        [0, 1, 7],
        [0, 7, 10],
        [0, 10, 11],
        [1, 5, 9],
        [5, 11, 4],
        [11, 10, 2],
        [10, 7, 6],
        [7, 1, 8],
        [3, 9, 4],
        [3, 4, 2],
        [3, 2, 6],
        [3, 6, 8],
        [3, 8, 9],
        [4, 9, 5],
        [2, 4, 11],
        [6, 2, 10],
        [8, 6, 7],
        [9, 8, 1],
    ];

    for face in initial_faces {
        indices.push(face[0] as u32);
        indices.push(face[1] as u32);
        indices.push(face[2] as u32);
    }

    for _ in 0..subdivisions {
        subdivide_mesh(positions, indices);
    }
}

fn subdivide_mesh(positions: &mut Vec<Vec3>, indices: &mut Vec<u32>) {
    let mut new_indices = Vec::new();
    let mut edge_cache: HashMap<(u32, u32), u32> = HashMap::new();

    for chunk in indices.chunks(3) {
        let [a, b, c] = [chunk[0], chunk[1], chunk[2]];

        let ab = get_or_create_midpoint(positions, &mut edge_cache, a, b);
        let bc = get_or_create_midpoint(positions, &mut edge_cache, b, c);
        let ca = get_or_create_midpoint(positions, &mut edge_cache, c, a);

        new_indices.extend_from_slice(&[a, ab, ca]);
        new_indices.extend_from_slice(&[b, bc, ab]);
        new_indices.extend_from_slice(&[c, ca, bc]);
        new_indices.extend_from_slice(&[ab, bc, ca]);
    }

    *indices = new_indices;
}

fn get_or_create_midpoint(
    positions: &mut Vec<Vec3>,
    edge_cache: &mut HashMap<(u32, u32), u32>,
    a: u32,
    b: u32,
) -> u32 {
    let key = if a < b { (a, b) } else { (b, a) };

    if let Some(&existing) = edge_cache.get(&key) {
        return existing;
    }

    let pos_a = positions[a as usize];
    let pos_b = positions[b as usize];
    let midpoint = ((pos_a + pos_b) * 0.5).normalize();

    let new_index = positions.len() as u32;
    positions.push(midpoint);
    edge_cache.insert(key, new_index);

    new_index
}

fn fractal_noise(
    noise: &Perlin,
    position: Vec3,
    octaves: usize,
    lacunarity: f32,
    persistence: f32,
    scale: f32,
) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = scale;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        let sample_point = [
            position.x as f64 * frequency as f64,
            position.y as f64 * frequency as f64,
            position.z as f64 * frequency as f64,
        ];

        value += noise.get(sample_point) as f32 * amplitude;
        max_value += amplitude;

        amplitude *= persistence;
        frequency *= lacunarity;
    }

    value / max_value
}

fn calculate_flat_normals(positions: &[Vec3], indices: &[u32]) -> Vec<Vec3> {
    let mut normals = vec![Vec3::ZERO; positions.len()];
    let mut counts = vec![0; positions.len()];

    for chunk in indices.chunks(3) {
        let [i0, i1, i2] = [chunk[0] as usize, chunk[1] as usize, chunk[2] as usize];
        let v0 = positions[i0];
        let v1 = positions[i1];
        let v2 = positions[i2];

        let face_normal = (v1 - v0).cross(v2 - v0).normalize();

        normals[i0] += face_normal;
        normals[i1] += face_normal;
        normals[i2] += face_normal;
        counts[i0] += 1;
        counts[i1] += 1;
        counts[i2] += 1;
    }

    for (normal, count) in normals.iter_mut().zip(counts.iter()) {
        if *count > 0 {
            *normal = (*normal / *count as f32).normalize();
        }
    }

    normals
}

fn generate_spherical_uvs(positions: &[Vec3]) -> Vec<Vec2> {
    positions
        .iter()
        .map(|pos| {
            let normalized = pos.normalize();
            let u = 0.5 + normalized.z.atan2(normalized.x) / (2.0 * std::f32::consts::PI);
            let v = 0.5 - normalized.y.asin() / std::f32::consts::PI;
            Vec2::new(u, v)
        })
        .collect()
}
