use bevy::prelude::*;
use crate::player;
use crate::enemy;
use crate::event;
use crate::state;
use crate::fx;

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

pub fn check_player_death(
    player_query: Query<&Living, With<player::Player>>,
    mut next_state: ResMut<NextState<state::GameState>>,
) {
    for living in &player_query {
        if living.health <= 0 {
            info!("Game Over! Player health: {}", living.health);
            next_state.set(state::GameState::GameOver);
            break; // Only need to trigger game over once
        }
    }
}

pub fn handle_damage_events(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut damage_events: EventReader<event::DamageEvent>,
    mut living_query: Query<(&mut Living, &Transform)>,
    mut score: ResMut<state::GameScore>,
    player_query: Query<&player::Player>,
    mut shoot_sounds: ResMut<Assets<fx::fm::FMSound>>,
    blood_materials: Res<fx::blood::BloodMaterials>,
) {
    use std::time::Duration;
    use bevy::audio::Volume;
    
    for damage_event in damage_events.read() {
        if let Ok((mut living, target_transform)) = living_query.get_mut(damage_event.target) {
            living.health -= damage_event.damage;
            info!("Entity {:?} took {} damage! Health: {}", 
                damage_event.target, damage_event.damage, living.health);
                
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
                
                // Spawn death effects
                fx::blood::spawn_blood_explosion(
                    &mut commands,
                    &mut meshes,
                    target_transform.translation,
                    &blood_materials,
                    100,
                    5.0,
                    Vec3::ZERO,
                    Vec3::new(10.0, 5.0, 0.0),
                );

                let shoot_sound_handle = shoot_sounds.add(fx::fm::FMSound {
                    config: fx::fm::DEATH_SOUND,
                    duration: Duration::from_millis(100),
                });

                commands.spawn((
                    AudioPlayer(shoot_sound_handle),
                    PlaybackSettings::DESPAWN
                        .with_spatial(true)
                        .with_volume(Volume::Decibels(36.0)),
                    Transform::from_translation(target_transform.translation),
                ));
            }
        }
    }
}

pub fn handle_enemy_deaths(
    mut commands: Commands,
    enemy_query: Query<(Entity, &Living), With<enemy::Enemy>>,
) {
    for (entity, living) in &enemy_query {
        if living.health <= 0 {
            info!("Enemy died! Despawning entity {:?}", entity);
            commands.entity(entity).despawn();
        }
    }
}
