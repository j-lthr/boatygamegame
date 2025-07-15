use bevy::prelude::*;
use fluent::{FluentBundle, FluentResource};
use fluent_templates::{static_loader, LanguageIdentifier, Loader};
use std::collections::HashMap;
use unic_langid::langid;
use crate::modifiers::{Stat, StatKind, ModifierID};

const US_ENGLISH: LanguageIdentifier = langid!("en-US");

static_loader! {
    static LOCALES = {
        locales: "./assets/locale",
        fallback_language: "en-US",
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

    pub fn format_stat(&self, stat: Stat) -> String {
        let stat_key = format!("stat-{}", stat.id.as_str());
        
        let (type_str, value) = match stat.kind {
            StatKind::Additive(v) => ("additive", format!("{:.0}%", v * 100.0)),
            StatKind::Multiplicative(v) => ("multiplicative", format!("{:.0}%", (v - 1.0) * 100.0)),
        };


        let mut args = HashMap::<String, fluent::FluentValue>::new();
        args.insert("type".to_string(), type_str.into());
        args.insert("value".to_string(), value.into());
        args.insert("stat".to_string(), self.get_text(&stat_key, None).into());

        self.get_text("modifier-text", Some(&args))
    }
}

pub fn plugin(app: &mut App) {
    app.insert_resource(LocalizationResource::default());
}