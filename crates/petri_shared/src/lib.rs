pub mod terrain;

use bevy::prelude::*;
use bevy_replicon::{prelude::*, renet::ClientId};
use serde::{Deserialize, Serialize};

pub const PLAYER_HEIGHT: f32 = 1.0;

#[derive(Debug, Component, Serialize, Deserialize)]
pub struct Player(pub ClientId);

/// An intention to move in a particular direction.
#[derive(Event, Debug, Default, Deserialize, Serialize)]
pub struct MoveDirection(pub Vec2);

/// FIXME: Can this be removed to just replicate [Transform]?
/// e.g. https://github.com/cryscan/dollis/blob/2ebe33535b7c3ea291927ffb263f7b4ecdd19d0d/src/core/network.rs#L26
#[derive(Component, Serialize, Deserialize)]
pub struct ReplicatedPos(pub GlobalTransform);

/// Send from the client when it moves its eyes
#[derive(Event, Serialize, Deserialize)]
pub struct Aim(pub Direction3d);

#[derive(Component, Serialize, Deserialize)]
pub struct ReplicatedAim(pub Direction3d);

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
    aim: ReplicatedAim,
    replicate: Replication,
}

impl ReplicationBundle {
    pub fn new(tint: Tint, appearance: Appearance) -> Self {
        Self {
            tint,
            appearance,
            pos: ReplicatedPos(default()),
            // look forward by default
            aim: ReplicatedAim(Direction3d::NEG_Z),
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
            .replicate::<ReplicatedAim>()
            .replicate::<Appearance>()
            .replicate::<Name>()
            // events
            .add_client_event::<AdminCommand>(EventType::Ordered)
            .add_client_event::<MoveDirection>(EventType::Ordered)
            .add_client_event::<SetName>(EventType::Ordered)
            .add_client_event::<Aim>(EventType::Unordered)
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

#[derive(Debug, Event, Serialize, Deserialize)]
pub enum AdminCommand {
    SpawnBoxWall { side_size: u8, at: Vec3 },
}
