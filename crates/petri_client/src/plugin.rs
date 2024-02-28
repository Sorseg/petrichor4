use bevy::{
    core_pipeline::Skybox,
    ecs::query::QueryEntityError,
    input::{keyboard::KeyboardInput, mouse::MouseMotion},
    prelude::*,
    window::CursorGrabMode,
};
use bevy_replicon::{
    client_just_connected,
    prelude::{NetworkChannels, RenetClient},
    renet::{
        transport::{ClientAuthentication, NetcodeClientTransport},
        ConnectionConfig,
    },
};
use petri_shared::{
    get_player_capsule_size, MoveDirection, Player, PlayerColor, PlayerName, PlayerPos, SetName,
};
use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};
use crate::login_plugin::{CurrentUserLogin, LoginPlugin};

pub struct PetriClientPlugin;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PetriState {
    Login,
    Scene,
}

impl Plugin for PetriClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(PetriState::Login)
            .add_plugins(LoginPlugin)
            .add_systems(
                OnEnter(PetriState::Scene),
                (setup_scene, setup_connection.map(Result::unwrap)),
            )
            .add_systems(
                Update,
                (
                    grab_mouse,
                    send_name.run_if(client_just_connected),
                    (aim, hud_update_player_names, send_movement)
                        .run_if(any_with_component::<Eyes>),
                    hydrate_players,
                    move_player_from_network,
                )
                    .run_if(in_state(PetriState::Scene)),
            )
            .add_systems(OnExit(PetriState::Scene), || todo!("Clean up world"));

        fn hydrate_players(
            mut commands: Commands,
            mut meshes: ResMut<Assets<Mesh>>,
            mut materials: ResMut<Assets<StandardMaterial>>,
            players_without_mesh: Query<(Entity, &Player, &PlayerColor), Added<Player>>,
            my_player_id: Res<MyPlayerId>,
            asset_server: Res<AssetServer>,
        ) {
            for (entity, player, player_color) in players_without_mesh.iter() {
                info!("Adding mesh to {player:?}");
                let (capsule_diameter, capsule_segment_half_height) = get_player_capsule_size();
                let mut entity = commands.entity(entity);
                if player.0.raw() == my_player_id.0 {
                    entity.insert((Me, TransformBundle::default()));
                    let height = 1.0;
                    entity.with_children(|parent| {
                        parent.spawn(
                            // camera
                            (
                                Eyes,
                                Camera3dBundle {
                                    transform: Transform::from_xyz(0.0, height, 0.0).looking_at(
                                        // TODO: replace with zero, will be rewritten by the aiming system anyway
                                        Vec3 {
                                            x: 0.0,
                                            y: height,
                                            z: 10.0,
                                        },
                                        Vec3::Y,
                                    ),
                                    ..default()
                                },
                                Skybox {
                                    image: asset_server.load("specular.ktx2"),
                                    brightness: 150.0,
                                },
                                EnvironmentMapLight {
                                    specular_map: asset_server.load("specular.ktx2"),
                                    diffuse_map: asset_server.load("diffuse.ktx2"),
                                    intensity: 150.0,
                                },
                            ),
                        );
                    });
                } else {
                    entity.insert(PbrBundle {
                        mesh: meshes.add(Capsule3d::new(
                            capsule_diameter / 2.0,
                            capsule_segment_half_height * 2.0,
                        )),
                        material: materials.add(player_color.0),
                        transform: Transform::from_xyz(0.0, 0.5, 0.0),
                        ..default()
                    });
                }
            }
        }

        fn move_player_from_network(mut players: Query<(&mut Transform, &PlayerPos)>) {
            for (mut t, p) in &mut players {
                *t = p.0.into();
            }
        }

        fn aim(
            mut mouse_motion_events: EventReader<MouseMotion>,
            mut eyes: Query<&mut Transform, With<Eyes>>,
            mut windows: Query<&mut Window>,
        ) {
            // only aim when cursor is grabbed
            if windows.single().cursor.visible {
                return;
            }
            let sensitivity = 0.001;

            let mut transform = eyes.single_mut();
            let delta = mouse_motion_events.read().map(|e| e.delta).sum::<Vec2>();
            // FIXME: limit X turn angle
            transform.rotate_y(-delta.x * sensitivity);
            transform.rotate_local_x(-delta.y * sensitivity);
        }

        /// load the 3d scene
        fn setup_scene(
            mut commands: Commands,
            mut meshes: ResMut<Assets<Mesh>>,
            mut materials: ResMut<Assets<StandardMaterial>>,
            asset_server: Res<AssetServer>,
        ) {
            commands.spawn(SceneBundle {
                scene: asset_server.load("petrichor4-intro.glb#Scene0"),
                ..default()
            });
            // circular base
            commands.spawn(PbrBundle {
                mesh: meshes.add(Circle::new(4.0)),
                material: materials.add(Color::WHITE),
                transform: Transform::from_rotation(Quat::from_rotation_x(
                    -std::f32::consts::FRAC_PI_2,
                )),
                ..default()
            });
        }

        // Player id of the player who is playing this instance of the game
        #[derive(Resource)]
        struct MyPlayerId(u64);

        fn setup_connection(
            mut commands: Commands,
            network_channels: Res<NetworkChannels>,
        ) -> anyhow::Result<()> {
            let server_channels_config = network_channels.get_server_configs();
            let client_channels_config = network_channels.get_client_configs();

            let client = RenetClient::new(ConnectionConfig {
                server_channels_config,
                client_channels_config,
                ..default()
            });

            let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
            let client_id = current_time.as_millis() as u64;

            let addr = std::env::args()
                .nth(1)
                .map(|v| dns_lookup::lookup_host(&v).unwrap()[0])
                .unwrap_or(Ipv4Addr::LOCALHOST.into());
            let server_addr = SocketAddr::new(addr, 8989);
            info!("Connecting to {server_addr:?}...");

            let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
            let authentication = ClientAuthentication::Unsecure {
                client_id,
                protocol_id: 0,
                server_addr,
                user_data: None,
            };

            let transport = NetcodeClientTransport::new(current_time, authentication, socket)?;

            commands.insert_resource(MyPlayerId(client_id));
            commands.insert_resource(client);
            commands.insert_resource(transport);

            Ok(())
        }

        fn send_name(mut set_name: EventWriter<SetName>, login: Res<CurrentUserLogin>) {
            info!("sending my name {:?}", login.0);
            set_name.send(SetName(login.0.clone()));
        }

        #[derive(Component)]
        struct PlayerNameLabel(Entity);

        // FIXME: this doesn't clean up labels on disconnects
        fn hud_update_player_names(
            mut commands: Commands,
            players: Query<(Entity, &PlayerName, &GlobalTransform), Without<Me>>,
            mut labels: Query<&mut PlayerNameLabel>,
            mut styles: Query<&mut Style>,
            asset_server: Res<AssetServer>,
            camera: Query<(&Camera, &GlobalTransform), With<Eyes>>,
        ) {
            let (camera, camera_transform) = camera.single();
            for (player_entity, name, player_transform) in &players {
                // FIXME: update and create in a single step
                match labels.get_mut(player_entity) {
                    Ok((label)) => {
                        let pos = camera
                            .world_to_viewport(camera_transform, player_transform.translation());
                        if let Some(p) = pos {
                            let mut style = styles.get_mut(label.0).unwrap();
                            style.left = Val::Px(p.x);
                            style.top = Val::Px(p.y);
                        }
                    }
                    Err(QueryEntityError::QueryDoesNotMatch(..)) => {
                        info!("Creating label for {name:?}");
                        let node = commands
                            .spawn(TextBundle {
                                text: Text::from_section(
                                    &name.0,
                                    TextStyle {
                                        font: asset_server.load("open-sans.ttf"),
                                        font_size: 10.0,
                                        color: Color::WHITE,
                                    },
                                ),
                                style: Style {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(10.0),
                                    bottom: Val::Px(10.0),
                                    ..default()
                                },
                                ..default()
                            })
                            .id();
                        commands.entity(player_entity).insert(PlayerNameLabel(node));
                    }
                    Err(e) => panic!("{e:?}"),
                };
            }
        }

        fn cleanup_despawned_name_plaques() {}
    }
}


