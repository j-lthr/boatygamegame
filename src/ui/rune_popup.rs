use crate::{
    init::DespawnOnReset,
    localization::LocalizationResource,
    rune::{ModifierRune, RuneApplicationEvent},
    ui::common::{FadeAnimation, MoveAnimation},
};
use bevy::prelude::*;

#[derive(Component)]
pub struct RunePopup {
    pub initial_y: f32,
}

#[derive(Resource)]
pub struct RunePopupManager {
    pub next_slot: u32,
    pub slot_height: f32,
    pub base_y: f32,
}

impl Default for RunePopupManager {
    fn default() -> Self {
        Self {
            next_slot: 0,
            slot_height: 64.0,
            base_y: 100.0,
        }
    }
}

pub fn setup_rune_popup_ui(mut commands: Commands) {
    commands.insert_resource(RunePopupManager::default());
}

pub fn handle_rune_pickup_ui(
    mut commands: Commands,
    mut pickup_events: EventReader<RuneApplicationEvent<ModifierRune>>,
    mut popup_manager: ResMut<RunePopupManager>,
    localization: Res<LocalizationResource>,
    asset_server: Res<AssetServer>,
) {
    for event in pickup_events.read() {
        let localized_text = localization.format_stat(event.rune.modifier);

        // Calculate position for this popup
        let y_position =
            popup_manager.base_y + (popup_manager.next_slot as f32 * popup_manager.slot_height);
        let start_pos = Vec2::new(50.0, y_position);

        // Create popup with animations and background
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(start_pos.y),
                    left: Val::Px(start_pos.x),
                    padding: UiRect::all(Val::Px(12.0)),
                    margin: UiRect::all(Val::Px(2.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
                BorderColor(event.rune.color),
                BorderRadius::all(Val::Px(8.0)),
                //MoveAnimation::new(Vec2::new(100.0, -30.0), start_pos), // Move right and slightly up
                FadeAnimation::fade_out(3.0), // Fade out over 3 seconds
                RunePopup {
                    initial_y: y_position,
                },
                DespawnOnReset,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(localized_text),
                    TextFont {
                        font: asset_server.load("fonts/Jersey15-Regular.ttf"),
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    //FadeAnimation::fade_out(3.0), // Fade text too
                ));
            });

        // Increment slot for next popup
        popup_manager.next_slot += 1;
    }
}

// System to clean up popup slots when popups are despawned
pub fn cleanup_popup_slots(
    mut popup_manager: ResMut<RunePopupManager>,
    popup_query: Query<&RunePopup>,
) {
    let active_popup_count = popup_query.iter().count() as u32;

    // Reset slot counter if no popups are active
    if active_popup_count == 0 {
        popup_manager.next_slot = 0;
    }
}

// System to move existing popups up when space becomes available
pub fn compact_popups(
    mut popup_query: Query<(&mut Node, &RunePopup, &mut MoveAnimation)>,
    popup_manager: Res<RunePopupManager>,
) {
    let mut popups: Vec<_> = popup_query.iter_mut().collect();
    popups.sort_by(|a, b| a.1.initial_y.partial_cmp(&b.1.initial_y).unwrap());

    for (slot_index, (node, _popup, mut move_anim)) in popups.into_iter().enumerate() {
        let target_y = popup_manager.base_y + (slot_index as f32 * popup_manager.slot_height);

        // Smoothly move to new position if needed
        if let Val::Px(current_y) = node.top
            && (current_y - target_y).abs() > 5.0
        {
            // Update the movement animation to head towards the new target
            let current_pos = Vec2::new(50.0, current_y);
            let target_pos = Vec2::new(50.0, target_y);
            let direction = (target_pos - current_pos).normalize_or_zero();
            move_anim.velocity = direction * 200.0; // Fast compacting movement
        }
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_rune_popup_ui);
    app.add_systems(
        Update,
        (handle_rune_pickup_ui, cleanup_popup_slots, compact_popups),
    );
}
