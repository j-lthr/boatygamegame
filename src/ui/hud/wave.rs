use bevy::prelude::*;

use crate::enemy::spawn::SpawnerState;

#[derive(Component)]
pub struct WaveNumberText;

#[derive(Component)]
pub struct WaveTimer;

pub fn update_wave_number_display(
    mut wave_number_query: Query<&mut Text, With<WaveNumberText>>,
    spawner_state: Res<SpawnerState>,
) {
    if let Ok(mut wave_number_text) = wave_number_query.single_mut() {
        wave_number_text.0 = format!("Wave {}", spawner_state.wave_index);
    }
}

pub fn update_wave_timer(
    wave_timer_query: Query<&mut Node, With<WaveTimer>>,
    spawner_state: Res<SpawnerState>,
) {
    for mut wave_timer_node in wave_timer_query {
        wave_timer_node.width = Val::Px(100.0 * spawner_state.wave_timer.fraction_remaining());
    }
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Percent(0.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },))
        .with_children(|spawner| {
            spawner.spawn((
                Text::new(""),
                Node {
                    position_type: PositionType::Relative,
                    height: Val::Px(10.0),
                    top: Val::Px(33.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(1.0, 1.0, 1.0)),
                WaveTimer,
            ));

            spawner.spawn((
                Text::new(""),
                Node {
                    position_type: PositionType::Relative,
                    height: Val::Px(10.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(1.0, 1.0, 1.0)),
                WaveTimer,
            ));

            spawner.spawn((
                Text::new(""),
                TextFont {
                    font: asset_server.load(super::FONT_PATH),
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                TextLayout::new_with_justify(JustifyText::Center),
                Node {
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                WaveNumberText,
            ));
        });
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup);
    app.add_systems(Update, (update_wave_number_display, update_wave_timer));
}
