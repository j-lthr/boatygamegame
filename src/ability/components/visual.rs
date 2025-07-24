use bevy::prelude::*;
use super::common::Lifetime;

#[derive(Component, Clone)]
pub struct LifetimeFadeout {
    pub fade_duration: f32,
}

impl LifetimeFadeout {
    pub fn new(fade_duration: f32) -> Self {
        Self { fade_duration }
    }
}

pub fn lifetime_fadeout_system(
    mut query: Query<(&mut Transform, &mut Visibility, &Lifetime, &LifetimeFadeout)>,
) {
    for (mut transform, mut visibility, lifetime, fadeout) in query.iter_mut() {
        let elapsed_ratio = lifetime.elapsed_ratio();
        let fade_start_ratio = 1.0 - fadeout.fade_duration;
        
        if elapsed_ratio >= fade_start_ratio {
            // Calculate fade progress (0.0 = fade start, 1.0 = fully faded)
            let fade_progress = if fadeout.fade_duration > 0.0 {
                (elapsed_ratio - fade_start_ratio) / fadeout.fade_duration
            } else {
                1.0
            };
            
            // Apply fade to scale (from 1.0 to 0.0)
            let fade_scale = 1.0 - fade_progress;
            transform.scale = Vec3::splat(fade_scale);
            
            // Apply fade to alpha via visibility
            if fade_progress >= 1.0 {
                *visibility = Visibility::Hidden;
            } else {
                *visibility = Visibility::Visible;
            }
        }
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update, lifetime_fadeout_system);
}