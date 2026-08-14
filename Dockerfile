# Stage 1: Static Musl Builder
FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev pkgconfig ca-certificates

WORKDIR /app

# Copy source code and definitions
COPY Cargo.toml Cargo.lock ./
COPY rust-lib-core ./rust-lib-core
COPY rust-web-axum ./rust-web-axum
COPY models ./models

# Build optimized release binary
RUN cargo build --release --bin teaql-registry

# Stage 2: Ultra-minimal Scratch Runtime (~5.8MB total image size)
FROM scratch

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /app/target/release/teaql-registry /teaql-registry

EXPOSE 8081

ENV PORT=8081 \
    RUST_LOG=info

ENTRYPOINT ["/teaql-registry"]
