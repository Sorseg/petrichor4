use bevy::prelude::*;
use bevy_replicon::{prelude::*, renet::ClientId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Component, Serialize, Deserialize)]
pub struct Player(pub ClientId);

/// An intention to move in a particular direction.
#[derive(Event, Debug, Default, Deserialize, Serialize)]
pub struct MoveDirection(pub Vec2);

#[derive(Component, Serialize, Deserialize)]
pub struct ReplicatedPos(pub GlobalTransform);

#[derive(Component, Serialize, Deserialize)]
pub struct Tint(pub Color);

#[derive(Event, Debug, Serialize, Deserialize)]
pub struct SetName(pub String);

#[derive(Component, Debug, Serialize, Deserialize)]
pub enum Appearance {
    Capsule,
    Box,
}

#[derive(Bundle)]
pub struct ReplicationBundle {
    tint: Tint,
    appearance: Appearance,
    pos: ReplicatedPos,
    replicate: Replication,
}

impl ReplicationBundle {
    pub fn new(tint: Tint, appearance: Appearance) -> Self {
        Self {
            tint,
            appearance,
            pos: ReplicatedPos(default()),
            replicate: Replication,
        }
    }
}

pub struct PetriReplicationSetupPlugin;

impl Plugin for PetriReplicationSetupPlugin {
    fn build(&self, app: &mut App) {
        _ = app
            // components
            .replicate::<Player>()
            .replicate::<Tint>()
            .replicate::<ReplicatedPos>()
            .replicate::<Appearance>()
            .replicate::<Name>()
            // events
            .add_client_event::<MoveDirection>(EventType::Ordered)
            .add_client_event::<SetName>(EventType::Ordered)
    }
}

pub fn get_player_capsule_size() -> (f32, f32) {
    // fatness
    let capsule_diameter = 0.8;
    // capsule total height
    let capsule_total_height = 1.4;

    let capsule_total_half_height = capsule_total_height / 2.0;
    let capsule_segment_half_height = capsule_total_half_height - (capsule_diameter / 2.0);
    (capsule_diameter, capsule_segment_half_height)
}
