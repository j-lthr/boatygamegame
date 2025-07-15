use bevy::prelude::*;

#[derive(Component)]
pub struct MoveAnimation {
    pub velocity: Vec2,
    pub start_position: Vec2,
}

impl MoveAnimation {
    pub fn new(velocity: Vec2, start_position: Vec2) -> Self {
        Self {
            velocity,
            start_position,
        }
    }
    
    pub fn downward(speed: f32, start_position: Vec2) -> Self {
        Self::new(Vec2::new(0.0, -speed), start_position)
    }
}

#[derive(Component)]
pub struct FadeAnimation {
    pub start_alpha: f32,
    pub end_alpha: f32,
    pub duration: f32,
    pub elapsed: f32,
}

impl FadeAnimation {
    pub fn new(start_alpha: f32, end_alpha: f32, duration: f32) -> Self {
        Self {
            start_alpha,
            end_alpha,
            duration,
            elapsed: 0.0,
        }
    }
    
    pub fn fade_out(duration: f32) -> Self {
        Self::new(1.0, 0.0, duration)
    }
    
    pub fn current_alpha(&self) -> f32 {
        let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
        self.start_alpha + (self.end_alpha - self.start_alpha) * t
    }
    
    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration
    }
}

pub fn update_move_animations(
    time: Res<Time>,
    mut query: Query<(&mut Node, &MoveAnimation)>,
) {
    for (mut node, move_anim) in query.iter_mut() {
        let delta_movement = move_anim.velocity * time.delta_secs();
        
        // Read current values
        let current_top = match node.top {
            Val::Px(val) => val,
            _ => move_anim.start_position.y,
        };
        let current_left = match node.left {
            Val::Px(val) => val,
            _ => move_anim.start_position.x,
        };
        
        // Modify and write back
        node.top = Val::Px(current_top + delta_movement.y);
        node.left = Val::Px(current_left + delta_movement.x);
    }
}

pub fn update_fade_animations(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut BackgroundColor, &mut FadeAnimation)>,
) {
    for (entity, mut bg_color, mut fade_anim) in query.iter_mut() {
        fade_anim.elapsed += time.delta_secs();
        
        let alpha = fade_anim.current_alpha();
        bg_color.0 = bg_color.0.with_alpha(alpha);
        
        if fade_anim.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn update_text_fade_animations(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut TextColor, &mut FadeAnimation)>,
) {
    for (entity, mut text_color, mut fade_anim) in query.iter_mut() {
        fade_anim.elapsed += time.delta_secs();
        
        let alpha = fade_anim.current_alpha();
        text_color.0 = text_color.0.with_alpha(alpha);
        
        if fade_anim.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (
        update_move_animations,
        update_fade_animations,
        update_text_fade_animations,
    ));
}