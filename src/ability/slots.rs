use bevy::prelude::*;
use std::collections::HashMap;
use std::time::Duration;

use crate::ability::{CastDynamicAbility, DynamicAbility, IntendedTarget};
use crate::common::Targetable;
use crate::input::Cursor;
use crate::player::Player;

/// Targeting behavior for abilities
#[derive(Clone, Debug)]
pub enum AbilityTargeting {
    Cursor, // Target cursor position
    NearestEnemyToCursor {
        // Target nearest enemy to cursor
        max_range: f32,
    },
    SelfCast, // Target self (IntendedTarget::None)
}

/// Wrapper for DynamicAbility with targeting, cooldowns, etc.
#[derive(Clone, Debug)]
pub struct SlottedAbility {
    pub ability: DynamicAbility,
    pub targeting: AbilityTargeting,
    pub base_cooldown: f32,    // Base cooldown in seconds
    pub cooldown_timer: Timer, // Current cooldown state
}

impl SlottedAbility {
    pub fn new(ability: DynamicAbility, targeting: AbilityTargeting, cooldown: f32) -> Self {
        Self {
            ability,
            targeting,
            base_cooldown: cooldown,
            cooldown_timer: Timer::from_seconds(0.0, TimerMode::Once), // Start ready
        }
    }

    pub fn is_ready(&self) -> bool {
        self.cooldown_timer.finished()
    }

    pub fn trigger_cooldown(&mut self) {
        self.cooldown_timer = Timer::from_seconds(self.base_cooldown, TimerMode::Once);
    }

    pub fn update_cooldown(&mut self, delta: Duration) {
        self.cooldown_timer.tick(delta);
    }

    pub fn remaining_cooldown(&self) -> f32 {
        if self.is_ready() {
            0.0
        } else {
            self.cooldown_timer.duration().as_secs_f32() - self.cooldown_timer.elapsed_secs()
        }
    }
}

/// Component for entities that can use abilities
#[derive(Component, Debug)]
pub struct AbilitySlots {
    slots: Vec<Option<SlottedAbility>>,
}

/// Slot identifier for targeting specific slots
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct SlotId(pub usize);

impl AbilitySlots {
    pub fn new(slot_count: usize) -> Self {
        Self {
            slots: vec![None; slot_count],
        }
    }

    pub fn with_abilities(abilities: Vec<SlottedAbility>) -> Self {
        Self {
            slots: abilities.into_iter().map(Some).collect(),
        }
    }

    pub fn set_ability(&mut self, slot_id: SlotId, ability: SlottedAbility) -> Result<(), String> {
        if slot_id.0 >= self.slots.len() {
            return Err(format!("Slot {} out of range", slot_id.0));
        }
        self.slots[slot_id.0] = Some(ability);
        Ok(())
    }

    pub fn get_ability(&self, slot_id: SlotId) -> Option<&SlottedAbility> {
        self.slots.get(slot_id.0)?.as_ref()
    }

    pub fn get_ability_mut(&mut self, slot_id: SlotId) -> Option<&mut SlottedAbility> {
        self.slots.get_mut(slot_id.0)?.as_mut()
    }

    pub fn clear_slot(&mut self, slot_id: SlotId) {
        if slot_id.0 < self.slots.len() {
            self.slots[slot_id.0] = None;
        }
    }

    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    pub fn available_slots(&self) -> Vec<SlotId> {
        self.slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.is_none())
            .map(|(i, _)| SlotId(i))
            .collect()
    }

    pub fn occupied_slots(&self) -> Vec<SlotId> {
        self.slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.is_some())
            .map(|(i, _)| SlotId(i))
            .collect()
    }
}

/// Configurable keymap for ability slots
#[derive(Component, Clone, Default)]
pub struct AbilityKeymap {
    bindings: HashMap<KeyCode, SlotId>,
    mouse_bindings: HashMap<MouseButton, SlotId>,
}

impl AbilityKeymap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind_key(&mut self, key: KeyCode, slot: SlotId) -> &mut Self {
        self.bindings.insert(key, slot);
        self
    }

    pub fn bind_mouse(&mut self, button: MouseButton, slot: SlotId) -> &mut Self {
        self.mouse_bindings.insert(button, slot);
        self
    }

    pub fn unbind_key(&mut self, key: &KeyCode) -> &mut Self {
        self.bindings.remove(key);
        self
    }

    pub fn unbind_mouse(&mut self, button: &MouseButton) -> &mut Self {
        self.mouse_bindings.remove(button);
        self
    }

    pub fn get_slot_for_key(&self, key: &KeyCode) -> Option<SlotId> {
        self.bindings.get(key).copied()
    }

    pub fn get_slot_for_mouse(&self, button: &MouseButton) -> Option<SlotId> {
        self.mouse_bindings.get(button).copied()
    }

    pub fn get_keys_for_slot(&self, slot_id: SlotId) -> Vec<KeyCode> {
        self.bindings
            .iter()
            .filter(|&(_, slot)| *slot == slot_id)
            .map(|(&key, _)| key)
            .collect()
    }

    pub fn get_mouse_buttons_for_slot(&self, slot_id: SlotId) -> Vec<MouseButton> {
        self.mouse_bindings
            .iter()
            .filter(|&(_, slot)| *slot == slot_id)
            .map(|(&button, _)| button)
            .collect()
    }
}

