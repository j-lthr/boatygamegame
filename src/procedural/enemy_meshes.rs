use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};

/// Generate a spiky crystal-like mesh for aggressive enemies
pub fn generate_crystal_mesh(radius: f32, spikes: usize) -> Mesh {
    let mut positions = Vec::new();
    let mut indices = Vec::new();

    // Center vertex
    positions.push(Vec3::ZERO);

    // Generate spikes around the circumference
    for i in 0..spikes {
        let angle = (i as f32 / spikes as f32) * 2.0 * std::f32::consts::PI;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;

        // Base of spike
        positions.push(Vec3::new(x * 0.6, 0.0, z * 0.6));
        // Tip of spike
        positions.push(Vec3::new(x, radius * 0.8, z));
        // Side vertices for spike width
        let side_angle1 = angle + 0.2;
        let side_angle2 = angle - 0.2;
        positions.push(Vec3::new(
            side_angle1.cos() * radius * 0.7,
            radius * 0.2,
            side_angle1.sin() * radius * 0.7,
        ));
        positions.push(Vec3::new(
            side_angle2.cos() * radius * 0.7,
            radius * 0.2,
            side_angle2.sin() * radius * 0.7,
        ));
    }

    // Generate triangles for spikes
    for i in 0..spikes {
        let base_idx = 1 + i * 4;
        let next_base_idx = 1 + ((i + 1) % spikes) * 4;

        // Connect center to base
        indices.extend_from_slice(&[0, base_idx as u32, next_base_idx as u32]);

        // Create spike triangles
        indices.extend_from_slice(&[
            base_idx as u32,
            (base_idx + 1) as u32,
            (base_idx + 2) as u32,
        ]);
        indices.extend_from_slice(&[
            base_idx as u32,
            (base_idx + 2) as u32,
            (base_idx + 3) as u32,
        ]);
        indices.extend_from_slice(&[
            (base_idx + 1) as u32,
            (base_idx + 2) as u32,
            (base_idx + 3) as u32,
        ]);
    }

    create_mesh_from_data(positions, indices)
}

/// Generate a segmented worm-like mesh for summoner enemies
pub fn generate_segmented_mesh(radius: f32, segments: usize, segment_height: f32) -> Mesh {
    let mut positions = Vec::new();
    let mut indices = Vec::new();

    let sides = 8; // Octagonal cross-section

    // Generate vertices for each segment
    for segment in 0..=segments {
        let y = segment as f32 * segment_height;
        let segment_radius = radius * (1.0 - (segment as f32 / segments as f32) * 0.3); // Taper

        for side in 0..sides {
            let angle = (side as f32 / sides as f32) * 2.0 * std::f32::consts::PI;
            let x = angle.cos() * segment_radius;
            let z = angle.sin() * segment_radius;
            positions.push(Vec3::new(x, y, z));
        }
    }

    // Generate triangles between segments
    for segment in 0..segments {
        for side in 0..sides {
            let current_base = segment * sides + side;
            let next_base = (segment + 1) * sides + side;
            let current_next = segment * sides + (side + 1) % sides;
            let next_next = (segment + 1) * sides + (side + 1) % sides;

            // Two triangles per quad
            indices.extend_from_slice(&[
                current_base as u32,
                next_base as u32,
                current_next as u32,
            ]);
            indices.extend_from_slice(&[current_next as u32, next_base as u32, next_next as u32]);
        }
    }

    create_mesh_from_data(positions, indices)
}

