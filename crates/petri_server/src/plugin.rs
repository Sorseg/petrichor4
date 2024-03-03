use std::{
    io::Cursor,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

use bevy::{prelude::*, utils::HashMap};
use bevy_rapier3d::prelude::*;
use bevy_replicon::{
    prelude::*,
    renet::{
        transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
        ClientId, ConnectionConfig, ServerEvent,
    },
};
use obj::{load_obj, Obj, Position};
use petri_shared::{
    get_player_capsule_size, AdminCommand, Aim, Appearance, MoveDirection, Player, ReplicatedAim,
    ReplicatedPos, ReplicationBundle, SetName, Tint,
};
use rand::random;

use crate::{
    blob_assets::{Blob, BlobLoaderPlugin},
    enemy::EnemyPlugin,
};

pub struct PetriServerPlugin;

impl Plugin for PetriServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BlobLoaderPlugin)
            .add_plugins(EnemyPlugin)
            .init_resource::<ObjFileWithColliderHandle>()
            .init_resource::<PlayerMap>()
            .add_systems(
                Startup,
                (load_collider, setup_server_networking.map(Result::unwrap)),
            )
            .add_systems(
                Update,
                (
                    server_event_system,
                    receive_names,
                    load_collider_from_mesh,
                    move_clients,
                    apply_aim,
                    update_player_pos,
                    handle_admin_commands,
                    kill_y,
                ),
            )
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::default());

        fn receive_names(
            mut events: EventReader<FromClient<SetName>>,
            mut clients: Query<(Entity, &Player), Without<Name>>,
            mut commands: Commands,
        ) {
            // FIXME: get entity by client id
            for event in events.read() {
                info!("Received name from {:?} {:?}", event.client_id, event.event);
                for (entity, Player(client_id)) in clients.iter_mut() {
                    if client_id == &event.client_id {
                        commands
                            .entity(entity)
                            .insert(Name::new(event.event.0.clone()));
                    }
                }
            }
        }

        /// Logs server events and spawns a new player whenever a client connects.
        fn server_event_system(
            mut commands: Commands,
            mut server_event: EventReader<ServerEvent>,
            mut player_map: ResMut<PlayerMap>,
        ) {
            for event in server_event.read() {
                match event {
                    ServerEvent::ClientConnected { client_id } => {
                        info!("client: {client_id} Connected");
                        // Generate pseudo random color from client id.
                        let r = ((client_id.raw() % 23) as f32) / 23.0;
                        let g = ((client_id.raw() % 27) as f32) / 27.0;
                        let b = ((client_id.raw() % 39) as f32) / 39.0;

                        let (capsule_diameter, capsule_segment_half_height) =
                            get_player_capsule_size();

                        let entity = commands
                            .spawn((
                                Player(*client_id),
                                // FIXME: Players are Admins by default
                                Admin,
                                ReplicationBundle::new(
                                    Tint(Color::rgb(r, g, b)),
                                    Appearance::Capsule,
                                ),
                                PhysicsBundle {
                                    collider: Collider::capsule_y(
                                        capsule_segment_half_height,
                                        capsule_diameter / 2.0,
                                    ),
                                    trans: TransformBundle::from_transform(Transform::from_xyz(
                                        random::<f32>() * 3.0 + 1.5,
                                        2.5,
                                        random::<f32>() * 3.0 + 1.5,
                                    )),
                                    ..default()
                                },
                                LockedAxes::ROTATION_LOCKED,
                                // FIXME: replace with friction
                                Damping {
                                    linear_damping: 0.5,
                                    angular_damping: 0.0,
                                },
                            ))
                            .id();
                        player_map.0.insert(*client_id, entity);
                    }
                    ServerEvent::ClientDisconnected { client_id, reason } => {
                        info!("client {client_id} disconnected: {reason}");
                        commands
                            .entity(player_map.0.remove(client_id).expect(
                                "Disconnect event should only trigger for connected clients",
                            ))
                            .despawn_recursive();
                    }
                }
            }
        }

        fn update_player_pos(
            mut players: Query<(&GlobalTransform, &mut ReplicatedPos), Changed<GlobalTransform>>,
        ) {
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
            let hosted_on_fly = std::env::args().any(|a| a == "--flyio");

            let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
            // fly.io requires UDP apps to bind to a specific address
            // https://fly.io/docs/networking/udp-and-tcp/
            let ip = if hosted_on_fly {
                dns_lookup::lookup_host("fly-global-services")
                    .unwrap()
                    .into_iter()
                    .find(IpAddr::is_ipv4)
                    .unwrap()
            } else {
                Ipv4Addr::LOCALHOST.into()
            };
            let socket_address = SocketAddr::new(ip, port);
            info!("Starting server on {socket_address:?}");
            let socket = UdpSocket::bind(socket_address)?;
            let server_config = ServerConfig {
                current_time,
                max_clients: 10,
                protocol_id: 0,
                authentication: ServerAuthentication::Unsecure,
                public_addresses: vec![socket_address],
            };
            let transport = NetcodeServerTransport::new(server_config, socket)?;

            commands.insert_resource(server);
            commands.insert_resource(transport);
            Ok(())
        }
    }
}

