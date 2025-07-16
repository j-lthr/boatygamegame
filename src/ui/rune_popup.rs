use bevy::prelude::*;
use crate::{
    localization::LocalizationResource,
    rune::{RuneApplicationEvent, ModifierRune},
    init::DespawnOnReset,
    ui::common::{MoveAnimation, FadeAnimation},
};


pub fn setup_rune_popup_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(100.0),
            left: Val::Px(50.0),
            ..default()
        },
        Visibility::Hidden,
        Name::new("RunePopupContainer"),
    ));
}

pub fn handle_rune_pickup_ui(
    mut commands: Commands,
    mut pickup_events: EventReader<RuneApplicationEvent<ModifierRune>>,
    localization: Res<LocalizationResource>,
    container_query: Query<Entity, With<Name>>,
) {
    for event in pickup_events.read() {
        let localized_text = localization.format_stat(event.rune.modifier);
        let start_pos = Vec2::new(50.0, 100.0);
        
        // Create popup with animations
        let popup_entity = commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(start_pos.y),
                left: Val::Px(start_pos.x),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            MoveAnimation::downward(-50.0, start_pos), // Move down at 50px/sec
            FadeAnimation::fade_out(2.0), // Fade out over 2 seconds
            DespawnOnReset,
        )).with_children(|parent| {
            parent.spawn((
                Text::new(localized_text),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                FadeAnimation::fade_out(2.0), // Fade text too
            ));
        }).id();
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_rune_popup_ui);
    app.add_systems(Update, handle_rune_pickup_ui);
}