use bevy::app::App;

pub mod music;


pub fn plugin(app: &mut App) {
    music::plugin(app);
}