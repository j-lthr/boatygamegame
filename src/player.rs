use std::time::Duration;

use bevy::audio::Volume;
use bevy::prelude::*;

use crate::bullet;
use crate::common::Inertia;
use crate::fx;
use crate::ui;

// Component to mark the player
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct DashTimer {
    pub timer: Timer,
}


/// System to handle player movement with WASD keys (camera-relative)
pub fn handle_movement(
    mut player_query: Query<(&mut Transform, &mut DashTimer), With<Player>>,
    camera_query: Query<&GlobalTransform, With<Camera3d>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if let (Ok((mut player_transform, mut dash_timer)), Ok(camera_transform)) = (player_query.single_mut(), camera_query.single()) {
        let mut velocity = Vec3::ZERO;
        let speed = 4.0;

        dash_timer.timer.tick(time.delta());

        // Get camera's forward and right vectors, but keep them horizontal for ground movement
        let forward = camera_transform.forward();
        let right = -camera_transform.right();

        // Project forward and right vectors onto the horizontal plane (y=0)
        let forward_horizontal = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
        let right_horizontal = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

        // WASD movement relative to camera direction
        if keyboard_input.pressed(KeyCode::KeyW) {
            velocity += forward_horizontal;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            velocity -= forward_horizontal;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            velocity += right_horizontal;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            velocity -= right_horizontal;
        }

        if keyboard_input.pressed(KeyCode::Space) {
            if dash_timer.timer.finished() {
                // Dash forward in the direction the player is facing
                let dash_distance = 2.5; // Distance to dash

                player_transform.translation += velocity * dash_distance;
                dash_timer.timer.reset(); // Reset dash timer
            }
        }

        // Normalize diagonal movement and apply
        if velocity.length() > 0.0 {
            velocity = velocity.normalize();
            player_transform.translation += velocity * speed * time.delta_secs();
        }
    }
}

/// System to make camera follow player with smooth interpolation
pub fn handle_camera(
    player_query: Query<(&Transform, &Inertia), (With<Player>, Without<Camera3d>)>,
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<Player>)>,
    time: Res<Time>,
) {
    if let (Ok((player_transform, player_inertia)), Ok(mut camera_transform)) =
        (player_query.single(), camera_query.single_mut())

        {

        let player_velocity_direction = (player_transform.translation - player_inertia.prev_pos).normalize_or_zero();
        
        // Define the offset from player to camera (above and behind)
        let camera_offset = (-10.0 * player_velocity_direction).with_y(10.0);

        // Calculate desired camera position
        let target_position = player_transform.translation + camera_offset;

        // Lerp factor - higher values = faster following, lower = smoother
        let lerp_factor = 1.0 * time.delta_secs();

        // Smoothly move camera towards target position
        camera_transform.translation = camera_transform
            .translation
            .lerp(target_position, lerp_factor);

        // Make camera look at the player
        let look_target = player_transform.translation + Vec3::new(0.0, 0.5, 0.0); // Look slightly above player center
        camera_transform.look_at(look_target, Vec3::Y);
    }
}




#[derive(Component)]
pub(crate) struct WeaponCooldown {
    pub timer: Timer,
}

/// System to handle shooting
pub fn shoot_gun(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut shoot_sounds: ResMut<Assets<fx::fm::FMSound>>,
    time: Res<Time>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut player_query: Query<(&mut Transform, &mut WeaponCooldown), With<Player>>,
    mut camera_query: Query<(&GlobalTransform,  &Camera)>,
) {
    if let (Ok((camera_transform, camera)), Ok((mut player_transform, mut gun))) =
        (camera_query.single_mut(), player_query.single_mut())
    {
        gun.timer.tick(time.delta());

        // Check if the gun's bullet timer allows shooting
        if mouse_input.pressed(MouseButton::Left) && gun.timer.finished() {
            gun.timer.reset(); // Reset timer for next shot

            // Play shooting sound (FM synthesis)
            let shoot_sound_handle = shoot_sounds.add(fx::fm::FMSound {
                config: fx::fm::GUN_SOUND,             // How much the frequency varies
                duration: Duration::from_millis(1000), // Short punchy sound
            });

            let bullet_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.2, 0.2, 0.2),
                metallic: 0.5,
                perceptual_roughness: 0.5,
                emissive: Color::srgb(5.0, 5.0, 5.0).into(), // Slightly glowing
                ..default()
            });

            if let Some(cursor_pos) = ui::compute_3d_cursor_pos(windows, camera, &camera_transform) {
                for _ in 0..5 {
                    // Calculate bullet direction based on camera forward vector
                    let direction = (cursor_pos - player_transform.translation)
                        .normalize()
                        .with_y(0.0)
                        + Vec3::new(fastrand::f32() - 0.5, 0.0, fastrand::f32() - 0.5) * 0.5; // Add slight random jitter

                    // Spawn bullet at camera position
                    commands.spawn((
                        Mesh3d(meshes.add(Sphere::new(0.1))),
                        MeshMaterial3d(bullet_mat.clone()), // Red color
                        Transform::from_translation(player_transform.translation),
                        bullet::Bullet {
                            direction,
                            speed: 30.0,
                            lifetime: 1.0, // 5 seconds before cleanup
                        },
                        AudioPlayer(shoot_sound_handle.clone()),
                        PlaybackSettings::ONCE
                            .with_spatial(true)
                            .with_volume(Volume::Decibels(24.0)), // Play sound once with spatial audio
                    ));

                    player_transform.translation -= direction * 0.1; // Move player back slightly with each shot
                }
            }
        }
    }
}