use std::f64::INFINITY;
use std::time::Instant;

use crate::event;
use crate::event::DeathEvent;
use crate::modifiers::*;
use crate::player;
use crate::state;
use bevy::prelude::*;

#[derive(Component)]
pub struct HealthPool {
    pub current_health: i32,
    pub max_health: i32,
    pub regen: i32,
}

#[derive(Component)]
pub struct Health {
    pub base_health: i32,
    pub base_regen: i32,
}

impl Health {
    pub fn new(base_health: i32, base_regen: i32) -> Self {
        Self {
            base_health,
            base_regen,
        }
    }
}

impl HealthPool {
    pub(crate) fn health_fraction(&self) -> f32 {
        self.current_health as f32 / self.max_health as f32
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

pub struct RegenTick {
    pub last_regen_tick: f64,
}

impl Default for RegenTick {
    fn default() -> Self {
        Self {
            last_regen_tick: 0.0,
        }
    }
}

pub fn apply_regen(query: Query<&mut HealthPool>, time: Res<Time>, mut tick: Local<RegenTick>) {
    let now = time.elapsed_secs_f64();

    if now - tick.last_regen_tick > 1.0 {
        for mut pool in query {
            pool.current_health = (pool.current_health + pool.regen).min(pool.max_health);
        }

        tick.last_regen_tick = now;
    }
}

pub fn update_health_pool(query: Query<(&mut HealthPool, &Health, Option<&ModifierStack>)>) {
    for (mut pool, health, modifiers) in query {
        let fraction = pool.health_fraction();

        pool.max_health =
            apply_modifier_if_present(modifiers, MAX_HEALTH_MODIFIER, health.base_health as f32)
                as i32;
        pool.regen =
            apply_modifier_if_present(modifiers, HEALTH_REGEN_MODIFIER, health.base_regen as f32)
                as i32;

        pool.current_health = (fraction * pool.max_health as f32) as i32;
    }
}

#[derive(Bundle)]
pub struct HealthBundle {
    health: Health,
    pool: HealthPool,
}

impl HealthBundle {
    pub fn new(base_health: i32, base_regen: i32) -> Self {
        Self {
            health: Health {
                base_health,
                base_regen,
            },
            pool: HealthPool {
                max_health: base_health,
                current_health: base_health,
                regen: base_regen,
            },
        }
    }
}

pub fn handle_inertia(mut player_query: Query<(&mut Transform, &mut Inertia)>) {
    for (mut transform, mut inertia) in &mut player_query {
        let last_timestep_movement = transform.translation - inertia.prev_pos;
        inertia.prev_pos = transform.translation;
        transform.translation += last_timestep_movement * inertia.damping;
    }
}

pub fn handle_velocity_averaging(
    mut query: Query<(&Transform, &Inertia, &mut VelocityEWA)>,
    time: Res<Time>,
) {
    for (transform, inertia, mut velocity_ewa) in &mut query {
        let alpha = 1.0 / (1.0 + time.delta_secs() * velocity_ewa.tau);
        let velocity = (transform.translation - inertia.prev_pos) / time.delta_secs().max(0.001);
        velocity_ewa.velocity_ewa = alpha * velocity_ewa.velocity_ewa + (1.0 - alpha) * velocity;
    }
}

pub fn handle_damage_events(
    mut damage_events: EventReader<event::DamageEvent>,
    mut living_query: Query<(&mut HealthPool, &Transform)>,
    mut score: ResMut<state::GameScore>,
    player_query: Query<&player::Player>,
) {
    for damage_event in damage_events.read() {
        if let Ok((mut living, _)) = living_query.get_mut(damage_event.target) {
            living.current_health -= damage_event.damage;
            info!(
                "Entity {:?} took {} damage! Health: {}",
                damage_event.target, damage_event.damage, living.current_health
            );

            // Check if entity died
            if living.current_health <= 0 {
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
    enemy_query: Query<(Entity, &HealthPool, &Transform)>,
    mut death_events: EventWriter<DeathEvent>,
) {
    for (entity, living, _) in &enemy_query {
        if living.current_health <= 0 {
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
    npc_query: Query<&HealthPool, Without<player::Player>>,
    mut death_events: EventReader<DeathEvent>,
) {
    for death_event in death_events.read() {
        if npc_query.get(death_event.entity).is_ok() {
            commands.entity(death_event.entity).despawn();
        }
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (update_health_pool, apply_regen));
}
