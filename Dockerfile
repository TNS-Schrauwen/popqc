# Build stage
FROM rust:1.95-trixie AS builder

WORKDIR /app

# Copy only what’s needed for build
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --release --workspace

# Runtime stage
FROM debian:trixie-slim

WORKDIR /app

# Copy the compiled binary
COPY --from=builder /app/target/release/popqc /usr/local/bin/popqc

CMD ["popqc"]
