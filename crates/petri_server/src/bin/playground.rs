use std::time::Instant;

use bevy::{
    app::{App, RunMode, ScheduleRunnerPlugin},
    prelude::{Capsule3d, Commands, Mesh, Meshable, Res, Startup, Timer, Torus, Update, Vec3},
    render::mesh::{Indices, VertexAttributeValues},
    time::{Time, TimePlugin},
    MinimalPlugins,
};
use bevy_rapier3d::prelude::{Collider, NoUserData, RapierPhysicsPlugin};

fn main() {}

fn count_iter_time(t: Res<Time>) {
    println!("Counting")
}

fn add_colliders(mut commands: Commands) {
    println!("Sup!")
}
