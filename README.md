# Docker Networking Tests

This is a Rust project containing a single configurable Axum server application that can be deployed as multiple instances for testing Docker networking capabilities and DNS resolution between containers.

## Project Structure

```
docker_networking_tests/
├── Cargo.toml              # Project configuration
├── docker-compose.yml      # Docker Compose configuration
├── Dockerfile              # Multi-stage Docker build
├── .dockerignore           # Docker ignore file
├── README.md               # This file
└── src/
    └── main.rs             # Single configurable Axum server
```

## Features

- **Configurable Single Binary**: One Rust application that can be configured via environment variables
- **Environment-Based Configuration**: 
  - `SERVER_NAME`: Display name for the server instance
  - `TARGET_SERVER`: Target server for DNS resolution testing
- **Multiple Endpoints**:
  - `/` - Hello world page with navigation links
  - `/test-dns` - Test DNS resolution between containers
  - `/test-google` - Test external connectivity to Google
- **Bridge Network**: Both container instances are connected to a custom bridge network for DNS resolution testing
- **Docker Compose**: Easy orchestration of both service instances

## Building and Running

### Local Development

#### Build the project:
```bash
cargo build
```

#### Run locally with environment variables:
```bash
# Run as server1 (will be available at http://localhost:8080)
SERVER_NAME="Server 1" TARGET_SERVER="localhost:8082" cargo run

# Run as server2 (will be available at http://localhost:8080)
SERVER_NAME="Server 2" TARGET_SERVER="localhost:8081" cargo run
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
- **Server 1**: http://localhost:8081 (configured with `SERVER_NAME="Server 1"` and `TARGET_SERVER="server2:8080"`)
- **Server 2**: http://localhost:8082 (configured with `SERVER_NAME="Server 2"` and `TARGET_SERVER="server1:8080"`)

### Available Endpoints

Each server instance provides the following endpoints:

- **`GET /`**: Hello world page with server identification and navigation links
- **`GET /test-dns`**: Test DNS resolution to the target server (configured via `TARGET_SERVER`)
- **`GET /test-google`**: Test external connectivity to Google.com

### DNS Resolution Testing

Once running with Docker Compose, both container instances can communicate with each other using their service names:

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

### Cross-Compose DNS Testing

This project includes `docker-compose-external.yml` for testing DNS resolution across separate Docker Compose instances sharing the same bridge network.

#### Setup:

**Primary Compose Stack** (`docker-compose.yml`):
- **server1** (port 8081) → targets `server2:8080`
- **server2** (port 8082) → targets `server1:8080`
- Creates the `dns-test-bridge` network

**External Compose Stack** (`docker-compose-external.yml`):
- **server3** (port 8083) → targets `server1:8080` (from primary compose)
- **server4** (port 8084) → targets `server2:8080` (from primary compose)
- References the existing `dns-test-bridge` network as external

#### Testing Steps:

1. **Start the primary compose stack:**
```bash
docker-compose up -d
```

2. **Start the external compose stack:**
```bash
docker-compose -f docker-compose-external.yml up -d
```

3. **Access all services:**
   - **Server 1**: http://localhost:8081
   - **Server 2**: http://localhost:8082  
   - **Server 3**: http://localhost:8083 (can reach server1 and server2)
   - **Server 4**: http://localhost:8084 (can reach server1 and server2)

4. **Test cross-compose connectivity via web interface:**
```bash
# From server3, test connectivity to servers in the original compose
curl http://localhost:8083/test-dns?target=server1:8080
curl http://localhost:8083/test-dns?target=server2:8080

# From server1, test connectivity to servers in the external compose
curl http://localhost:8081/test-dns?target=server3:8080
curl http://localhost:8081/test-dns?target=server4:8080
```

5. **Manual container DNS testing:**
```bash
# Connect to server3 and test DNS resolution to original compose servers
docker exec -it server3 /bin/bash
nslookup server1
nslookup server2
ping server1
ping server2

# Connect to server1 and test DNS resolution to external compose servers
docker exec -it server1 /bin/bash
nslookup server3
nslookup server4
```

6. **Cleanup:**
```bash
# Stop external compose
docker-compose -f docker-compose-external.yml down

# Stop primary compose
docker-compose down
```

#### Key Features:
- **External Network Reference**: Uses `external: true` to share the `dns-test-bridge` network
- **Cross-Instance Communication**: All 4 servers can communicate via DNS names
- **Independent Management**: Start/stop each compose stack independently
- **Port Isolation**: Each server uses a unique host port (8081, 8082, 8083, 8084)

## Configuration

The application is configured entirely through environment variables:

- **`SERVER_NAME`** (optional): Display name for the server instance. Defaults to "Unknown Server"
- **`TARGET_SERVER`** (optional): Target server for DNS testing. Defaults to "other-server:8080"

## Development

This is a single binary Rust project using Axum for the HTTP server and reqwest for making HTTP requests. The same binary is deployed as two different container instances with different environment variable configurations.

Key dependencies:
- `axum`: HTTP server framework
- `tokio`: Async runtime
- `reqwest`: HTTP client for testing connectivity
- `tracing-subscriber`: Logging
