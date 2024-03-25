//! Client app

mod login_plugin;
mod plugin;

use bevy::prelude::*;
use bevy_replicon::{server::ServerPlugin, ReplicationPlugins};
use petri_shared::PetriReplicationSetupPlugin;

use crate::plugin::PetriClientPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        title: "Petrichor IV".into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    // TODO: add auto-processing
                    mode: AssetMode::Unprocessed,

                    file_path: "../../asset_sources".into(),
                    ..default()
                }),
            ReplicationPlugins.build().disable::<ServerPlugin>(),
            PetriReplicationSetupPlugin,
            PetriClientPlugin,
        ))
        .run();
}
