use bevy::prelude::*;
use crate::ability::components::common::Lifetime;
use crate::ability::components::visual::LifetimeFadeout;
use crate::player::Player;
use crate::ui::common::FadeAnimation;
use crate::utils::{normal_dist_1d, normal_dist_2d};

#[derive(Resource)]
pub struct StarAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

#[derive(Component)]
pub struct StarEffect {
    pub spawn_rate: f32,
    pub spawn_radius: f32,
}

#[derive(Component)]
pub struct Star {
    pub timer: Timer,
    pub scale: f32,
}

pub fn setup_stars(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(
      StarAssets {
          mesh: meshes.add(Sphere::new(0.1)),
          material: materials.add(StandardMaterial {
              emissive: LinearRgba::rgb(20.0,20.0,20.0),
              ..Default::default()
          })
      }
    );
}

pub fn spawn_stars(mut commands: Commands, assets: Res<StarAssets>, query: Query<(&Transform, &StarEffect)>, time: Res<Time>) {
    for (transform, effect) in query.iter() {

        let mut prob = effect.spawn_rate * time.delta_secs();

         while fastrand::f32() < prob {

             let pos = transform.translation + normal_dist_2d(Vec2::ZERO, effect.spawn_rate).xxy().with_y(normal_dist_1d(-20.0, 0.0));

             commands.spawn((
                 Transform::from_translation(pos),
                 Mesh3d(assets.mesh.clone()),
                 MeshMaterial3d(assets.material.clone()),
                 Star {
                     timer: Timer::from_seconds(10.0, TimerMode::Once),
                     scale: normal_dist_1d(1.0, 0.2),
                 }
             ));

             prob -= 1.0;
         }
    }
}

pub fn handle_stars(mut commands: Commands, query: Query<(Entity, &mut Transform, &mut Star), Without<Player>>, player: Query<&Transform, With<Player>>, time: Res<Time>) -> Result {

    let player_transform = player.single()?;

    for (entity, mut transform, mut star) in query {
        star.timer.tick(time.delta());

        let t = star.timer.elapsed_secs();
        let r = star.timer.remaining_secs();

        if t < 1.0 {
            transform.scale = Vec3::splat(t * star.scale);
        }
        if r < 1.0 {
            transform.scale = Vec3::splat(r * star.scale);
        }

        if star.timer.finished() {
            commands.entity(entity).despawn();
        }

        let d2p = (player_transform.translation - transform.translation).with_y(0.0);

        //transform.translation += d2p.cross(Vec3::Y) * time.delta_secs() * d2p.length() * 0.002 * (0.1 * time.elapsed_secs()).sin();
    }

    Ok(())
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_stars);
    app.add_systems(Update, (spawn_stars, handle_stars));
}