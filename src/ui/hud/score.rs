use crate::state::GameScore;
use bevy::prelude::*;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct ComboText;



// Setup the score UI
pub fn setup_score_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Score display
    commands.spawn((
        Text::new("Score: 0"),
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
) {
    // Update main score
    if let Ok(mut score_text) = score_query.single_mut() {
        score_text.0 = format!("{}", score.current);
    }
}

pub fn update_combo_display(
    mut combo_query: Query<&mut Text, With<ComboText>>,
    score: Res<GameScore>,
) {
    // Update combo display
    if let Ok(mut combo_text) = combo_query.single_mut() {
        if score.combo > 1 {
            combo_text.0 = format!("x{}", score.combo);
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
