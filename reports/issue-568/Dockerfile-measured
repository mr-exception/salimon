FROM rust:1.89.0-bookworm AS rust
FROM node:22.16.0-bookworm-slim AS node
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive CARGO_HOME=/usr/local/cargo RUSTUP_HOME=/usr/local/rustup
ENV PATH=/opt/venv/bin:/usr/local/cargo/bin:/usr/local/bin:/usr/bin:/bin
COPY --from=rust /usr/local/cargo /usr/local/cargo
COPY --from=rust /usr/local/rustup /usr/local/rustup
COPY --from=node /usr/local /usr/local
RUN apt-get update && apt-get install -y --no-install-recommends build-essential pkg-config git ca-certificates python3 python3-venv libasound2-dev libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev libx11-dev libxi-dev libxcursor-dev libxrandr-dev libvulkan1 mesa-vulkan-drivers xvfb xauth xdotool imagemagick && rm -rf /var/lib/apt/lists/*
RUN python3 -m venv /opt/venv && pip install jsonschema==4.26.0 Pillow==12.3.0
ENV CARGO_BUILD_JOBS=2 WGPU_BACKEND=vulkan
WORKDIR /source
