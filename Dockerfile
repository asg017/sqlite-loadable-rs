# kernel 6.1
FROM debian:bookworm-slim

RUN apt-get update

# development
RUN apt-get install -y curl valgrind build-essential clang pahole git

# connect with vs code remote via ssh
RUN apt install -y openssh-server

# project
RUN apt-get install -y libsqlite3-dev sqlite3 liburing-dev

# upgrade kernel to 6.1
RUN apt upgrade -y linux-image-arm64

# rust
ENV RUST_VERSION=stable
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain=$RUST_VERSION

# Install cargo-valgrind
RUN /bin/bash -c "source /root/.cargo/env && cargo install cargo-valgrind"

# Check sqlite compile options:
RUN echo "PRAGMA compile_options;" | sqlite3

RUN sed -i 's/#PermitRootLogin prohibit-password/PermitRootLogin yes/' /etc/ssh/sshd_config

EXPOSE 22

ENTRYPOINT service ssh start && bash

# on visual code studio
# install "remote development" "remote - ssh" "rust-analyzer"
