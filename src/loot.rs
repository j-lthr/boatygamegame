use bevy::prelude::*;

use crate::{
    event::{DeathEvent, EventSource},
    rune::{Rune, RuneSpawnEvent},
};

pub struct RuneDrop<T: Rune> {
    pub rune: T,
}

impl<T: Rune> EventSource<Vec3> for RuneDrop<T> {
    fn send_event(&self, mut commands: Commands, input: Vec3) {
        commands.send_event(RuneSpawnEvent {
            position: input,
            rune: self.rune.clone(),
        });
    }
}

#[derive(Component)]
pub struct DropTable {
    entries: Vec<(f32, Box<dyn EventSource<Vec3>>)>,
}

impl DropTable {
    pub fn drop_random(&self, mut commands: Commands, position: Vec3) {
        let x = fastrand::f32();

        for (p, event_source) in &self.entries {
            if *p > x {
                event_source.send_event(commands.reborrow(), position);
                break;
            }
        }
    }

    fn from_weighted_list(weighted_items: Vec<(f32, Box<dyn EventSource<Vec3>>)>) -> Self {
        if weighted_items.is_empty() {
            return Self {
                entries: Vec::new(),
            };
        }

        let total_weight: f32 = weighted_items.iter().map(|(weight, _)| weight).sum();

        if total_weight <= 0.0 {
            return Self {
                entries: Vec::new(),
            };
        }

        let mut cumulative_prob = 0.0;
        let mut entries = Vec::new();

        for (weight, system) in weighted_items {
            cumulative_prob += weight / total_weight;
            entries.push((cumulative_prob, system));
        }

        Self { entries }
    }
}

pub struct DropTableBuilder {
    weighted_items: Vec<(f32, Box<dyn EventSource<Vec3>>)>,
}

impl DropTableBuilder {
    pub fn new() -> Self {
        Self {
            weighted_items: vec![],
        }
    }

    pub fn add_rune<T: Rune + 'static>(mut self, weight: f32, rune: T) -> Self {
        self.weighted_items
            .push((weight, Box::new(RuneDrop { rune })));
        self
    }

    pub fn build(self) -> DropTable {
        DropTable::from_weighted_list(self.weighted_items)
    }
}

pub fn handle_drops(
    mut commands: Commands,
    mut death_events: EventReader<DeathEvent>,
    query: Query<(&Transform, &DropTable)>,
) {
    for death_event in death_events.read() {
        let result = query.get(death_event.entity);
        if let Ok((transform, drop_table)) =  result {
            let drop_position = transform.translation;
            info!("Received death event for entity {}, dropping loot.", death_event.entity);
            drop_table.drop_random(commands.reborrow(), drop_position);
        } else {
            unsafe {
                dbg!(result.unwrap_err_unchecked());
            }
        }
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update, handle_drops);
}
