use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct GameInit;

#[derive(Component)]
pub struct DespawnOnReset;

pub fn trigger(mut commands: Commands) {
    commands.run_schedule(GameInit);
}

pub fn plugin(app: &mut App) {
    //app.init_schedule(GameInit);
    app.add_systems(Startup, trigger);
}
