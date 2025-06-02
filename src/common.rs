use bevy::prelude::*;

#[derive(Component)]
pub struct Living {
    pub health: i32,
    pub max_health: i32,
}


#[derive(Component)]
pub struct Inertia {
    pub prev_pos: Vec3,
    pub damping: f32,
}

pub fn handle_inertia(mut player_query: Query<(&mut Transform, &mut Inertia)>) {
    for (mut transform, mut inertia) in &mut player_query {
        let last_timestep_movement = transform.translation - inertia.prev_pos;
        inertia.prev_pos = transform.translation;
        transform.translation += last_timestep_movement * inertia.damping;
    }
}
