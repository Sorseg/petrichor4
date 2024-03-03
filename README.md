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
