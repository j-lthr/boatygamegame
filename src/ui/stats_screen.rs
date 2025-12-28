use crate::localization::LocalizationResource;
use crate::modifiers::ModifierStack;
use crate::player::Player;
use bevy::prelude::*;

#[derive(Component)]
pub struct StatsScreen;

pub fn handle_stats_screen_visibility(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    localization: Res<LocalizationResource>,
    player_query: Query<&ModifierStack, With<Player>>,
    existing_screen: Query<Entity, With<StatsScreen>>,
) {
    let should_show = keyboard_input.pressed(KeyCode::Tab);
    let screen_exists = !existing_screen.is_empty();

    match (should_show, screen_exists) {
        (true, false) => {
            // Show screen
            if let Ok(modifier_stack) = player_query.single() {
                spawn_stats_screen(&mut commands, &asset_server, &localization, modifier_stack);
            }
        }
        (false, true) => {
            // Hide screen
            for entity in existing_screen.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
        _ => {} // No change needed
    }
}

fn spawn_stats_screen(
    commands: &mut Commands,
    asset_server: &AssetServer,
    localization: &LocalizationResource,
    modifier_stack: &ModifierStack,
) {
    // Get all active modifiers
    let modifiers = modifier_stack.list_compound_mods();

    // Background overlay
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::End,
                align_items: AlignItems::Center,
                padding: UiRect::right(Val::Px(50.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.1)),
            StatsScreen,
        ))
        .with_children(|parent| {
            // Stats panel
            parent
                .spawn((
                    Node {
                        width: Val::Px(400.0),
                        height: Val::Auto,
                        padding: UiRect::all(Val::Px(20.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(10.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
                    BorderRadius::all(Val::Px(10.0)),
                ))
                .with_children(|panel| {
                    // Title
                    panel.spawn((
                        Text::new(localization.get_text("stats-screen-title", None)),
                        TextFont {
                            font: asset_server.load("fonts/Jersey15-Regular.ttf"),
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 1.0, 1.0)),
                        Node {
                            margin: UiRect::bottom(Val::Px(20.0)),
                            ..default()
                        },
                    ));

                    // Modifiers list
                    if modifiers.is_empty() {
                        panel.spawn((
                            Text::new("No modifiers active"),
                            TextFont {
                                font: asset_server.load("fonts/Jersey15-Regular.ttf"),
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.7, 0.7, 0.7)),
                        ));
                    } else {
                        for modifier in modifiers {
                            panel.spawn((
                                Text::new(localization.format_stat(modifier)),
                                TextFont {
                                    font: asset_server.load("fonts/Jersey15-Regular.ttf"),
                                    font_size: 24.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                Node {
                                    margin: UiRect::vertical(Val::Px(2.0)),
                                    ..default()
                                },
                            ));
                        }
                    }
                });
        });
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        handle_stats_screen_visibility,
    );
}