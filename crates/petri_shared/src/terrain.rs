//! Terrain related items
//!
//! Density lattice generates collider and texture surfaces
//! using the marching cubes algorithm.
//! Cells are split into cubes of [TERRAIN_CHUNK_SIDE_LEN]^3
//! Only non-uniform cubes generate surfaces, so only they are
//! sent to the client and converted to colliders   

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::ReplicatedPos;

pub const TERRAIN_CHUNK_SIDE_LEN: usize = 10;
pub const TERRAIN_UNIT_SIDE_LENGTH: f32 = 0.25;

#[derive(Debug, Serialize, Deserialize, Default)]
pub enum TerrainType {
    #[default]
    Air = 0,
    MoonDust = 1,
}

#[derive(Debug, Component, Serialize, Deserialize, Default)]
pub struct TerrainData(
    /// FIXME: check if there is a more ergonomic structure for the data than the array
    /// Index order `z,y,x`
    /// Cell is (type, amount)
    pub  [[[(TerrainType, u8); TERRAIN_CHUNK_SIDE_LEN]; TERRAIN_CHUNK_SIDE_LEN];
        TERRAIN_CHUNK_SIDE_LEN],
);

#[derive(Bundle)]
pub struct TerrainChunkBundle {
    pub data: TerrainData,
    /// Coordinates need to be multiples of [TERRAIN_UNIT_SIDE_LENGTH] * [TERRAIN_CHUNK_SIDE_LEN]
    pub pos: ReplicatedPos,
}
