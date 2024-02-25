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

#[derive(Component, Debug, Serialize, Deserialize)]
pub struct PlayerName(pub String);

#[derive(Event, Debug, Serialize, Deserialize)]
pub struct SetName(pub String);

pub struct PetriSharedSetup;

impl Plugin for PetriSharedSetup {
    fn build(&self, app: &mut App) {
        app.replicate::<Player>()
            .replicate::<PlayerColor>()
            .replicate::<PlayerPos>()
            .replicate::<PlayerName>()
            .add_client_event::<MoveDirection>(EventType::Ordered)
            .add_client_event::<SetName>(EventType::Ordered);
    }
}
