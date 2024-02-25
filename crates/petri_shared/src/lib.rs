use bevy::prelude::*;
use bevy_replicon::renet::ClientId;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize)]
pub struct Player(pub ClientId);

/// A movement event for the controlled box.
#[derive(Event, Debug, Default, Deserialize, Serialize)]
pub struct MoveDirection(pub Vec2);

#[derive(Component, Serialize, Deserialize)]
pub struct PlayerPos(pub Vec3);

#[derive(Component, Serialize, Deserialize)]
pub struct PlayerColor(pub Color);
