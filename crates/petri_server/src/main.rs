mod blob_assets;
mod enemy;
mod petri_obj;
mod plugin;

use std::time::Duration;

use bevy::{
    app::{RunMode, ScheduleRunnerPlugin},
    log::LogPlugin,
    prelude::*,
};
use bevy_replicon::prelude::*;
use petri_shared::PetriReplicationSetupPlugin;

use crate::plugin::PetriServerPlugin;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins.set(ScheduleRunnerPlugin {
                run_mode: RunMode::Loop {
                    // run at most 200 ticks/s
                    wait: Some(Duration::from_millis(5)),
                },
            }),
            LogPlugin::default(),
            AssetPlugin {
                // only processes assets when `bevy/asset_processor` cargo feature is enabled
                // only live reloads assets when `bevy/file_watcher` cargo feature is enabled
                mode: AssetMode::Processed,
                file_path: "../../asset_sources".into(),
                ..default()
            },
            ReplicationPlugins
                .build()
                .disable::<ClientPlugin>()
                .set(ServerPlugin {
                    tick_policy: TickPolicy::MaxTickRate(60),
                    ..Default::default()
                }),
            PetriReplicationSetupPlugin,
            PetriServerPlugin,
        ))
        .run();
}
