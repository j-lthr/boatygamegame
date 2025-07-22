use bevy::prelude::*;
use crate::common::HealthPool;
use crate::player::Player;

#[derive(Component)]
pub struct PlayerHealthBar;

#[derive(Component)]
pub struct PlayerHealthBarFill;

#[derive(Component)]
pub struct PlayerHealthText;

#[derive(Component)]
pub struct PlayerHealthDisplay {
    pub current_displayed_health: f32,
    pub current_displayed_max: f32,
    pub lerp_speed: f32,
}

pub fn setup_player_health_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Container for health bar and text
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(40.0),
                left: Val::Px(20.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            },
            PlayerHealthBar,
            PlayerHealthDisplay {
                current_displayed_health: 50.0,
                current_displayed_max: 50.0,
                lerp_speed: 10.0,
            },
        ))
        .with_children(|parent| {
            // Health bar background
            parent
                .spawn((
                    Node {
                        width: Val::Px(400.0),
                        height: Val::Px(30.0),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                ))
                .with_children(|bar_parent| {
                    // Health bar fill
                    bar_parent.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(1.0, 1.0, 1.0)),
                        PlayerHealthBarFill,
                    ));
                });

            // Health text
            parent.spawn((
                Text::new("50 / 50"),
                TextFont {
                    font: asset_server.load(super::FONT_PATH),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                Node {
                    ..default()
                },
                PlayerHealthText,
            ));
        });
}

pub fn update_player_health_display(
    player_query: Query<&HealthPool, With<Player>>,
    mut health_display_query: Query<&mut PlayerHealthDisplay, With<PlayerHealthBar>>,
    mut health_fill_query: Query<(&mut Node, &mut BackgroundColor), With<PlayerHealthBarFill>>,
    mut health_text_query: Query<&mut Text, With<PlayerHealthText>>,
    time: Res<Time>,
) {
    if let (Ok(health_pool), Ok(mut health_display)) = 
        (player_query.single(), health_display_query.single_mut()) {
        
        let target_health = health_pool.current_health as f32;
        let target_max = health_pool.max_health as f32;
        let lerp_factor = (health_display.lerp_speed * time.delta_secs()).min(1.0);
        
        // Lerp displayed values towards actual values
        health_display.current_displayed_health = health_display.current_displayed_health
            .lerp(target_health, lerp_factor);
        health_display.current_displayed_max = health_display.current_displayed_max
            .lerp(target_max, lerp_factor);
        
        let health_fraction = if health_display.current_displayed_max > 0.0 {
            health_display.current_displayed_health / health_display.current_displayed_max
        } else {
            0.0
        };
        
        // Update health bar
        if let Ok((mut fill_node, mut fill_color)) = health_fill_query.single_mut() {
            fill_node.width = Val::Percent((health_fraction * 100.0).max(0.0));
            
            // White until 20%, then red
            if health_fraction > 0.2 {
                fill_color.0 = Color::srgb(1.0, 1.0, 1.0); // White
            } else {
                fill_color.0 = Color::srgb(0.8, 0.2, 0.2); // Red
            }
        }
        
        // Update health text (show actual values, not lerped for precision)
        if let Ok(mut health_text) = health_text_query.single_mut() {
            health_text.0 = format!("{} / {}", target_health as i32, target_max as i32);
        }
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_player_health_ui)
        .add_systems(Update, update_player_health_display);
}