use crate::localization::LocalizationResource;
use crate::state::GameScore;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct ComboText;

// Setup the score UI
pub fn setup_score_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    localization: Res<LocalizationResource>,
) {
    // Score display
    let mut score_args = HashMap::new();
    score_args.insert("score".to_string(), 0.into());
    commands.spawn((
        Text::new(localization.get_text("hud-score", Some(&score_args))),
        TextFont {
            font: asset_server.load(super::FONT_PATH),
            font_size: 32.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(20.0),
            ..default()
        },
        ScoreText,
    ));

    // Combo display (initially hidden)
    commands.spawn((
        Text::new(""),
        TextFont {
            font: asset_server.load(super::FONT_PATH),
            font_size: 28.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.7, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(60.0),
            left: Val::Px(20.0),

            ..default()
        },
        ComboText,
    ));
}

// Update score display system
pub fn update_score_display(
    mut score_query: Query<&mut Text, With<ScoreText>>,
    score: Res<GameScore>,
    localization: Res<LocalizationResource>,
) {
    // Update main score
    if let Ok(mut score_text) = score_query.single_mut() {
        let mut score_args = HashMap::new();
        score_args.insert("score".to_string(), score.current.into());
        score_text.0 = localization.get_text("hud-score", Some(&score_args));
    }
}

pub fn update_combo_display(
    mut combo_query: Query<&mut Text, With<ComboText>>,
    score: Res<GameScore>,
    localization: Res<LocalizationResource>,
) {
    // Update combo display
    if let Ok(mut combo_text) = combo_query.single_mut() {
        if score.combo > 1 {
            let mut combo_args = HashMap::new();
            combo_args.insert("multiplier".to_string(), score.combo.into());
            combo_text.0 = localization.get_text("hud-combo", Some(&combo_args));
        } else {
            combo_text.0 = String::new();
        }
    }
}

pub fn update_combo_system(mut score: ResMut<GameScore>, time: Res<Time>) {
    score.combo_timer.tick(time.delta());

    // Reset combo if timer expires
    if score.combo_timer.finished() && score.combo > 0 {
        score.combo = 0;
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_score_display,
            update_combo_display,
            update_combo_system,
        ),
    );
}
