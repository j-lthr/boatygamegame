use bevy::prelude::*;

use crate::{ability::{basic_projectile_attack::BasicProjectileAttack, AbilitySlot}, common::Living, player::Player, rune::{Rune, RuneApplicationEvent}};

#[derive(Clone, Debug)]
pub struct SpeedRune {
    pub increase: f32,
}

impl Rune for SpeedRune {
    fn register_systems(app: &mut App) {
        app.add_systems(Update, handle_speed_rune);
    }

    fn emissive() -> LinearRgba {
        LinearRgba::new(0.5,1.0,100.0,1.0)
    }
}

pub fn handle_speed_rune(mut player_query: Query<&mut Player>, mut event_reader: EventReader<RuneApplicationEvent<SpeedRune>>) {
    for event in event_reader.read() {
        if let Ok(mut player) = player_query.get_mut(event.entity) {
            player.speed += event.rune.increase;
        }
    }
}

#[derive(Clone, Debug)]
pub struct HealRune {
    pub restore_amount: i32,
}

impl Rune for HealRune {
    fn register_systems(app: &mut App) {
        app.add_systems(Update, handle_heal_rune);
    }

    fn emissive() -> LinearRgba {
        LinearRgba::new(0.5,100.0,1.0,1.0)
    }
}

pub fn handle_heal_rune(mut player_query: Query<&mut Living>, mut event_reader: EventReader<RuneApplicationEvent<HealRune>>) {
    for event in event_reader.read() {
        if let Ok(mut living) = player_query.get_mut(event.entity) {
            living.health = (living.health + event.rune.restore_amount).min(living.max_health);
        }
    }
}

#[derive(Clone, Debug)]
pub struct MultishotRune {
    pub added_bullets: i32,
}

impl Rune for MultishotRune {
    fn register_systems(app: &mut App) {
        app.add_systems(Update, handle_multishot_rune);
    }

    fn emissive() -> LinearRgba {
        LinearRgba::new(100.0,1.0,1.0,1.0)
    }
}

pub fn handle_multishot_rune(mut player_query: Query<&mut AbilitySlot<BasicProjectileAttack>>, mut event_reader: EventReader<RuneApplicationEvent<MultishotRune>>) {
    for event in event_reader.read() {
        if let Ok(mut projectile_attack) = player_query.get_mut(event.entity) {
            projectile_attack.ability.bullet_count += event.rune.added_bullets;    
            projectile_attack.ability.spread += event.rune.added_bullets as f32 * 0.0005;
        }
    }
}



pub const SPEED_RUNE: SpeedRune = SpeedRune {
    increase: 1.0,
};

pub const HEAL_RUNE: HealRune = HealRune {
    restore_amount: 10,
};

pub const MULTISHOT_RUNE: MultishotRune = MultishotRune {
    added_bullets: 2,
};

