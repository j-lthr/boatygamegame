use std::time::Duration;

use bevy::prelude::*;

use crate::{
    ability::{AbilitySlot, basic_projectile_attack::BasicProjectileAttack},
    common::Living,
    player::Player,
    rune::{Rune, RuneApplicationEvent},
};

#[derive(Clone, Debug)]
pub struct SpeedRune {
    pub increase: f32,
}

impl Rune for SpeedRune {
    fn register_systems(app: &mut App) {
        app.add_systems(Update, handle_speed_rune);
    }

    fn emissive(&self) -> LinearRgba {
        LinearRgba::new(0.5, 1.0, 100.0, 1.0)
    }
}

pub fn handle_speed_rune(
    mut player_query: Query<&mut Player>,
    mut event_reader: EventReader<RuneApplicationEvent<SpeedRune>>,
) {
    for event in event_reader.read() {
        if let Ok(mut player) = player_query.get_mut(event.entity) {
            player.speed += event.rune.increase;

            player.speed = player.speed.min(100.0);
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

    fn emissive(&self) -> LinearRgba {
        LinearRgba::new(100.0, 100.0, 100.0, 1.0)
    }
}

pub fn handle_heal_rune(
    mut player_query: Query<&mut Living>,
    mut event_reader: EventReader<RuneApplicationEvent<HealRune>>,
) {
    for event in event_reader.read() {
        if let Ok(mut living) = player_query.get_mut(event.entity) {
            living.health = (living.health + event.rune.restore_amount).min(living.max_health);
        }
    }
}

#[derive(Clone, Debug)]
pub struct BasicProjectileAttackRune {
    pub added_bullets: i32,
    pub added_damage: i32,
    pub cooldown_recovery_rate_factor: f32,
    pub projectile_speed: f32,
    pub pierce: i32,
    pub color: Color,
}

impl BasicProjectileAttackRune {
    pub const fn empty() -> Self {
        Self {
            added_bullets: 0,
            added_damage: 0,
            cooldown_recovery_rate_factor: 1.0,
            projectile_speed: 0.0,
            pierce: 0,
            color: Color::BLACK,
        }
    }
}

impl Rune for BasicProjectileAttackRune {
    fn register_systems(app: &mut App) {
        app.add_systems(Update, handle_multishot_rune);
    }

    fn emissive(&self) -> LinearRgba {
        self.color.into()
    }
}

pub fn handle_multishot_rune(
    mut player_query: Query<&mut AbilitySlot<BasicProjectileAttack>>,
    mut event_reader: EventReader<RuneApplicationEvent<BasicProjectileAttackRune>>,
) {
    for event in event_reader.read() {
        if let Ok(mut projectile_attack) = player_query.get_mut(event.entity) {
            projectile_attack.ability.bullet_count += event.rune.added_bullets;
            projectile_attack.ability.spread += event.rune.added_bullets as f32 * 0.0005;
            projectile_attack.ability.damage += event.rune.added_damage;
            projectile_attack.ability.speed += event.rune.projectile_speed;
            projectile_attack.ability.pierce += event.rune.pierce;
            let current_duration = projectile_attack.cooldown.duration().as_secs_f32();
            projectile_attack.cooldown.set_duration(Duration::from_secs_f32(current_duration * event.rune.cooldown_recovery_rate_factor));
        }
    }
}

pub const SPEED_RUNE: SpeedRune = SpeedRune { increase: 1.0 };

pub const HEAL_RUNE: HealRune = HealRune { restore_amount: 10 };

pub const MULTISHOT_RUNE: BasicProjectileAttackRune = BasicProjectileAttackRune { added_bullets: 2 , color: Color::linear_rgb(100.0, 0.0, 0.0), ..BasicProjectileAttackRune::empty()};
pub const DAMAGE_RUNE: BasicProjectileAttackRune = BasicProjectileAttackRune { added_damage: 2 , color: Color::linear_rgb(100.0, 50.0, 0.0), ..BasicProjectileAttackRune::empty()};
pub const ATTACK_SPEED_RUNE: BasicProjectileAttackRune = BasicProjectileAttackRune { cooldown_recovery_rate_factor: 0.9 , color: Color::linear_rgb(0.0, 100.0, 100.0), ..BasicProjectileAttackRune::empty()};
pub const PROJECTILE_SPEED_RUNE: BasicProjectileAttackRune = BasicProjectileAttackRune { projectile_speed: 1.0, color: Color::linear_rgb(100.0, 0.0, 100.0), ..BasicProjectileAttackRune::empty()};
pub const PIERCE_RUNE: BasicProjectileAttackRune = BasicProjectileAttackRune { pierce: 1, color: Color::linear_rgb(10.0, 0.0, 15.0), ..BasicProjectileAttackRune::empty()};