FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y curl valgrind build-essential clang
# Install Rust
ENV RUST_VERSION=stable
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain=$RUST_VERSION
# Install cargo-valgrind
RUN /bin/bash -c "source /root/.cargo/env && cargo install cargo-valgrind"
