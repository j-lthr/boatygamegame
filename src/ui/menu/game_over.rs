use crate::audio::music::PlayMusicEvent;
use crate::enemy::spawn::SpawnerState;
use crate::init::{DespawnOnReset, GameInit};
use crate::state::{GameScore, GameState};
use crate::localization::LocalizationResource;
use bevy::prelude::*;
use std::collections::HashMap;

const FONT_PATH: &str = "fonts/Jersey15-Regular.ttf";

#[derive(Component)]
pub struct GameOverScreen;

#[derive(Component)]
pub struct FinalScoreText;

// Setup game over screen
pub fn setup_game_over_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    score: Res<GameScore>,
    localization: Res<LocalizationResource>,
) {
    // Background overlay
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            GameOverScreen,
        ))
        .with_children(|parent| {
            // Game Over title
            parent.spawn((
                Text::new(localization.get_text("game-over-title", None)),
                TextFont {
                    font: asset_server.load(FONT_PATH),
                    font_size: 64.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.2, 0.2)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            // Final score
            let mut score_args = HashMap::new();
            score_args.insert("score".to_string(), score.current.into());
            parent.spawn((
                Text::new(localization.get_text("game-over-final-score", Some(&score_args))),
                TextFont {
                    font: asset_server.load(FONT_PATH),
                    font_size: 36.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
                FinalScoreText,
            ));

            // Kill count
            let mut kills_args = HashMap::new();
            kills_args.insert("kills".to_string(), score.kills.into());
            parent.spawn((
                Text::new(localization.get_text("game-over-enemies-defeated", Some(&kills_args))),
                TextFont {
                    font: asset_server.load(FONT_PATH),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // Restart instructions
            parent.spawn((
                Text::new(localization.get_text("game-over-restart-instructions", None)),
                TextFont {
                    font: asset_server.load(FONT_PATH),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 1.0)),
            ));
        });

        commands.send_event(PlayMusicEvent {
            track_name: "audio/bg_game_over.wav",
            mode: bevy::audio::PlaybackMode::Once,
        });

}

// Handle input on game over screen
pub fn handle_game_over_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        next_state.set(GameState::Playing);
    }

    if keyboard_input.just_pressed(KeyCode::Escape) {
        app_exit_events.write(AppExit::Success);
    }
}

// Update game over screen (for any animations or dynamic content)
pub fn update_game_over_screen(
    mut final_score_query: Query<&mut Text, With<FinalScoreText>>,
    score: Res<GameScore>,
    localization: Res<LocalizationResource>,
) {
    // Add a pulsing effect to the final score
    if let Ok(mut score_text) = final_score_query.single_mut() {
        let mut score_args = HashMap::new();
        score_args.insert("score".to_string(), score.current.into());
        score_text.0 = localization.get_text("game-over-final-score", Some(&score_args));
    }
}
// Cleanup game over screen
pub fn cleanup_game_over_screen(
    mut commands: Commands,
    despawn_query: Query<Entity, With<DespawnOnReset>>,
    game_over_query: Query<Entity, With<GameOverScreen>>,
) {
    commands.remove_resource::<GameScore>();
    commands.remove_resource::<SpawnerState>();
    commands.init_resource::<GameScore>();
    commands.init_resource::<SpawnerState>();

    for entity in &game_over_query {
        commands.entity(entity).despawn();
    }

    for entity in &despawn_query {
        commands.entity(entity).despawn();
    }

    commands.run_schedule(GameInit);
}
