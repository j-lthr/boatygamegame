use bevy::prelude::*;

use crate::{
    modifiers::*,
    rune::{ModifierRune},
};

pub const SPEED_RUNE: ModifierRune = ModifierRune {
    modifier: Modifier::additive(PLAYER_SPEED_MODIFIER, 0.1),
    color: Color::linear_rgb(15.0, 25.0, 50.0),
};

pub const MULTISHOT_RUNE: ModifierRune = ModifierRune {
    modifier: Modifier::flat_added(PROJECTILE_COUNT_MODIFIER, 1.0),
    color: Color::linear_rgb(50.0, 10.0, 5.0),
};

pub const DAMAGE_RUNE: ModifierRune = ModifierRune {
    modifier: Modifier::additive(DAMAGE_MODIFIER, 0.3),
    color: Color::linear_rgb(40.0, 20.0, 5.0),
};

pub const AOE_RUNE: ModifierRune = ModifierRune {
    modifier: Modifier::additive(AOE_RADIUS_MODIFIER, 0.5),
    color: Color::linear_rgb(10.0, 30.0, 35.0),
};

pub const MAX_HEALTH_RUNE: ModifierRune = ModifierRune {
    modifier: Modifier::additive(MAX_HEALTH_MODIFIER, 0.1),
    color: Color::linear_rgb(5.0, 40.0, 10.0),
};

pub const HEALTH_REGEN_RUNE: ModifierRune = ModifierRune {
    modifier: Modifier::flat_added(HEALTH_REGEN_MODIFIER, 1.0),
    color: Color::linear_rgb(30.0, 50.0, 30.0),
};

pub const COOLDOWN_RECOVERY_RATE_RUNE: ModifierRune = ModifierRune {
    modifier: Modifier::additive(COOLDOWN_RECOVERY_RATE_MODIFIER, 0.1),
    color: Color::linear_rgb(45.0, 15.0, 45.0),
};

pub const PROJECTILE_SPEED_RUNE: ModifierRune = ModifierRune {
    modifier: Modifier::additive(PROJECTILE_SPEED_MODIFIER, 0.2),
    color: Color::linear_rgb(10.0, 15.0, 45.0),
};

pub const HOMING_RUNE: ModifierRune = ModifierRune {
    modifier: Modifier::flat_added(HOMING_STRENGTH_MODIFIER, 1.0),
    color: Color::linear_rgb(25.0, 40.0, 20.0),
};