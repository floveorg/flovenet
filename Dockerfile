# Flovenet Multiplatform Docker Image
# Build with:
#   docker build -t flovenet:latest .
#   docker build --platform linux/amd64,linux/arm64 -t flovenet:latest .

# Stage 1: Build the Rust binary
FROM rust:1.85-slim-bookworm AS chef
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin daemon

# Stage 2: Production image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/daemon /usr/local/bin/flovenet

# Platform detection support
ENV FLOVENET_PLATFORM=linux

EXPOSE 9090 8080
ENTRYPOINT ["flovenet"]
