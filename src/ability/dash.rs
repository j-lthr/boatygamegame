use super::*;
use crate::fx;
use bevy::audio::Volume;
use std::time::Duration;

#[derive(Clone)]
pub struct Dash {
    pub range: f32,
}

#[derive(Copy, Clone)]
pub enum DashParams {
    Directional(Vec3),
    ToPosition(Vec3),
}

#[derive(Component)]
pub struct DashTrail {
    pub start_pos: Vec3,
    pub end_pos: Vec3,
    pub progress: f32,
    pub lifetime: Timer,
}

pub fn cast_dash(
    mut cast_events: EventReader<CastEvent<Dash>>,
    mut caster_query: Query<&mut Transform>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut dash_sounds: ResMut<Assets<fx::fm::FMSound>>,
) {
    for cast_event in cast_events.read() {
        if let Ok(mut caster_transform) = caster_query.get_mut(cast_event.caster) {
            let start_pos = caster_transform.translation;

            let delta = match cast_event.params {
                DashParams::Directional(direction) => {
                    direction.normalize() * cast_event.ability.range
                }
                DashParams::ToPosition(target_position) => (target_position
                    - caster_transform.translation)
                    .clamp_length_max(cast_event.ability.range),
            };

            let end_pos = start_pos + delta;
            caster_transform.translation = end_pos;

            // Spawn dash trail effect
            spawn_dash_trail(
                &mut commands,
                &mut meshes,
                &mut materials,
                start_pos,
                end_pos,
            );

            // Play dash sound
            let dash_sound_handle = dash_sounds.add(fx::fm::FMSound {
                config: fx::fm::DASH_SOUND,
                duration: Duration::from_millis(200),
            });

            commands.spawn((
                AudioPlayer(dash_sound_handle),
                PlaybackSettings::DESPAWN
                    .with_spatial(true)
                    .with_volume(Volume::Decibels(15.0)),
                Transform::from_translation(start_pos),
            ));
        }
    }
}

pub fn spawn_dash_trail(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    start_pos: Vec3,
    end_pos: Vec3,
) {
    // Create a cylinder between start and end positions
    let direction = end_pos - start_pos;
    let distance = direction.length();

    if distance > 0.1 {
        let center = (start_pos + end_pos) * 0.5;
        let normalized_dir = direction.normalize();

        // Create cylinder mesh
        let cylinder_mesh = meshes.add(Cylinder::new(0.1, distance));

        // Create glowing material
        let trail_material = materials.add(StandardMaterial {
            base_color: Color::srgba(0.3, 0.8, 1.0, 0.8),
            emissive: Color::srgb(0.3, 0.8, 1.0).into(),
            alpha_mode: AlphaMode::Blend,
            ..default()
        });

        // Calculate rotation to align cylinder with direction
        let rotation = if normalized_dir.y.abs() < 0.99 {
            Quat::from_rotation_arc(Vec3::Y, normalized_dir)
        } else {
            Quat::IDENTITY
        };

        commands.spawn((
            Mesh3d(cylinder_mesh),
            MeshMaterial3d(trail_material),
            Transform::from_translation(center).with_rotation(rotation),
            DashTrail {
                start_pos,
                end_pos,
                progress: 0.0,
                lifetime: Timer::from_seconds(0.3, TimerMode::Once),
            },
        ));
    }
}

pub fn animate_dash_trails(
    mut commands: Commands,
    mut trail_query: Query<(Entity, &mut Transform, &mut DashTrail)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut trail) in &mut trail_query {
        trail.lifetime.tick(time.delta());
        trail.progress = trail.lifetime.elapsed_secs() / trail.lifetime.duration().as_secs_f32();

        // Fade out the trail
        let alpha = 1.0 - trail.progress;

        if let Ok(material_handle) = material_query.get(entity) {
            if let Some(material) = materials.get_mut(&material_handle.0) {
                material.base_color.set_alpha(alpha);
                let emissive_intensity = alpha * 0.5;
                material.emissive = Color::srgb(
                    0.3 * emissive_intensity,
                    0.8 * emissive_intensity,
                    1.0 * emissive_intensity,
                )
                .into();
            }
        }

        // Scale down the trail over time
        let scale = 1.0 - (trail.progress * 0.5);
        transform.scale = Vec3::new(scale, 1.0, scale);

        // Remove when finished
        if trail.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

impl Ability for Dash {
    type CastParams = DashParams;

    fn add_systems(app: &mut bevy::app::App) {
        app.add_systems(Update, (cast_dash, animate_dash_trails));
    }
}
