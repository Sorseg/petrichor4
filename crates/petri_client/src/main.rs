//! Client app

mod plugin;

use crate::plugin::PetriClientPlugin;
use bevy::prelude::*;
use bevy_replicon::{server::ServerPlugin, ReplicationPlugins};
use petri_shared::PetriSharedSetup;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ReplicationPlugins.build().disable::<ServerPlugin>(),
            PetriSharedSetup,
            PetriClientPlugin,
        ))
        .run();
}
