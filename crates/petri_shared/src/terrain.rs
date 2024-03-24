//! Terrain related items
//!
//! Density lattice generates collider and texture surfaces
//! using the marching cubes algorithm.
//! Cells are split into cubes of [TERRAIN_CHUNK_SIDE_LEN]^3
//! Only non-uniform cubes generate surfaces, so only they are
//! sent to the client and converted to colliders   

use bevy::{
    prelude::*,
    utils::{info, HashMap},
};
use itertools::iproduct;
use serde::{Deserialize, Serialize};
use transvoxel::transition_sides::TransitionSides;

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
            let val = gx / 10.0 + gz / 30.0 - gy;
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
    // on AMD Ryzen 3900X in release mode (without lto)
    // This method takes ~150 microseconds
    pub fn get_polygons(&self) -> transvoxel::generic_mesh::Mesh<f32> {
        // FIXME: incorporate voxel size
        let threshold = 0f32;

        use transvoxel::prelude::*;
        let subdivisions = TERRAIN_CHUNK_SIDE_LEN;

        let block = Block::from([0.0, 0.0, 0.0], 10.0, subdivisions);
        let transition_sides = TransitionSides::full();

        use transvoxel::generic_mesh::GenericMeshBuilder;
        let sampled = std::sync::Mutex::new(std::collections::HashSet::new());

        let value_getter = |x: f32, y: f32, z: f32| {
            // make value in the middle of the block
            let x = x + 0.49;
            let y = y + 0.49;
            let z = z + 0.49;

            // if the sampling point is outside the cell, treat it as air,
            // so the algorithm generates walls for colliders
            // for proper decomposition
            // but these are unnecessary for the visible mesh, so
            // there is a possible optimization here to make a separate run for
            // visible mesh
            if [x, y, z].iter().any(|n| !(0.0..=9.9).contains(n)) {
                // this is a low value, so the wall is generated close to the
                // grid
                return -0.1;
            }

            let (terrain_type, val) = &self.0[z as usize][y as usize][x as usize];

            if matches!(terrain_type, TerrainType::Air) {
                -0.1
            } else {
                1.0 / (256.0 - *val as f32)
            }
        };
        let builder = extract_from_field(
            &|x, y, z| {
                let res = value_getter(x, y, z);
                if (3.0..4.0).contains(&x) || (3.0..4.0).contains(&z) {
                    sampled
                        .lock()
                        .unwrap()
                        .insert(((y * 1000.0) as i32, (res * 1000.0) as i32));
                }
                res
            },
            &block,
            threshold,
            transition_sides,
            GenericMeshBuilder::new(),
        );

        let res = builder.build();
        let mut sampled = sampled.lock().unwrap().iter().copied().collect::<Vec<_>>();
        sampled.sort();
        info!("Sampled {:?}", sampled);
        res
    }
}
