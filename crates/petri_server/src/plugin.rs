use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_rapier3d::prelude::*;
use bevy_replicon::{
    prelude::*,
    renet::{
        transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
        ConnectionConfig, ServerEvent,
    },
};
use petri_shared::{MoveDirection, Player, PlayerColor, PlayerName, PlayerPos, SetName};
use rand::random;
use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

pub struct PetriServerPlugin;

impl Plugin for PetriServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (setup_physics, setup_server_networking.map(Result::unwrap)),
        )
        .add_systems(
            Update,
            (
                server_event_system,
                receive_names,
                move_clients.after(ServerSet::Receive),
            ),
        )
        // FIXME(opt): make sure `Update` schedule is running the same frequency as the server sends event
        .add_systems(Update, update_player_pos.before(ServerSet::Send))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default());

        fn receive_names(
            mut events: EventReader<FromClient<SetName>>,
            mut clients: Query<(Entity, &Player), Without<PlayerName>>,
            mut commands: Commands,
        ) {
            // FIXME: get entity by client id
            for event in events.read() {
                info!("Received name from {:?} {:?}", event.client_id, event.event);
                for (entity, Player(client_id)) in clients.iter_mut() {
                    if client_id == &event.client_id {
                        commands
                            .entity(entity)
                            .insert(PlayerName(event.event.0.clone()));
                    }
                }
            }
        }

        /// Logs server events and spawns a new player whenever a client connects.
        fn server_event_system(
            mut commands: Commands,
            mut server_event: EventReader<ServerEvent>,
            clients: Query<(Entity, &Player)>,
        ) {
            for event in server_event.read() {
                match event {
                    ServerEvent::ClientConnected { client_id } => {
                        info!("client: {client_id} Connected");
                        // Generate pseudo random color from client id.
                        let r = ((client_id.raw() % 23) as f32) / 23.0;
                        let g = ((client_id.raw() % 27) as f32) / 27.0;
                        let b = ((client_id.raw() % 39) as f32) / 39.0;
                        commands.spawn((
                            Player(*client_id),
                            TransformBundle::from_transform(Transform::from_xyz(
                                random::<f32>() * 3.0 + 1.5,
                                2.5,
                                random::<f32>() * 3.0 + 1.5,
                            )),
                            PlayerPos(default()),
                            ExternalImpulse::default(),
                            PlayerColor(Color::rgb(r, g, b)),
                            Replication,
                            Collider::cuboid(0.5, 0.5, 0.5),
                            RigidBody::Dynamic,
                        ));
                    }
                    ServerEvent::ClientDisconnected { client_id, reason } => {
                        info!("client {client_id} disconnected: {reason}");
                        for (e, p) in clients.iter() {
                            if &p.0 == client_id {
                                commands.entity(e).despawn_recursive();
                            }
                        }
                    }
                }
            }
        }

        fn update_player_pos(mut players: Query<(&GlobalTransform, &mut PlayerPos)>) {
            players.iter_mut().for_each(|(local_pos, mut shared_pos)| {
                shared_pos.0 = *local_pos;
            })
        }

        fn setup_server_networking(
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
            info!("Starting server on {public_addr:?}");
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
    }
}

fn setup_physics(mut commands: Commands) {
    //floor
    commands.spawn(Collider::cuboid(1000.0, 0.1, 1000.0));
}

fn move_clients(
    mut events: EventReader<FromClient<MoveDirection>>,
    mut player: Query<(&Player, &mut ExternalImpulse)>,
) {
    // FIXME: make this a map of client_id to entity and update it once per [Update]
    let mut player_to_ext_force: HashMap<_, _> = player
        .iter_mut()
        .map(|(Player(client_id), force)| (client_id, force))
        .collect();

    for event in events.read() {
        info!("Received move event");
        const KONSTANTA: f32 = 0.1;
        if let Some(force) = player_to_ext_force.get_mut(&event.client_id) {
            let normalized = event.event.0.normalize_or_zero();
            force.impulse = Vec3 {
                x: normalized.x,
                y: 0.0,
                // N.B.
                z: normalized.y,
            } * KONSTANTA;
        } else {
            info!("POLTERGEIST IS MOVING");
        }
    }
}