#[derive(Resource, Default, Debug)]
struct PlayerMap(HashMap<ClientId, Entity>);

// TODO: is it ok to create default handle?
#[derive(Resource, Default)]
struct ObjFileWithColliderHandle(Handle<Blob>);

fn load_collider_from_mesh(
    mut commands: Commands,
    mut ev_asset: EventReader<AssetEvent<Blob>>,
    blob: Res<ObjFileWithColliderHandle>,
    blobs: Res<Assets<Blob>>,
    mut loaded: Local<bool>,
) {
    if *loaded {
        return;
    }
    // FIXME: move all of this to a new "ColliderAssetLoader"
    if ev_asset.read().next().is_some() {
        if let Some(Blob(bytes)) = blobs.get(&blob.0) {
            info!("Gotmesh? {:x?}", &bytes[0..10]);
            let obj: Obj<Position> = load_obj(Cursor::new(bytes)).unwrap();
            commands.spawn(Collider::trimesh(
                obj.vertices
                    .into_iter()
                    .map(|v| Vec3 {
                        x: v.position[0],
                        y: v.position[1],
                        z: v.position[2],
                    })
                    .collect(),
                obj.indices
                    .chunks(3)
                    .map(|c| [c[0] as u32, c[1] as u32, c[2] as u32])
                    .collect(),
            ));
            *loaded = true;
        }
    }
}

fn load_collider(asset_server: Res<AssetServer>, mut blob: ResMut<ObjFileWithColliderHandle>) {
    blob.0 = asset_server.load("level_collider.obj");
}

fn move_clients(
    mut events: EventReader<FromClient<MoveDirection>>,
    mut player: Query<(&Player, &mut ExternalImpulse, &ReadMassProperties)>,
) {
    // FIXME: make this a map of client_id to entity and update it once per [Update]
    let mut player_to_ext_force: HashMap<_, _> = player
        .iter_mut()
        .map(|(Player(client_id), force, props)| (client_id, (force, props)))
        .collect();

    for event in events.read() {
        const KONSTANTA: f32 = 0.1;

        if let Some((force, props)) = player_to_ext_force.get_mut(&event.client_id) {
            let normalized = event.event.0.normalize_or_zero();
            force.impulse = Vec3 {
                x: normalized.x,
                y: 0.0,
                // N.B.
                z: normalized.y,
            } * KONSTANTA
                * props.mass;
        } else {
            error!("POLTERGEIST IS MOVING");
        }
    }
}

fn apply_aim(
    mut events: EventReader<FromClient<Aim>>,
    mut player: Query<&mut ReplicatedAim>,
    map: Res<PlayerMap>,
) {
    for e in events.read() {
        let Some(entity) = map.0.get(&e.client_id) else {
            error!("Unknown client id {}", e.client_id);
            continue;
        };
        let Ok(mut aim) = player.get_mut(*entity) else {
            error!("Player does not have aim component {}", e.client_id);
            continue;
        };
        aim.0 = e.event.0;
    }
}

#[derive(Bundle, Default)]
pub struct PhysicsBundle {
    pub impulse: ExternalImpulse,
    pub collider: Collider,
    pub mass_props: ReadMassProperties,
    pub rigid_body: RigidBody,
    pub trans: TransformBundle,
}

#[derive(Component)]
pub struct Admin;

fn handle_admin_commands(
    mut commands: Commands,
    mut admin_commands: EventReader<FromClient<AdminCommand>>,
) {
    for command in admin_commands.read() {
        match command.event {
            AdminCommand::SpawnBoxWall { side_size, at } => {
                for xi in 0..side_size {
                    for yi in 0..side_size {
                        commands.spawn((
                            PhysicsBundle {
                                collider: Collider::cuboid(0.5, 0.5, 0.5),
                                trans: TransformBundle::from_transform(Transform::from_xyz(
                                    at.x + xi as f32,
                                    at.y + yi as f32,
                                    at.z,
                                )),
                                // FIXME: boxes probably do not need mass props
                                ..default()
                            },
                            // FIXME: boxes don't have aim
                            ReplicationBundle::new(Tint(Color::GREEN), Appearance::Box),
                        ));
                    }
                }
            }
        }
    }
}

fn kill_y(mut commands: Commands, query: Query<(Entity, &GlobalTransform)>) {
    for (e, t) in query.iter() {
        if t.translation().y < -1000.0 {
            commands.entity(e).despawn_recursive();
        }
    }
}
