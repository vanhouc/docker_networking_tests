# Use the official Rust image as a build stage
FROM rust:latest AS builder

# Set the working directory
WORKDIR /app

# Copy the workspace configuration
COPY Cargo.toml ./
COPY server1/Cargo.toml ./server1/
COPY server2/Cargo.toml ./server2/

# Copy the source code
COPY server1/src ./server1/src
COPY server2/src ./server2/src

# Build the application
RUN cargo build --release --bin server2

# Use a minimal runtime image
FROM debian:bookworm-slim

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/server2 /usr/local/bin/server2

# Expose the port
EXPOSE 8080

# Run the binary
CMD ["server2"]
