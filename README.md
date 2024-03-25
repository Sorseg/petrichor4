# Petrichor

For faster re-builds times, it is recommended to use `--features bevy/dynamic_linking` flag

Run server
```shell
cargo run --bin petri_server --features bevy/dynamic_linking
```

Run client
```shell
cargo run --bin petri_client --features bevy/dynamic_linking
```

## Bevy coordinates

- The X axis goes from left to right (+X points right).
- The Y axis goes from bottom to top (+Y points up).
- The Z axis goes from far to near (+Z points towards you, out of the screen).

## Assets

When saving blend files, check "compressed"

When working on the assets,

excerpt from [bevy doc](https://bevyengine.org/news/bevy-0-12/#enabling-pre-processing):

> `cargo run --features bevy/asset_processor,bevy/file_watcher`
> This will start the AssetProcessor in parallel with your app.
> It will run until all assets are read from their source

(`asset_sources` in the case of Petrichor4)

> processed, and the results have been written to their destination 
> (defaults to the imported_assets folder). 
> This pairs with asset hot-reloading. 
> If you make a change, this will be detected by the AssetProcessor, 
> the asset will be reprocessed, and the result will be hot-reloaded in your app.