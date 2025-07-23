use std::f32;
use bevy::math::Vec2;

pub fn normal_dist_1d(mu: f32, sigma: f32) -> f32 {
    let u_1 = fastrand::f32();
    let u_2 = fastrand::f32();

    let r = f32::sqrt(-2.0 * f32::ln(u_1));

    sigma * r * f32::cos(2.0 * f32::consts::PI * u_2) + mu
}

pub fn normal_dist_2d(mu: Vec2, sigma: f32) -> Vec2 {
    let u_1 = fastrand::f32();
    let u_2 = fastrand::f32();

    let r = f32::sqrt(-2.0 * f32::ln(u_1));

    let x = f32::cos(2.0 * f32::consts::PI * u_2);
    let y = f32::sin(2.0 * f32::consts::PI * u_2);

    sigma * r * Vec2::new(x,y) + mu
}
