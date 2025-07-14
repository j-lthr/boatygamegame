
use bevy::prelude::*;

use crate::{
    modifiers::{ModifierID, ModifierStack},
    rune::{Rune, RuneApplicationEvent},
};

#[derive(Clone, Debug)]
pub struct ModifierRune {
    pub modifier_id: ModifierID,
    pub multiplicative_factor: f32,
    pub additive_factor: f32,
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
            modifier_stack.add_multiplicative_modifier(
                event.rune.modifier_id,
                event.rune.multiplicative_factor,
            );
            modifier_stack
                .add_additive_modifier(event.rune.modifier_id, event.rune.additive_factor);
        }
    }
}
