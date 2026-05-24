FROM rust:1.85-slim-bookworm AS builder

WORKDIR /app
COPY . .
RUN cargo build --release --bin daemon

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/daemon /usr/local/bin/flovenet
EXPOSE 9090 8080
ENTRYPOINT ["flovenet"]
