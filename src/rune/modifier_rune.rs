use bevy::prelude::*;

use crate::{
    common::HealthPool, modifiers::{Modifier, ModifierStack}, rune::{Rune, RuneApplicationEvent}
};

#[derive(Clone, Debug)]
pub struct ModifierRune {
    pub modifier: Modifier,
    pub color: Color,
}

impl Rune for ModifierRune {
    fn register_systems(app: &mut App) {
        app.add_systems(Update, handle_modifier_rune);
    }

    fn emissive(&self) -> LinearRgba {
        self.color.into()
    }
}

pub fn handle_modifier_rune(
    mut query: Query<&mut ModifierStack>,
    mut event_reader: EventReader<RuneApplicationEvent<ModifierRune>>,
) {
    for event in event_reader.read() {
        if let Ok(mut modifier_stack) = query.get_mut(event.entity) {
            modifier_stack.add_modifier(event.rune.modifier);
        }
    }
}

#[derive(Clone, Debug)]
pub struct HealRune {
    pub amount: i32,
    pub color: Color,
}

impl Rune for HealRune {
    fn register_systems(app: &mut App) {
        app.add_systems(Update, handle_heal_rune);
    }

    fn emissive(&self) -> LinearRgba {
        self.color.into()
    }
}

pub fn handle_heal_rune(
    mut query: Query<&mut HealthPool>,
    mut event_reader: EventReader<RuneApplicationEvent<HealRune>>,
) {
    for event in event_reader.read() {
        if let Ok(mut modifier_stack) = query.get_mut(event.entity) {
            modifier_stack.current_health += event.rune.amount;
        }
    }
}