/// Marks the entity with the camera that represents players eyes
#[derive(Component)]
struct Eyes;

/// Marks the entity that represents the player
#[derive(Component)]
struct Me;

fn send_movement(
    mut writer: EventWriter<MoveDirection>,
    input: Res<ButtonInput<KeyCode>>,
    eyes: Query<&GlobalTransform, With<Eyes>>,
) {
    let pos = eyes.single();
    let forward = pos.forward();

    let mut direction = Vec2::default();
    // +Y is right
    // +X is forward
    static KEYBINDINGS: &[(KeyCode, Vec2)] = &[
        (KeyCode::KeyA, Vec2::new(0.0, -1.0)),
        (KeyCode::KeyD, Vec2::new(0.0, 1.0)),
        (KeyCode::KeyW, Vec2::new(1.0, 0.0)),
        (KeyCode::KeyS, Vec2::new(-1.0, 0.0)),
    ];

    for (key, dir) in KEYBINDINGS {
        if input.pressed(*key) {
            direction += *dir;
        }
    }

    let rotated = direction.rotate(Vec2 {
        x: forward.x,
        y: forward.z,
    });

    if direction.length() != 0.0 {
        writer.send(MoveDirection(rotated));
    }
}

/// This system grabs the mouse when the left mouse button is pressed
/// and releases it when the escape key is pressed
fn grab_mouse(
    mut windows: Query<&mut Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    let mut window = windows.single_mut();

    if mouse.just_pressed(MouseButton::Left) {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
    }

    if key.just_pressed(KeyCode::Escape) {
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
    }
}
