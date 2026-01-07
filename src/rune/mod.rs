use std::marker::PhantomData;

use bevy::prelude::*;

mod modifier_rune;
mod runes;

pub use modifier_rune::*;
pub use runes::*;

use crate::ability::components::spawn::RandomSpawnOffset;
use crate::{common::Inertia, init::DespawnOnReset};

#[derive(Component)]
pub struct Collector {
    pub collect_radius: f32,
    pub magnet_radius: f32,
    pub magnet_force: f32,
}

#[derive(Component)]
pub struct RunePickup<T: Rune> {
    pub rune: T,
}

pub trait Rune: Send + Sync + 'static + Clone + std::fmt::Debug {
    fn register_systems(app: &mut App);

    fn emissive(&self) -> LinearRgba;
}

#[derive(Event)]
pub struct RuneSpawnEvent<T: Rune> {
    pub rune: T,
    pub position: Vec3,
}

#[derive(Event)]
pub struct RuneApplicationEvent<T: Rune> {
    pub entity: Entity,
    pub rune: T,
}

pub fn handle_rune_pickup<T: Rune>(
    mut commands: Commands,
    mut collection_events: EventWriter<RuneApplicationEvent<T>>,
    collector_query: Query<(Entity, &Transform, &Collector), Without<RunePickup<T>>>,
    pickup_query: Query<(Entity, &mut Transform, &RunePickup<T>)>,
    time: Res<Time>,
) {
    for (pickup_entity, mut pickup_transform, effect_pickup) in pickup_query {
        for (collector_entity, collector_transform, collector) in collector_query {
            let distance =
                (collector_transform.translation - pickup_transform.translation).length();

            if distance < collector.collect_radius {
                collection_events.write(RuneApplicationEvent::<T> {
                    entity: collector_entity,
                    rune: effect_pickup.rune.clone(),
                });

                let effect = effect_pickup.rune.clone();

                info!(
                    "Entity {collector_entity} picked up rune {pickup_entity} with effect {effect:?}"
                );

                commands.entity(pickup_entity).despawn()
            } else if distance < collector.magnet_radius {
                pickup_transform.translation = pickup_transform.translation.lerp(
                    collector_transform.translation,
                    collector.magnet_force / distance.powi(2)
                        * time.delta_secs()
                        * time.delta_secs(),
                );
            }
        }
    }
}

#[derive(Resource)]
pub struct RuneAssets<T: Rune> {
    mesh: Handle<Mesh>,
    _marker: PhantomData<T>,
}

pub fn setup_rune_assets<T: Rune>(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mesh_handle = asset_server.load::<Mesh>("models/ico.obj");

    commands.insert_resource(RuneAssets {
        mesh: mesh_handle,
        _marker: PhantomData::<T>,
    });
}

#[derive(Component)]
struct RuneAnimation {
    speed: f32,
}

pub fn spawn_runes<T: Rune>(
    mut commands: Commands,
    mut spawn_events: EventReader<RuneSpawnEvent<T>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<RuneAssets<T>>,
) {
    for spawn_event in spawn_events.read() {
        info!(
            "Rune '{:?}' spawned at {}",
            spawn_event.rune, spawn_event.position
        );

        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 1.0, 1.0),
            emissive: spawn_event.rune.emissive(),
            ..Default::default()
        });

        commands.spawn((
            Transform::from_translation(spawn_event.position),
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(material),
            RunePickup::<T> {
                rune: spawn_event.rune.clone(),
            },
            RuneAnimation {
                speed: 10.0 + fastrand::f32(),
            },
            DespawnOnReset,
            Inertia {
                prev_pos: spawn_event.position,
                damping: 0.95,
            },
            RandomSpawnOffset::new(0.05, 0.0),
        ));
    }
}

fn handle_rune_animation(time: Res<Time>, query: Query<(&mut Transform, &RuneAnimation)>) {
    for (mut transform, animation) in query {
        let s = f32::sin(animation.speed * time.elapsed_secs());

        transform.scale = Vec3::splat(1.0 + s * 0.4);
        transform.rotation *= Quat::from_rotation_y(animation.speed * time.elapsed_secs());
        //transform.translation.x = 0.5 + s * 0.2;
    }
}

pub fn register_rune<T: Rune>(app: &mut App) {
    app.add_event::<RuneApplicationEvent<T>>();
    app.add_event::<RuneSpawnEvent<T>>();
    app.add_systems(Update, setup_rune_assets::<T>);
    app.add_systems(Update, handle_rune_pickup::<T>);
    app.add_systems(PostUpdate, spawn_runes::<T>);
    T::register_systems(app);
}

pub fn plugin(app: &mut App) {
    register_rune::<ModifierRune>(app);
    register_rune::<HealRune>(app);

    app.add_systems(Update, handle_rune_animation);
}
