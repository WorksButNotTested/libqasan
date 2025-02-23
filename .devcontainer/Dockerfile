################################################################################
# LIBRASAN                                                                     #
################################################################################
FROM ubuntu:24.04 AS librasan
ARG http_proxy
ARG https_proxy

ENV http_proxy=$http_proxy
ENV https_proxy=$https_proxy

ENV DEBIAN_FRONTEND="noninteractive"
RUN dpkg --add-architecture i386
RUN apt-get update && \
    apt-get install -y \
    curl \
    git \
    build-essential \
    gdb \
    gdb-multiarch \
    libclang-dev \
    qemu-user \
    gcc-arm-linux-gnueabi \
    gcc-aarch64-linux-gnu \
    gcc-powerpc-linux-gnu \
    gcc-i686-linux-gnu \
    python3 \
    python3-pip \
    ninja-build \
    libglib2.0-dev \
    llvm-18 \
    libc6-dev:i386 \
    libgcc-13-dev:i386

# Install rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.84.0
ENV PATH="${PATH}:/root/.cargo/bin"

RUN rustup target add armv7-unknown-linux-gnueabi
RUN rustup target add aarch64-unknown-linux-gnu
RUN rustup target add i686-unknown-linux-gnu
RUN rustup target add powerpc-unknown-linux-gnu

RUN rustup install nightly
RUN rustup target add powerpc-unknown-linux-gnu --toolchain nightly

# binstall
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

# cargo-fuzz
RUN cargo binstall -y cargo-fuzz

# taplo
RUN cargo binstall -y taplo-cli

ENV LLVM_CONFIG=llvm-config-18

# cargo-nextest
RUN cargo binstall -y cargo-nextest

# cargo-nextest
RUN cargo binstall -y just
