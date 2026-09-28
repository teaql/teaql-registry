# Stage 1: Static Musl Builder with dependency caching
FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev pkgconfig ca-certificates

WORKDIR /app

# 1. Copy workspace manifests and lock file for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY rust-lib-core/lib/Cargo.toml rust-lib-core/lib/
COPY rust-web-axum/Cargo.toml rust-web-axum/

# Create minimal source stubs so cargo can resolve the workspace
# Remove registry-tui from workspace members (not needed for server binary)
RUN sed -i '/"registry-tui"/d' Cargo.toml && \
    mkdir -p rust-lib-core/lib/src rust-web-axum/src && \
    touch rust-lib-core/lib/src/lib.rs && \
    echo 'fn main() {}' > rust-web-axum/src/main.rs && \
    touch rust-web-axum/src/lib.rs

# 2. Pre-build dependencies only (this layer is cached across rebuilds)
RUN cargo build --release 2>/dev/null; exit 0

# 3. Copy real source code and model definitions
COPY rust-lib-core ./rust-lib-core
COPY rust-web-axum ./rust-web-axum
COPY models ./models

# 4. Touch source files to invalidate cargo's fingerprint and rebuild
RUN touch rust-lib-core/lib/src/lib.rs rust-web-axum/src/main.rs && \
    cargo build --release --bin teaql-registry

# Upload handlers stream multipart bodies to temp files. Scratch has no /tmp.
RUN mkdir -m 1777 /app/runtime-tmp

# Stage 2: Ultra-minimal Scratch Runtime
FROM scratch

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /app/target/release/teaql-registry /teaql-registry
COPY --from=builder /app/runtime-tmp /tmp

EXPOSE 8081

ENV PORT=8081 \
    RUST_LOG=info

ENTRYPOINT ["/teaql-registry"]
