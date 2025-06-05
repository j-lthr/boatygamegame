use std::marker::PhantomData;
use bevy::prelude::*;

use crate::ability::dash::Dash;
use crate::ability::dash::DashParams;
use crate::ability::shotgun::{Shotgun, ShotgunParams};
use crate::ability::AttemptCastEvent;
use crate::common::Inertia;
use crate::ui;

// Component to mark the player
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct DashTimer {
    pub timer: Timer,
}

#[derive(Component)]
pub struct PlayerCamera;


/// System to handle player movement with WASD keys (camera-relative)
pub fn handle_movement(
    mut player_query: Query<(Entity, &mut Transform), With<Player>>,
    camera_query: Query<&GlobalTransform, With<Camera3d>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut dash_action: EventWriter<AttemptCastEvent<Dash>>,
    time: Res<Time>,
) {
    if let (Ok((player, mut player_transform)), Ok(camera_transform)) = (player_query.single_mut(), camera_query.single()) {
        let mut velocity = Vec3::ZERO;
        let speed = 4.0;

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
            dash_action.write(AttemptCastEvent { caster: player, params: DashParams::Directional(velocity), _marker: PhantomData::default()});
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

/// System to handle shooting using ability system
pub fn shoot_gun(
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    player_query: Query<Entity, With<Player>>,
    camera_query: Query<(&GlobalTransform, &Camera), With<PlayerCamera>>,
    mut shotgun_action: EventWriter<AttemptCastEvent<Shotgun>>,
) {
    if let (Ok(player), Ok((camera_transform, camera))) =
        (player_query.single(), camera_query.single())
    {
        if mouse_input.pressed(MouseButton::Left) {
            if let Some(cursor_pos) = ui::compute_3d_cursor_pos(windows, camera, &camera_transform) {
                shotgun_action.write(AttemptCastEvent {
                    caster: player,
                    params: ShotgunParams {
                        target_position: cursor_pos,
                    },
                    _marker: PhantomData::default(),
                });
            }
        }
    }
}