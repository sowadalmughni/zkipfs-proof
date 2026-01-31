FROM rust:latest as builder
WORKDIR /app
RUN apt-get update && apt-get install -y libssl-dev pkg-config build-essential clang lld git cmake perl

# Copy manifests first for caching (commented out for stability)
# COPY Cargo.toml ./
# COPY backend/core/Cargo.toml backend/core/Cargo.toml
# COPY backend/cli/Cargo.toml backend/cli/Cargo.toml
# COPY backend/server/Cargo.toml backend/server/Cargo.toml
# COPY backend/core/methods/Cargo.toml backend/core/methods/Cargo.toml

# Copy methods directory fully as Risc0 build scripts might need it
# COPY backend/core/methods backend/core/methods

# Create dummy src files to build deps
# RUN mkdir -p backend/core/src && echo "fn main() {}" > backend/core/src/lib.rs
# RUN mkdir -p backend/cli/src && echo "fn main() {}" > backend/cli/src/main.rs
# RUN mkdir -p backend/server/src && echo "fn main() {}" > backend/server/src/main.rs

# Build dependencies
# RUN cargo build --release --bin zkipfs-proof-server

# Remove dummy files
# RUN rm -rf backend/core/src backend/cli/src backend/server/src

# Copy source code
COPY . .

# Build the server
RUN cargo build --release --bin zkipfs-proof-server

# Runtime Stage
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/zkipfs-proof-server .
COPY --from=builder /app/config /app/config
# Ensure minimal deps
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
EXPOSE 3000
CMD ["./zkipfs-proof-server"]
