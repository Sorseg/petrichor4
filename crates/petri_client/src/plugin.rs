use bevy::ecs::query::QueryEntityError;
use bevy::{input::keyboard::KeyboardInput, prelude::*};
use bevy_replicon::{
    client_just_connected,
    prelude::{NetworkChannels, RenetClient},
    renet::{
        transport::{ClientAuthentication, NetcodeClientTransport},
        ConnectionConfig,
    },
};
use petri_shared::{Player, PlayerColor, PlayerName, PlayerPos, SetName};
use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    time::{Duration, SystemTime},
};

pub struct PetriClientPlugin;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PetriState {
    #[default]
    Login,
    Scene,
}

#[derive(Resource, Debug)]
pub struct CurrentUserLogin(String);

impl Plugin for PetriClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<PetriState>()
            .insert_resource(CurrentUserLogin(String::new()))
            .add_systems(OnEnter(PetriState::Login), login_screen)
            .add_systems(Update, login_input.run_if(in_state(PetriState::Login)))
            .add_systems(
                OnExit(PetriState::Login),
                |mut cmd: Commands, login_entity: Query<Entity, With<LoginUIMarker>>| {
                    login_entity.for_each(|l| cmd.entity(l).despawn_recursive())
                },
            )
            .add_systems(
                OnEnter(PetriState::Scene),
                (setup_scene, setup_connection.map(Result::unwrap)),
            )
            .add_systems(
                Update,
                (
                    send_name.run_if(client_just_connected()),
                    add_mesh_to_players,
                    move_player_from_network,
                    log_players,
                    hud_update_player_names,
                )
                    .run_if(in_state(PetriState::Scene)),
            )
            .add_systems(OnExit(PetriState::Scene), || todo!("Clean up world"));

        #[derive(Debug, Component)]
        struct LoginInput;

        #[derive(Debug, Component)]
        struct LoginUIMarker;

        fn login_screen(mut cmd: Commands, asset_server: Res<AssetServer>) {
            cmd.spawn((Camera2dBundle::default(), LoginUIMarker));
            let root = cmd
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        background_color: BackgroundColor(Color::DARK_GRAY),
                        ..default()
                    },
                    LoginUIMarker,
                ))
                .id();

            let prompt = cmd
                .spawn(
                    // Create a TextBundle that has a Text with a single section.
                    TextBundle::from_section(
                        // Accepts a `String` or any type that converts into a `String`, such as `&str`
                        "Login",
                        TextStyle {
                            // This font is loaded and will be used instead of the default font.
                            font: asset_server.load("open-sans.ttf"),
                            font_size: 100.0,
                            ..default()
                        },
                    ) // Set the alignment of the Text
                    .with_text_alignment(TextAlignment::Center), // Set the style of the TextBundle itself.
                )
                .id();

            let login_container = cmd
                .spawn((
                    NodeBundle {
                        background_color: BackgroundColor(Color::DARK_GREEN),
                        ..default()
                    },
                    Outline {
                        width: Val::Px(6.),
                        offset: Val::Px(6.),
                        color: Color::LIME_GREEN,
                    },
                ))
                .id();

            let input = cmd
                .spawn((
                    // Create a TextBundle that has a Text with a single section.
                    TextBundle::from_section(
                        // Accepts a `String` or any type that converts into a `String`, such as `&str`
                        "_",
                        TextStyle {
                            // This font is loaded and will be used instead of the default font.
                            font: asset_server.load("open-sans.ttf"),
                            font_size: 100.0,
                            ..default()
                        },
                    ) // Set the alignment of the Text
                    .with_text_alignment(TextAlignment::Center), // Set the style of the TextBundle itself.
                    LoginInput,
                ))
                .id();

            cmd.entity(login_container).add_child(input);
            cmd.entity(root)
                .add_child(prompt)
                .add_child(login_container);
        }

        fn login_input(
            mut char_input_events: EventReader<ReceivedCharacter>,
            mut keyboard_input_events: EventReader<KeyboardInput>,
            mut login_label: Query<&mut Text, With<LoginInput>>,
            mut login: ResMut<CurrentUserLogin>,
            mut next_state: ResMut<NextState<PetriState>>,
        ) {
            for event in keyboard_input_events.read() {
                match event.key_code {
                    Some(KeyCode::Return) => {
                        if !login.0.trim().is_empty() {
                            next_state.set(PetriState::Scene);
                            return;
                        }
                    }
                    Some(KeyCode::Back) => {
                        login.0.pop();
                    }
                    _ => {}
                }
            }
            for event in char_input_events.read() {
                login.0.push(event.char);
            }
            login_label.for_each_mut(|mut l| {
                l.sections[0].value = format!("{}_", login.0);
            })
        }

        #[derive(Component)]
        struct PlayerHydrated;

        fn add_mesh_to_players(
            mut commands: Commands,
            mut meshes: ResMut<Assets<Mesh>>,
            mut materials: ResMut<Assets<StandardMaterial>>,
            players_without_mesh: Query<(Entity, &Player, &PlayerColor), Without<PlayerHydrated>>,
        ) {
            for (entity, player, player_color) in players_without_mesh.iter() {
                info!("Adding mesh to {player:?}");
                commands
                    .entity(entity)
                    .insert(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                        material: materials.add(player_color.0.into()),
                        transform: Transform::from_xyz(0.0, 0.5, 0.0),
                        ..default()
                    })
                    .insert(PlayerHydrated);
            }
        }

        fn move_player_from_network(mut players: Query<(&mut Transform, &PlayerPos)>) {
            for (mut t, p) in &mut players {
                t.translation = p.0;
            }
        }

        /// set up a simple 3D scene
        fn setup_scene(
            mut commands: Commands,
            mut meshes: ResMut<Assets<Mesh>>,
            mut materials: ResMut<Assets<StandardMaterial>>,
        ) {
            // circular base
            commands.spawn(PbrBundle {
                mesh: meshes.add(shape::Circle::new(4.0).into()),
                material: materials.add(Color::WHITE.into()),
                transform: Transform::from_rotation(Quat::from_rotation_x(
                    -std::f32::consts::FRAC_PI_2,
                )),
                ..default()
            });
            // light
            commands.spawn(PointLightBundle {
                point_light: PointLight {
                    intensity: 1500.0,
                    shadows_enabled: true,
                    ..default()
                },
                transform: Transform::from_xyz(4.0, 8.0, 4.0),
                ..default()
            });
            // camera
            commands.spawn(Camera3dBundle {
                transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            });
        }

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
            let server_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8989);
            let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0))?;
            let authentication = ClientAuthentication::Unsecure {
                client_id,
                protocol_id: 0,
                server_addr,
                user_data: None,
            };
            let transport = NetcodeClientTransport::new(current_time, authentication, socket)?;

            commands.insert_resource(client);
            commands.insert_resource(transport);
            Ok(())
        }

        fn send_name(mut set_name: EventWriter<SetName>, login: Res<CurrentUserLogin>) {
            set_name.send(SetName(login.0.clone()));
        }

        fn log_players(time: Res<Time>, mut timer: Local<Timer>, players: Query<&PlayerName>) {
            if timer.tick(time.delta()).just_finished() {
                timer.set_duration(Duration::from_secs(1));
                timer.reset();
                for player in &players {
                    info!("{player:?}")
                }
            }
        }

        #[derive(Component)]
        struct PlayerNameLabel(Entity);

        // FIXME: this doesn't clean up labels on disconnects
        fn hud_update_player_names(
            mut commands: Commands,
            players: Query<(Entity, &PlayerName, &GlobalTransform)>,
            mut labels: Query<&mut PlayerNameLabel>,
            mut styles: Query<&mut Style>,
            asset_server: Res<AssetServer>,
            camera: Query<(&Camera, &GlobalTransform)>
        ) {
            let (camera, camera_transform) = camera.single();
            for (player_entity, name, player_transform) in &players {
                // FIXME: update and create in a single step
                match labels.get_mut(player_entity) {
                    Ok((label)) => {
                        let pos = camera.world_to_viewport(camera_transform, player_transform.translation());
                        if let Some(p) = pos {
                            let mut style = styles.get_mut(label.0).unwrap();
                            style.left = Val::Px(p.x);
                            style.top = Val::Px(p.y);
                        }
                    },
                    Err(QueryEntityError::QueryDoesNotMatch(..)) => {
                        info!("Creating label for {name:?}");
                        let node = commands.spawn(TextBundle {
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
                        }).id();
                        commands.entity(player_entity).insert(PlayerNameLabel(node));
                    }
                    Err(e) => panic!("{e:?}"),
                };
            }
        }

        fn cleanup_despawned_name_plaques() {

        }
    }
}
