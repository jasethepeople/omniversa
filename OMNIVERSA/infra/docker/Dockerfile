FROM rust:1.78-slim-bookworm AS builder
WORKDIR /build
RUN apt-get update && apt-get install -y pkg-config libssl-dev cmake clang && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ ./crates/
RUN cargo build --release --bin omniversa-core

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/* && useradd -m -s /bin/false sovereign
COPY --from=builder /build/target/release/omniversa-core /usr/local/bin/
USER sovereign
EXPOSE 8080 9090
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 CMD curl -f http://localhost:8080/health || exit 1
ENTRYPOINT ["omniversa-core"]
