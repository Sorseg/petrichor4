//! Client app

mod login_plugin;
mod plugin;

use crate::plugin::PetriClientPlugin;
use bevy::prelude::*;
use bevy_replicon::{server::ServerPlugin, ReplicationPlugins};
use petri_shared::PetriReplicationSetupPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: bevy::window::PresentMode::AutoNoVsync,
                    title: "Petrichor IV".into(),
                    ..default()
                }),
                ..default()
            }),
            ReplicationPlugins.build().disable::<ServerPlugin>(),
            PetriReplicationSetupPlugin,
            PetriClientPlugin,
        ))
        .run();
}
