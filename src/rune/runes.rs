use bevy::prelude::*;

use crate::{
    modifiers::*,
    rune::{ModifierRune},
};



pub const SPEED_RUNE: ModifierRune = ModifierRune {
    stat: Stat::additive(PLAYER_SPEED_MODIFIER, 0.1),
    color: Color::linear_rgb(0.5, 1.0, 100.0),
};

pub const MULTISHOT_RUNE: ModifierRune = ModifierRune {
    stat: Stat::additive(PROJECTILE_COUNT_MODIFIER, 1.0),
    color: Color::linear_rgb(100.0, 0.0, 0.0),
};

pub const DAMAGE_RUNE: ModifierRune = ModifierRune {
    stat: Stat::additive(DAMAGE_MODIFIER, 0.1),
    color: Color::linear_rgb(100.0, 50.0, 0.0),
};

pub const AOE_RUNE: ModifierRune = ModifierRune {
    stat: Stat::additive(AOE_RADIUS_MODIFIER, 0.1),
    color: Color::linear_rgb(0.0, 50.0, 50.0),
};
