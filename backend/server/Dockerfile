FROM rust:1.75 as builder
WORKDIR /app

# Copy manifests first for caching
COPY Cargo.toml Cargo.lock ./
COPY backend/core/Cargo.toml backend/core/Cargo.toml
COPY backend/cli/Cargo.toml backend/cli/Cargo.toml
COPY backend/server/Cargo.toml backend/server/Cargo.toml
COPY backend/core/methods/Cargo.toml backend/core/methods/Cargo.toml

# Create dummy src files to build deps
RUN mkdir -p backend/core/src && echo "fn main() {}" > backend/core/src/lib.rs
RUN mkdir -p backend/cli/src && echo "fn main() {}" > backend/cli/src/main.rs
RUN mkdir -p backend/server/src && echo "fn main() {}" > backend/server/src/main.rs
RUN mkdir -p backend/core/methods/guest/src && echo "fn main() {}" > backend/core/methods/guest/src/main.rs

# Build dependencies
# Note: This might fail if workspace structure is complex, but commonly works. 
# Alternatively, just copy everything and build.
COPY . .
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
