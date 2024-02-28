use bevy::{
    app::{App, Plugin},
    prelude::{Color, Commands, Component, Name, Startup, Transform, TransformBundle},
    utils::default,
};
use bevy_rapier3d::{
    dynamics::LockedAxes,
    prelude::{Collider, RigidBody},
};
use bevy_replicon::prelude::Replication;
use petri_shared::{get_player_capsule_size, Appearance, ReplicatedPos, ReplicationBundle, Tint};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_monster);
    }
}

#[derive(Component)]
struct Monster;

fn spawn_monster(mut command: Commands) {
    let (capsule_diameter, capsule_segment_half_height) = get_player_capsule_size();

    command.spawn((
        Name::new("Monster"),
        Monster,
        Collider::capsule_y(capsule_segment_half_height, capsule_diameter / 2.0),
        RigidBody::Dynamic,
        LockedAxes::ROTATION_LOCKED,
        TransformBundle {
            local: Transform::from_xyz(0.0, 10.0, 0.0),
            ..default()
        },
        ReplicationBundle::new(Tint(Color::PINK), Appearance::Box),
    ));
}
