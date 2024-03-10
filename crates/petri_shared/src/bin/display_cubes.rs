//! This example demonstrates how to create a custom mesh,
//! assign a custom UV mapping for a custom texture,
//! and how to change the UV mapping at run-time.

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
};
use itertools::iproduct;
use petri_shared::terrain::{sample_terrain, TerrainData};

// Define a "marker" component to mark the custom mesh. Marker components are often used in Bevy for
// filtering entities in queries with With, they're usually not queried directly since they don't contain information within them.
#[derive(Component)]
struct CustomUV;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, input_handler)
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let cells = sample_terrain();
    for (x, y, z) in iproduct!(-2..=2, -1..=1, -2..=2) {
        // Create and save a handle to the mesh.
        let cube_mesh_handle: Handle<Mesh> = meshes.add(create_cube_mesh(&cells[&(x, y, z)]));

        // Render the mesh with the custom texture using a PbrBundle, add the marker.
        commands.spawn((
            PbrBundle {
                mesh: cube_mesh_handle,
                material: materials.add(Color::DARK_GREEN),
                transform: Transform::from_xyz(x as f32 * 10.0, y as f32 * 10.0, z as f32 * 10.0),
                ..default()
            },
            CustomUV,
        ));
    }

    // Transform for the camera and lighting, looking at (0,0,0) (the position of the mesh).
    let camera_and_light_transform =
        Transform::from_xyz(20.0, 20.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y);

    // Camera in 3D space.
    commands.spawn(Camera3dBundle {
        transform: camera_and_light_transform,
        ..default()
    });

    // Light up the scene.
    commands.spawn(PointLightBundle {
        transform: camera_and_light_transform,
        ..default()
    });

    // Text to describe the controls.
    commands.spawn(
        TextBundle::from_section(
            "Controls:\nX/Y/Z: Rotate\nR: Reset orientation",
            TextStyle {
                font_size: 20.0,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        }),
    );
}

// System to receive input from the user,
// check out examples/input/ for more examples about user input.
fn input_handler(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<&mut Transform, With<CustomUV>>,
    time: Res<Time>,
) {
    if keyboard_input.pressed(KeyCode::KeyX) {
        for mut transform in &mut query {
            transform.rotate_x(time.delta_seconds() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::KeyY) {
        for mut transform in &mut query {
            transform.rotate_y(time.delta_seconds() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::KeyZ) {
        for mut transform in &mut query {
            transform.rotate_z(time.delta_seconds() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::KeyR) {
        for mut transform in &mut query {
            transform.look_to(Vec3::NEG_Z, Vec3::Y);
        }
    }
}

fn create_cube_mesh(cell: &TerrainData) -> Mesh {
    let mesh = cell.get_polygons();
    let vertex_count = mesh.positions.len();

    // Keep the mesh data accessible in future frames to be able to mutate it in toggle_texture.
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        mesh.positions
            .chunks_exact(3)
            .map(|c| c.try_into().unwrap())
            .collect::<Vec<[f32; 3]>>(),
    )
    // TODO UV
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; vertex_count])
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        mesh.normals
            .chunks_exact(3)
            .map(|n| n.try_into().unwrap())
            .collect::<Vec<[f32; 3]>>(),
    )
    .with_inserted_indices(Indices::U32(
        mesh.triangle_indices.iter().map(|v| *v as u32).collect(),
    ))
}
