use bevy::prelude::{info, GlobalTransform, Quat, Query, Time, Virtual, With, Without};
use bevy::{
    app::{RunMode, ScheduleRunnerPlugin},
    prelude::{default, App, Commands, Startup, Transform, TransformBundle, Update, Vec3},
    time::TimePlugin,
};
use bevy_rapier3d::prelude::{
    Collider, ExternalForce, LockedAxes, NoUserData, RapierPhysicsPlugin, RigidBody,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::{Duration, Instant};

#[derive(bevy::prelude::Component)]
struct Player;

fn create_world_with_colliders(n: u32) -> App {
    let mut app = App::new();
    app.add_plugins((
        RapierPhysicsPlugin::<NoUserData>::default(),
        TimePlugin,
        ScheduleRunnerPlugin {
            run_mode: RunMode::Once,
        },
    ))
    .add_systems(Startup, move |mut commands: Commands| {
        commands.spawn((
            Player,
            TransformBundle::from(Transform::from_xyz(0.0, 4.0, 0.0)),
            Collider::capsule_y(0.5, 0.5),
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED,
            // to trigger collision events
            ExternalForce {
                force: Vec3 {
                    x: 10000.0,
                    y: 0.0,
                    z: 0.0,
                },
                ..default()
            },
        ));
        let mut colliders = Vec::with_capacity((n as usize).pow(2));
        for x in 0..n {
            for z in 0..n {
                colliders.push((
                    bevy_rapier3d::prelude::Vect {
                        x: x as f32 - n as f32 / 2.0,
                        y: 0.0,
                        z: z as f32 - n as f32 / 2.0,
                    },
                    Quat::from_rotation_x(0.0),
                    Collider::cuboid(0.5, 0.5, 0.5),
                ));
            }
        }
        commands.spawn((Collider::compound(colliders), RigidBody::Fixed));
    })
    // debug
    .add_systems(
        Update,
        // teleport back when out of map
        move |mut q: Query<(&GlobalTransform, &mut Transform), With<Player>>| {
            let (glob, mut tr) = q.single_mut();
            let glob = glob.translation();
            if glob.to_array().iter().any(|c| *c > n as f32 / 2.0) {
                *tr = Transform::from_xyz(0.0, 4.0, 0.0);
            }
        },
    );
    app.finish();
    app.cleanup();
    app
}

fn advance_world(app: &mut App) {
    app.world
        .resource_mut::<Time>()
        .advance_by(Duration::from_millis(10));
    app.update();
}

fn criterion_benchmark(c: &mut Criterion) {
    for n in [100, 1000] {
        let mut world = create_world_with_colliders(n);
        let start = Instant::now();
        for _ in 0..10 {
            world.update();
        }
        println!("First 10 world iteration time {:?}", start.elapsed());

        c.bench_function(&format!("phys sim with {} colliders", n * n), |b| {
            b.iter(|| advance_world(&mut world));
        });
    }
}


criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

// fn main() {
//     let mut world = create_world_with_colliders(100);
//     for _ in 0..100 {
//         advance_world(&mut world);
//     }
//     let sys = world
//         .world
//         .register_system(|q: Query<&GlobalTransform, With<Player>>| {
//             println!("{:?}", q.single());
//         });
//     world.world.run_system(sys).unwrap();
// }