/// Generate an angular geometric mesh for robotic/mechanical enemies
pub fn generate_angular_mesh(size: f32) -> Mesh {
    let mut positions = Vec::new();
    let mut indices = Vec::new();

    // Create a truncated octahedron shape
    let vertices = vec![
        // Top pyramid
        Vec3::new(0.0, size, 0.0),
        Vec3::new(size * 0.7, size * 0.5, 0.0),
        Vec3::new(0.0, size * 0.5, size * 0.7),
        Vec3::new(-size * 0.7, size * 0.5, 0.0),
        Vec3::new(0.0, size * 0.5, -size * 0.7),
        // Middle section
        Vec3::new(size * 0.8, 0.0, size * 0.8),
        Vec3::new(-size * 0.8, 0.0, size * 0.8),
        Vec3::new(-size * 0.8, 0.0, -size * 0.8),
        Vec3::new(size * 0.8, 0.0, -size * 0.8),
        // Bottom pyramid
        Vec3::new(0.0, -size, 0.0),
        Vec3::new(size * 0.7, -size * 0.5, 0.0),
        Vec3::new(0.0, -size * 0.5, size * 0.7),
        Vec3::new(-size * 0.7, -size * 0.5, 0.0),
        Vec3::new(0.0, -size * 0.5, -size * 0.7),
    ];

    positions.extend_from_slice(&vertices);

    // Define faces manually for angular look
    let faces = vec![
        // Top pyramid
        [0, 1, 2],
        [0, 2, 3],
        [0, 3, 4],
        [0, 4, 1],
        // Middle connecting faces
        [1, 5, 2],
        [2, 6, 3],
        [3, 7, 4],
        [4, 8, 1],
        [5, 6, 2],
        [6, 7, 3],
        [7, 8, 4],
        [8, 5, 1],
        // Bottom pyramid
        [9, 11, 10],
        [9, 12, 11],
        [9, 13, 12],
        [9, 10, 13],
        // Bottom connecting faces
        [10, 11, 5],
        [11, 12, 6],
        [12, 13, 7],
        [13, 10, 8],
        [11, 6, 5],
        [12, 7, 6],
        [13, 8, 7],
        [10, 5, 8],
    ];

    for face in faces {
        indices.extend_from_slice(&[face[0] as u32, face[1] as u32, face[2] as u32]);
    }

    create_mesh_from_data(positions, indices)
}

/// Generate a simple multi-faced gem for minion enemies
pub fn generate_gem_mesh(radius: f32) -> Mesh {
    let mut positions = Vec::new();
    let mut indices = Vec::new();

    // Top point
    positions.push(Vec3::new(0.0, radius, 0.0));

    // Middle ring
    let ring_points = 6;
    for i in 0..ring_points {
        let angle = (i as f32 / ring_points as f32) * 2.0 * std::f32::consts::PI;
        let x = angle.cos() * radius * 0.8;
        let z = angle.sin() * radius * 0.8;
        positions.push(Vec3::new(x, 0.0, z));
    }

    // Bottom point
    positions.push(Vec3::new(0.0, -radius, 0.0));

    // Top faces
    for i in 0..ring_points {
        let next = (i + 1) % ring_points;
        indices.extend_from_slice(&[0, (i + 1) as u32, (next + 1) as u32]);
    }

    // Bottom faces
    for i in 0..ring_points {
        let next = (i + 1) % ring_points;
        indices.extend_from_slice(&[(ring_points + 1) as u32, (next + 1) as u32, (i + 1) as u32]);
    }

    create_mesh_from_data(positions, indices)
}

/// Generate a twisted pillar shape for ranged enemies
pub fn generate_twisted_pillar(radius: f32, height: f32, twist_amount: f32) -> Mesh {
    let mut positions = Vec::new();
    let mut indices = Vec::new();

    let segments = 8;
    let sides = 6;

    // Generate twisted vertices
    for segment in 0..=segments {
        let t = segment as f32 / segments as f32;
        let y = t * height - height * 0.5;
        let twist = t * twist_amount;
        let segment_radius = radius * (1.0 - t * 0.3); // Slight taper

        for side in 0..sides {
            let angle = (side as f32 / sides as f32) * 2.0 * std::f32::consts::PI + twist;
            let x = angle.cos() * segment_radius;
            let z = angle.sin() * segment_radius;
            positions.push(Vec3::new(x, y, z));
        }
    }

    // Generate faces
    for segment in 0..segments {
        for side in 0..sides {
            let current = segment * sides + side;
            let next_segment = (segment + 1) * sides + side;
            let current_next = segment * sides + (side + 1) % sides;
            let next_segment_next = (segment + 1) * sides + (side + 1) % sides;

            indices.extend_from_slice(&[current as u32, next_segment as u32, current_next as u32]);
            indices.extend_from_slice(&[
                current_next as u32,
                next_segment as u32,
                next_segment_next as u32,
            ]);
        }
    }

    create_mesh_from_data(positions, indices)
}

fn create_mesh_from_data(positions: Vec<Vec3>, indices: Vec<u32>) -> Mesh {
    // Calculate normals
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

    // Generate simple UVs
    let uvs: Vec<Vec2> = positions
        .iter()
        .map(|pos| {
            let normalized = pos.normalize();
            let u = 0.5 + normalized.z.atan2(normalized.x) / (2.0 * std::f32::consts::PI);
            let v = 0.5 - normalized.y.asin() / std::f32::consts::PI;
            Vec2::new(u, v)
        })
        .collect();

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
