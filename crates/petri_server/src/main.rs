mod plugin;

use crate::plugin::PetriServerPlugin;
use bevy::{log::LogPlugin, prelude::*};
use bevy_replicon::prelude::*;
use petri_shared::PetriSharedSetup;

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
            PetriSharedSetup,
            PetriServerPlugin,
        ))
        .run();
}
