# Declarative sandbox kit Dockerfile for Portable State Engine
# Layers the Debian trixie Rust 1.85.1 toolchain onto the base agent workload.
FROM docker.io/sbx/antigravity:latest

USER root
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      build-essential=12.12+dhi1 \
      ca-certificates \
      cargo=1.85.1+dfsg1-1+deb13u1 \
      rust-clippy=1.85.1+dfsg1-1+deb13u1 \
      rustc=1.85.1+dfsg1-1+deb13u1 \
      rustfmt=1.85.1+dfsg1-1+deb13u1 && \
    rm -rf /var/lib/apt/lists/*

USER agent
