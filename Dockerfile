# Build stage
FROM rust:latest as builder

WORKDIR /app

# Copy the Cargo.toml first to cache dependencies
COPY Cargo.toml ./

# Create a dummy src/main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this will be cached)
RUN cargo build --release

# Remove the dummy main.rs
RUN rm src/main.rs

# Copy the real source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/docker_networking_tests /usr/local/bin/docker_networking_tests

# Set the startup command
CMD ["docker_networking_tests"]