/// Event for requesting ability cast from a slot
#[derive(Event, Debug)]
pub struct TriggerAbilitySlot {
    pub slot_id: SlotId,
}

impl TriggerAbilitySlot {
    pub fn from_id(id: SlotId) -> Self {
        Self { slot_id: id }
    }
}

/// System to update ability cooldowns
pub fn update_ability_cooldowns(mut ability_slots: Query<&mut AbilitySlots>, time: Res<Time>) {
    for mut slots in ability_slots.iter_mut() {
        for ability in slots.slots.iter_mut().flatten() {
            ability.update_cooldown(time.delta());
        }
    }
}

/// System to handle keyboard and mouse input for abilities
pub fn handle_ability_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    player_query: Query<(Entity, &AbilityKeymap), With<Player>>,
) {
    let Ok((player_entity, keymap)) = player_query.single() else {
        return;
    };

    // Handle keyboard inputs
    for key in keyboard.get_pressed() {
        if let Some(slot_id) = keymap.get_slot_for_key(key) {
            commands
                .entity(player_entity)
                .trigger(TriggerAbilitySlot::from_id(slot_id));
        }
    }

    // Handle mouse inputs
    for button in mouse.get_pressed() {
        if let Some(slot_id) = keymap.get_slot_for_mouse(button) {
            commands
                .entity(player_entity)
                .trigger(TriggerAbilitySlot::from_id(slot_id));
        }
    }
}

/// Helper function to resolve nearest enemy to cursor
fn resolve_nearest_enemy_to_cursor(
    cursor_query: &Query<&Transform, With<Cursor>>,
    enemy_query: &Query<(Entity, &Transform), (With<Targetable>, Without<Player>)>,
    max_range: f32,
) -> IntendedTarget {
    if let Ok(cursor_transform) = cursor_query.single() {
        let cursor_pos = cursor_transform.translation;

        let nearest_enemy = enemy_query
            .iter()
            .filter_map(|(entity, transform)| {
                let distance = cursor_pos.distance(transform.translation);
                if distance <= max_range {
                    Some((entity, distance))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((entity, _)) = nearest_enemy {
            IntendedTarget::Entity(entity)
        } else {
            IntendedTarget::Position(cursor_pos) // Fallback to cursor
        }
    } else {
        IntendedTarget::None // Fallback to self-cast
    }
}

/// Observer system to handle ability slot triggers
pub fn handle_ability_slot_trigger(
    trigger: Trigger<TriggerAbilitySlot>,
    mut ability_slots: Query<&mut AbilitySlots>,
    cursor_query: Query<&Transform, With<Cursor>>,
    enemy_query: Query<(Entity, &Transform), (With<Targetable>, Without<Player>)>,
    mut commands: Commands,
) {
    let event = trigger.event();

    let Ok(mut slots) = ability_slots.get_mut(trigger.target()) else {
        return;
    };
    let Some(slotted_ability) = slots.get_ability_mut(event.slot_id) else {
        warn!(
            "Entity {} doesn't have an ability in slot {}",
            trigger.target(),
            event.slot_id.0
        );
        return;
    };

    // Check if ability is ready (not on cooldown)
    if !slotted_ability.is_ready() {
        // Could emit a "ability on cooldown" event here for UI feedback
        return;
    }

    let mut event = CastDynamicAbility::at_caster(
                        slotted_ability.ability.clone(),
                        trigger.target(),
                    );

    // Resolve target and cast based on ability's targeting behavior

    commands.entity(trigger.target()).trigger(
    match &slotted_ability.targeting {
        AbilityTargeting::Cursor => {
            if let Ok(cursor_transform) = cursor_query.single() {
                let cursor_pos = cursor_transform.translation;
                event.with_target_position(cursor_pos)
            } else {
                warn!("No cursor found, falling back to self-cast");
                event
            }

            
        }
        AbilityTargeting::NearestEnemyToCursor { max_range } => {
            let target = resolve_nearest_enemy_to_cursor(&cursor_query, &enemy_query, *max_range);
            match target {
                IntendedTarget::Entity(entity) => {
                    event.with_target_entity(entity)
                }
                IntendedTarget::Position(pos) => {
                    event.with_target_position(pos)
                }
                IntendedTarget::None => {
                   event
                }
            }
        }
        AbilityTargeting::SelfCast => {
            event
        }
    });

    // Trigger cooldown
    slotted_ability.trigger_cooldown();
}

/// Plugin to register all ability slot systems
pub fn plugin(app: &mut App) {
    app.add_event::<TriggerAbilitySlot>();

    app.add_systems(Update, (handle_ability_input, update_ability_cooldowns));

    app.add_observer(handle_ability_slot_trigger);
}
