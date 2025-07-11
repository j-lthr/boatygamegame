use crate::event;
use crate::event::DeathEvent;
use crate::player;
use crate::state;
use bevy::prelude::*;

#[derive(Component)]
pub struct Living {
    pub health: i32,
    pub max_health: i32,
}
impl Living {
    pub(crate) fn health_fraction(&self) -> f32 {
        self.health as f32 / self.max_health as f32
    }
}

#[derive(Component, PartialEq, Eq)]
pub enum Faction {
    Friendly,
    Enemy,
}

#[derive(Component)]
pub struct Inertia {
    pub prev_pos: Vec3,
    pub damping: f32,
}

#[derive(Component)]
pub struct VelocityEWA {
    pub velocity_ewa: Vec3,
    pub tau: f32,
}

pub fn handle_inertia(mut player_query: Query<(&mut Transform, &mut Inertia)>) {
    for (mut transform, mut inertia) in &mut player_query {
        let last_timestep_movement = transform.translation - inertia.prev_pos;
        inertia.prev_pos = transform.translation;
        transform.translation += last_timestep_movement * inertia.damping;
    }
}

pub fn handle_velocity_averaging(mut query: Query<(&Transform, &Inertia, &mut VelocityEWA)>, time: Res<Time>) {
    
    
    for (transform, inertia, mut velocity_ewa) in &mut query {
        let alpha = 1.0 / (1.0 + time.delta_secs() * velocity_ewa.tau);
        let velocity = (transform.translation - inertia.prev_pos) / time.delta_secs().max(0.001);
        velocity_ewa.velocity_ewa = alpha * velocity_ewa.velocity_ewa + (1.0 - alpha) * velocity;
    }
}

pub fn handle_damage_events(
    mut damage_events: EventReader<event::DamageEvent>,
    mut living_query: Query<(&mut Living, &Transform)>,
    mut score: ResMut<state::GameScore>,
    player_query: Query<&player::Player>,
) {
    for damage_event in damage_events.read() {
        if let Ok((mut living, _)) = living_query.get_mut(damage_event.target) {
            living.health -= damage_event.damage;
            info!(
                "Entity {:?} took {} damage! Health: {}",
                damage_event.target, damage_event.damage, living.health
            );

            // Check if entity died
            if living.health <= 0 {
                // Check if this was an enemy (not player) and killed by player for scoring
                if player_query.get(damage_event.target).is_err() {
                    if let Some(source) = damage_event.source {
                        if player_query.get(source).is_ok() {
                            // Player killed an enemy - update score!
                            score.kills += 1;
                            score.combo += 1;
                            score.combo_timer.reset(); // Reset combo timer

                            // Calculate points with combo multiplier
                            let base_points = 100;
                            let combo_bonus = (score.combo - 1) * 50; // 50 extra points per combo level
                            let points_earned = base_points + combo_bonus;
                            score.current += points_earned;
                        }
                    }
                }
            }
        }
    }
}

pub fn emit_death_events(
    enemy_query: Query<(Entity, &Living, &Transform)>,
    mut death_events: EventWriter<DeathEvent>,
) {
    for (entity, living, _) in &enemy_query {
        if living.health <= 0 {
            death_events.write(DeathEvent { entity });
        }
    }
}

pub fn handle_player_death(
    player_query: Query<(), With<player::Player>>,
    mut next_state: ResMut<NextState<state::GameState>>,
    mut death_events: EventReader<DeathEvent>,
) {
    for death_event in death_events.read() {
        if let Ok(()) = player_query.get(death_event.entity) {
            next_state.set(state::GameState::GameOver);
            break; // Only need to trigger game over once
        }
    }
}

pub fn handle_npc_death(
    mut commands: Commands,
    npc_query: Query<&Living, Without<player::Player>>,
    mut death_events: EventReader<DeathEvent>,
) {
    for death_event in death_events.read() {
        if npc_query.get(death_event.entity).is_ok() {
            commands.entity(death_event.entity).despawn();
        }
    }
}
