use bevy::prelude::*;
use bevy_replicon::{
    prelude::{NetworkChannels, RenetClient},
    renet::{
        transport::{ClientAuthentication, NetcodeClientTransport},
        ConnectionConfig,
    },
};
use petri_shared::{Player, PlayerColor, PlayerPos};
use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

pub struct PetriClientPlugin;

impl Plugin for PetriClientPlugin {
    fn build(&self, app: &mut App) {
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
                ..Default::default()
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

        app.add_systems(Startup, (setup_scene, setup_connection.map(Result::unwrap)))
            .add_systems(Update, (add_mesh_to_players, move_player_from_network));
    }
}
