use bevy::app::App;

pub mod player_health;
pub mod score;
pub mod watermark;
pub mod wave;

const FONT_PATH: &str = "fonts/Jersey15-Regular.ttf";

pub fn plugin(app: &mut App) {
    player_health::plugin(app);
    score::plugin(app);
    watermark::plugin(app);
    wave::plugin(app);
}
