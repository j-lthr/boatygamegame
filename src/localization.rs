use bevy::prelude::*;
use fluent::{FluentBundle, FluentResource};
use fluent_templates::{static_loader, LanguageIdentifier, Loader};
use std::collections::HashMap;
use unic_langid::langid;
use crate::modifiers::{Modifier, ModifierType, ModifierID};

const US_ENGLISH: LanguageIdentifier = langid!("en-US");
const GERMAN: LanguageIdentifier = langid!("de-DE");
const SWISS_GERMAN: LanguageIdentifier = langid!("ch-CH");

static_loader! {
    static LOCALES = {
        locales: "./assets/locale",
        fallback_language: "en-US",
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

#[derive(Resource)]
pub struct LocalizationResource {
    pub current_language: LanguageIdentifier,
}

impl Default for LocalizationResource {
    fn default() -> Self {

        Self {
            current_language: US_ENGLISH,
        }
    }
}

impl LocalizationResource {
    pub fn get_text(&self, key: &str, args: Option<&HashMap<String, fluent::FluentValue>>) -> String {
        LOCALES.lookup_with_args(&self.current_language, key, args.unwrap_or(&HashMap::new()))
    }

    pub fn switch_language(&mut self, language: LanguageIdentifier) {
        self.current_language = language;
    }

    pub fn cycle_language(&mut self) {
        self.current_language = match self.current_language {
            ref lang if *lang == US_ENGLISH => GERMAN,
            ref lang if *lang == GERMAN => SWISS_GERMAN,
            _ => US_ENGLISH,
        };
    }

    pub fn format_stat(&self, stat: Modifier) -> String {
        let stat_key = format!("stat-{}", stat.id.as_str());
        
        let (type_str, value) = match stat.typ {
            ModifierType::Additive(v) => ("additive", format!("{:.0}%", v * 100.0)),
            ModifierType::Multiplicative(v) => ("multiplicative", format!("{:.0}%", (v - 1.0) * 100.0)),
        };


        let mut args = HashMap::<String, fluent::FluentValue>::new();
        args.insert("type".to_string(), type_str.into());
        args.insert("value".to_string(), value.into());
        args.insert("stat".to_string(), self.get_text(&stat_key, None).into());

        self.get_text("modifier-text", Some(&args))
    }
}

fn handle_language_toggle(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut localization: ResMut<LocalizationResource>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyL) {
        localization.cycle_language();
        let lang_name = match localization.current_language {
            ref lang if *lang == US_ENGLISH => "English (US)",
            ref lang if *lang == GERMAN => "Deutsch",
            ref lang if *lang == SWISS_GERMAN => "Schwiizerdütsch",
            _ => "Unknown",
        };
        info!("Language switched to: {} ({:?})", lang_name, localization.current_language);
    }
}

pub fn plugin(app: &mut App) {
    app.insert_resource(LocalizationResource::default());
    app.add_systems(Update, handle_language_toggle);
}