# ==============================================================================
# zkIPFS Proof Backend Server Dockerfile
# ==============================================================================

# Stage 1: RISC0 Toolchain Installation
FROM rust:latest as risc0-base
WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev pkg-config build-essential clang lld git cmake perl curl \
    && rm -rf /var/lib/apt/lists/*

# Install RISC0 toolchain
# The rzup installer sets up the RISC0 proving environment
ENV RISC0_DEV_MODE=1
RUN curl -L https://risczero.com/install | bash \
    && $HOME/.risc0/bin/rzup install

# Add RISC0 to PATH
ENV PATH="$HOME/.risc0/bin:$PATH"

# Stage 2: Build the application
FROM risc0-base as builder
WORKDIR /app

# Copy Cargo manifests for better layer caching
COPY Cargo.toml Cargo.lock* ./
COPY backend/core/Cargo.toml backend/core/Cargo.toml
COPY backend/cli/Cargo.toml backend/cli/Cargo.toml
COPY backend/server/Cargo.toml backend/server/Cargo.toml
COPY backend/core/methods/Cargo.toml backend/core/methods/Cargo.toml
COPY backend/core/guest/Cargo.toml backend/core/guest/Cargo.toml

# Copy the rest of the source code
COPY . .

# Build the server in release mode
# RISC0_DEV_MODE=1 uses fake proofs for faster builds (disable in production)
RUN cargo build --release --bin zkipfs-proof-server

# Stage 3: Runtime
FROM debian:bookworm-slim
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary and config
COPY --from=builder /app/target/release/zkipfs-proof-server .
COPY --from=builder /app/config /app/config

# Create non-root user for security
RUN useradd -r -s /bin/false zkipfs && chown -R zkipfs:zkipfs /app
USER zkipfs

# Environment variables
ENV RUST_LOG=info
ENV RISC0_DEV_MODE=1

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

EXPOSE 3000
CMD ["./zkipfs-proof-server"]
