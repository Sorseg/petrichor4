use bevy::prelude::*;
use bevy_replicon::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            ReplicationPlugins
                .build()
                .disable::<ClientPlugin>()
                .set(ServerPlugin {
                    tick_policy: TickPolicy::MaxTickRate(60),
                    ..Default::default()
                }),
        ))
        .run();
}

fn distribute_events() {}
