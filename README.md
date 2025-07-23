# Docker Networking Tests

This is a Cargo workspace containing two separate Axum server applications for testing Docker networking capabilities and DNS resolution between containers.

## Project Structure

```
docker_networking_tests/
├── Cargo.toml              # Workspace configuration
├── docker-compose.yml      # Docker Compose configuration
├── .dockerignore           # Docker ignore file
├── README.md               # This file
├── server1/                # First Axum server application
│   ├── Cargo.toml
│   ├── Dockerfile
│   └── src/
│       └── main.rs
└── server2/                # Second Axum server application
    ├── Cargo.toml
    ├── Dockerfile
    └── src/
        └── main.rs
```

## Features

- **server1**: Axum HTTP server running on port 8080 (mapped to host port 8081)
- **server2**: Axum HTTP server running on port 8080 (mapped to host port 8082)
- **Bridge Network**: Both servers are connected to a custom bridge network for DNS resolution testing
- **Docker Compose**: Easy orchestration of both services

## Building and Running

### Local Development

#### Build the entire workspace:
```bash
cargo build
```

#### Run individual servers locally:
```bash
# Run server1 (will be available at http://localhost:8080)
cargo run --bin server1

# Run server2 (will be available at http://localhost:8080)
cargo run --bin server2
```

### Docker Development

#### Build and run with Docker Compose:
```bash
# Build and start both services
docker-compose up --build

# Run in detached mode
docker-compose up -d --build

# Stop the services
docker-compose down
```

#### Access the services:
- **server1**: http://localhost:8081
- **server2**: http://localhost:8082

### DNS Resolution Testing

Once running with Docker Compose, both containers can communicate with each other using their service names:

```bash
# From inside server1 container, you can reach server2 at:
http://server2:8080

# From inside server2 container, you can reach server1 at:  
http://server1:8080
```

To test DNS resolution between containers:
```bash
# Connect to server1 container
docker exec -it server1 /bin/bash

# Test DNS resolution to server2
nslookup server2
# or
ping server2
```

## Development

Each server is a separate binary crate within the workspace. You can add dependencies to each server independently by editing their respective `Cargo.toml` files.

The workspace configuration in the root `Cargo.toml` allows sharing common metadata like version, edition, and authors across all workspace members.
