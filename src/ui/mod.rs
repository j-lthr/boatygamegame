use bevy::prelude::*;

pub mod common;
pub mod damage_numbers;
pub mod healthbar;
pub mod hud;
pub mod locale;
pub mod menu;
pub mod rune_popup;

pub(crate) use damage_numbers::DamageNumbersPlugin;
pub(crate) use healthbar::HealthBarPlugin;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        HealthBarPlugin,
        DamageNumbersPlugin,
        hud::plugin,
        rune_popup::plugin,
        common::plugin,
    ));
}
