use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_replicon::{
    prelude::*,
    renet::{
        transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
        ClientId, ConnectionConfig, ServerEvent,
    },
};
use petri_shared::{MoveDirection, Player, PlayerColor, PlayerPos};
use serde::{Deserialize, Serialize};
use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            LogPlugin::default(),
            ReplicationPlugins
                .build()
                .disable::<ClientPlugin>()
                .set(ServerPlugin {
                    tick_policy: TickPolicy::MaxTickRate(60),
                    ..Default::default()
                }),
        ))
        .add_systems(Update, server_event_system)
        .add_systems(Startup, setup_server.map(Result::unwrap))
        .add_client_event::<MoveDirection>(EventType::Ordered)
        .run();
}

/// Logs server events and spawns a new player whenever a client connects.
fn server_event_system(mut commands: Commands, mut server_event: EventReader<ServerEvent>) {
    for event in server_event.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                info!("player: {client_id} Connected");
                // Generate pseudo random color from client id.
                let r = ((client_id.raw() % 23) as f32) / 23.0;
                let g = ((client_id.raw() % 27) as f32) / 27.0;
                let b = ((client_id.raw() % 39) as f32) / 39.0;
                commands.spawn((
                    Player(*client_id),
                    PlayerPos(Vec3::ZERO),
                    PlayerColor(Color::rgb(r, g, b)),
                    Replication,
                ));
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("client {client_id} disconnected: {reason}");
                // TODO: despawn
            }
        }
    }
}

fn setup_server(
    mut commands: Commands,
    network_channels: Res<NetworkChannels>,
) -> anyhow::Result<()> {
    let server_channels_config = network_channels.get_server_configs();
    let client_channels_config = network_channels.get_client_configs();

    let server = RenetServer::new(ConnectionConfig {
        server_channels_config,
        client_channels_config,
        ..Default::default()
    });

    let port = 8989;
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    let public_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), port);
    let socket = UdpSocket::bind(public_addr)?;
    let server_config = ServerConfig {
        current_time,
        max_clients: 10,
        protocol_id: 0,
        authentication: ServerAuthentication::Unsecure,
        public_addresses: vec![public_addr],
    };
    let transport = NetcodeServerTransport::new(server_config, socket)?;

    commands.insert_resource(server);
    commands.insert_resource(transport);
    Ok(())
}
