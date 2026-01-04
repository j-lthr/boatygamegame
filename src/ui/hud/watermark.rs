pub use bevy::prelude::*;

pub fn spawn_watermark(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Text::new(format!("Developer Build v{}", env!("CARGO_PKG_VERSION"))),
        TextFont {
            font: asset_server.load(super::FONT_PATH),
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.2)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            left: Val::Px(20.0),
            ..default()
        },
    ));
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_watermark);
}
