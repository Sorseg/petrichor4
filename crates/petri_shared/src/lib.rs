use bevy::prelude::*;
use bevy_replicon::{prelude::*, renet::ClientId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Component, Serialize, Deserialize)]
pub struct Player(pub ClientId);

/// An intention to move in a particular direction.
#[derive(Event, Debug, Default, Deserialize, Serialize)]
pub struct MoveDirection(pub Vec2);

#[derive(Component, Serialize, Deserialize)]
pub struct PlayerPos(pub GlobalTransform);

#[derive(Component, Serialize, Deserialize)]
pub struct PlayerColor(pub Color);

#[derive(Event, Debug, Serialize, Deserialize)]
pub struct SetName(pub String);

pub struct PetriSharedSetup;

impl Plugin for PetriSharedSetup {
    fn build(&self, app: &mut App) {
        app.replicate::<Player>()
            .replicate::<PlayerColor>()
            .replicate::<PlayerPos>()
            .replicate::<Name>()
            .add_client_event::<MoveDirection>(EventType::Ordered)
            .add_client_event::<SetName>(EventType::Ordered);
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
