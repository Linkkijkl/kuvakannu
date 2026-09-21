# https://github.com/devcontainers/images/tree/main/src/rust

# [Choice] Debian OS version (use bullseye on local arm64/Apple Silicon): buster, bullseye
ARG VARIANT="trixie"
FROM rust:1-${VARIANT}

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive

# Install development tools
RUN apt-get -y install git-lfs
RUN rustup component add clippy
RUN rustup component add rustfmt
RUN cargo install cargo-watch

# Install runtime libraries
# RUN apt-get -y install ffmpeg
# Use nightly releases of Rust
# RUN rustup install nightly
# RUN rustup default nightly
