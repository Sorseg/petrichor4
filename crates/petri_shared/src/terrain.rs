//! Terrain related items
//!
//! Density lattice generates collider and texture surfaces
//! using the marching cubes algorithm.
//! Cells are split into cubes of [TERRAIN_CHUNK_SIDE_LEN]^3
//! Only non-uniform cubes generate surfaces, so only they are
//! sent to the client and converted to colliders   

use bevy::{prelude::*, utils::HashMap};
use itertools::iproduct;
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

pub fn sample_terrain() -> HashMap<(isize, isize, isize), TerrainData> {
    let mut chunks: HashMap<_, TerrainData> = HashMap::new();
    for (chunk_x, chunk_y, chunk_z) in iproduct!(-5_isize..=5, -1_isize..=1, -5_isize..=5) {
        let mut data = TerrainData::default();
        for (x, y, z) in iproduct!(0..10, 0..10, 0..10) {
            let (gx, gy, gz) = (
                (chunk_x * 10 + x) as f32,
                (chunk_y * 10 + y) as f32,
                (chunk_z * 10 + z) as f32,
            );
            let val = (gx / 3.0).sin() + (gz / 3.0).sin() - gy;
            data.0[z as usize][y as usize][x as usize] = if val > 0.0 {
                (TerrainType::MoonDust, (val * 255.0) as u8)
            } else {
                (TerrainType::Air, 0)
            }
        }
        chunks.insert((chunk_x, chunk_y, chunk_z), data);
    }
    chunks
}

impl TerrainData {
    // This method takes ~126 microseconds on my machine per invocation with opt-level 1
    pub fn get_polygons(&self) -> transvoxel::generic_mesh::Mesh<f32> {
        let threshold = 0f32;

        // Then you need to decide for which region of the world you want to generate the mesh, and how
        // many subdivisions should be used (the "resolution"). You also need to tell which sides of the
        // block need to be transition (double-resolution) faces. We use `no_side` here for simplicity,
        // and will get just a regular Marching Cubes extraction, but the Transvoxel transitions can be
        // obtained simply by providing some sides instead (that is shown a bit later):
        use transvoxel::prelude::*;
        let subdivisions = 11;
        let block = Block::from([-0.1, -0.1, -0.1], 11.0, subdivisions);
        let transition_sides = transition_sides::no_side();

        // Finally, you can run the mesh extraction:
        use transvoxel::generic_mesh::GenericMeshBuilder;
        let builder = GenericMeshBuilder::new();
        let builder = extract_from_field(
            &|x: f32, y: f32, z: f32| {
                // if the sampling point is outside the cell, treat it as air,
                // so the algorithm generates walls for colliders
                // for proper decomposition
                // but these are unnecessary for the visible mesh, so
                // there is a possible optimization here to make a separate run for
                // visible mesh
                if [x, y, z].iter().any(|n| !(0.0..=9.0).contains(n)) {
                    return -1.0;
                }

                let (terrain_type, val) =
                    &self.0[z.round() as usize][y.round() as usize][x.round() as usize];
                if matches!(terrain_type, TerrainType::Air) {
                    -0.1
                } else {
                    *val as f32 / 255.0
                }
            },
            &block,
            threshold,
            transition_sides,
            builder,
        );
        builder.build()
    }
}
