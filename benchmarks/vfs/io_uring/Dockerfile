# kernel 6.1
FROM debian:bookworm-slim

RUN apt-get update

# development
RUN apt-get install -y curl valgrind build-essential clang pahole

# project
RUN apt-get install -y sqlite3 liburing-dev

# upgrade kernel to 6.1
RUN apt upgrade -y linux-image-arm64

# rust
ENV RUST_VERSION=stable
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain=$RUST_VERSION

# Install cargo-valgrind
RUN /bin/bash -c "source /root/.cargo/env && cargo install cargo-valgrind && cargo install --locked hyperfine"

# Check sqlite compile options:
RUN echo "PRAGMA compile_options;" | sqlite3
