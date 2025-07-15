
use bevy::prelude::*;

use crate::{
    modifiers::{ModifierID, ModifierStack, Stat},
    rune::{Rune, RuneApplicationEvent},
};

#[derive(Clone, Debug)]
pub struct ModifierRune {
    pub stat: Stat,
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
            modifier_stack.add_stat(event.rune.stat);
        }
    }
}
