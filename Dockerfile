FROM docker.io/rust:1.76 as builder

RUN apt-get update && apt-get install g++ pkg-config libx11-dev libasound2-dev libudev-dev libxkbcommon-x11-0 -y --no-install-recommends
WORKDIR /usr/local/src

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY crates/petri_server crates/petri_server
COPY crates/petri_shared crates/petri_shared

RUN \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/src/target \
    cargo build --bin petri_server --release; cp ./target/release/petri_server petri_server

FROM ubuntu:latest
WORKDIR game
# FIXME: figure out bevy features so the server doesn't need sound libraries
RUN apt-get update && apt-get install -y --no-install-recommends libasound2-dev
COPY --from=builder /usr/local/src/petri_server petri_server
COPY crates/petri_server/assets assets
ENTRYPOINT ["./petri_server", "--flyio"]
