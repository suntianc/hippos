# Hippos Deployment Guide

**High-Performance Context Management Service for AI Agents**

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Installation](#installation)
3. [Configuration](#configuration)
4. [Running the Server](#running-the-server)
5. [Docker Deployment](#docker-deployment)
6. [Production Setup](#production-setup)
7. [Monitoring](#monitoring)
8. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### System Requirements

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| CPU | 2 cores | 4+ cores |
| Memory | 4 GB | 8+ GB |
| Storage | 10 GB | 50+ GB SSD |
| Network | 100 Mbps | 1 Gbps |

### Software Dependencies

- **Rust**: 1.70.0 or later (2024 Edition)
- **SurrealDB**: 2.0.0 or later
- **Cargo**: Latest stable version

### Verify Dependencies

```bash
# Check Rust version
rustc --version

# Check Cargo version
cargo --version

# Check SurrealDB version
surrealdb --version
```

---

## Installation

### Option 1: Build from Source

```bash
# Clone the repository
git clone https://github.com/hippos/hippos.git
cd hippos

# Build the release binary
cargo build --release

# Verify the build
./target/release/hippos --version
```

### Option 2: Install via Cargo

```bash
cargo install hippos
```

### Option 3: Download Pre-built Binary

```bash
# Download the latest release
curl -L https://github.com/hippos/hippos/releases/latest/download/hippos-x86_64-unknown-linux-musl.tar.gz | tar xz

# Make executable
chmod +x hippos

# Move to PATH
sudo mv hippos /usr/local/bin/
```

---

## Configuration

### Quick Start (Default Configuration)

The simplest way to run Hippos is with default settings:

```bash
# Run with default configuration (in-memory database)
cargo run

# Or with the binary
./target/release/hippos
```

### Configuration File

Create a `config.yaml` file in the application directory:

```yaml
# Application Configuration
app:
  name: "hippos"
  environment: "development"
  host: "0.0.0.0"
  port: 8080
  workers: 4

# Database Configuration
database:
  url: "ws://localhost:8000"
  namespace: "hippos"
  database: "memories"
  username: "root"
  password: "root"
  min_connections: 5
  max_connections: 50
  connection_timeout: 30
  idle_timeout: 300

# Cache Configuration
cache:
  enabled: true
  max_size: 10000
  ttl_seconds: 300

# Batch Processing
batch:
  enabled: true
  size_threshold: 100
  timeout_ms: 50

# Security Configuration
security:
  api_key: "your-secure-api-key"
  jwt_secret: "your-jwt-secret-min-32-chars"
  rate_limit_enabled: false

# Logging Configuration
logging:
  level: "info"
  format: "json"
```

### Environment Variables

Override configuration using environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `HIPPOS_APP_NAME` | `hippos` | Application name |
| `HIPPOS_ENVIRONMENT` | `development` | Environment mode |
| `HIPPOS_SERVER_HOST` | `0.0.0.0` | Server bind address |
| `HIPPOS_SERVER_PORT` | `8080` | Server port |
| `HIPPOS_SERVER_WORKERS` | `4` | Worker threads |
| `HIPPOS_DATABASE_URL` | `ws://localhost:8000` | SurrealDB URL |
| `HIPPOS_DATABASE_NAMESPACE` | `hippos` | Database namespace |
| `HIPPOS_DATABASE_NAME` | `memories` | Database name |
| `HIPPOS_API_KEY` | `dev-api-key` | Default API key |
| `HIPPOS_LOG_LEVEL` | `info` | Logging level |

---

## Running the Server

### Development Mode

```bash
# Run with default configuration
cargo run

# Run with custom config file
HIPPOS_CONFIG=/path/to/config.yaml cargo run

# Run with environment variables
HIPPOS_SERVER_PORT=8080 HIPPOS_API_KEY=my-secret cargo run
```

### Production Mode

```bash
# Build release binary
cargo build --release

# Run with configuration file
./target/release/hippos --config /etc/hippos/config.yaml

# Run with environment variables
HIPPOS_ENVIRONMENT=production ./target/release/hippos
```

### Verify the Server

```bash
# Health check
curl http://localhost:8080/health

# Expected response:
# {
#   "status": "healthy",
#   "version": "0.1.0",
#   "timestamp": "2024-01-15T12:00:00Z"
# }
```

### API Endpoints

```bash
# List memories
curl -H "Authorization: ApiKey dev-api-key" \
  http://localhost:8080/api/v1/memories

# Create a memory
curl -X POST \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{"content": "User asked about Rust async programming", "memory_type": "episodic"}' \
  http://localhost:8080/api/v1/memories
```

---

## Docker Deployment

### Using Docker Compose

Create a `docker-compose.yml` file:

```yaml
version: '3.8'

services:
  hippos:
    image: hippos/hippos:latest
    container_name: hippos
    ports:
      - "8080:8080"
    environment:
      - HIPPOS_ENVIRONMENT=production
      - HIPPOS_DATABASE_URL=ws://surrealdb:8000
      - HIPPOS_DATABASE_NAMESPACE=hippos
      - HIPPOS_DATABASE_NAME=memories
      - HIPPOS_API_KEY=${HIPPOS_API_KEY}
    volumes:
      - hippos_data:/data
    depends_on:
      - surrealdb
    restart: unless-stopped

  surrealdb:
    image: surrealdb/surrealdb:2.0.0
    container_name: surrealdb
    ports:
      - "8000:8000"
    environment:
      - SURREALDB_ARGS=--bind=0.0.0.0:8000 --user=root --pass=root
    volumes:
      - surrealdb_data:/var/lib/surrealdb
    restart: unless-stopped

volumes:
  hippos_data:
  surrealdb_data:
```

Start the services:

```bash
# Set API key
export HIPPOS_API_KEY=$(openssl rand -base64 32)

# Start services
docker-compose up -d

# Check status
docker-compose ps

# View logs
docker-compose logs -f hippos
```

### Using Docker

```bash
# Run SurrealDB
docker run -d \
  --name surrealdb \
  -p 8000:8000 \
  -v surrealdb_data:/var/lib/surrealdb \
  surrealdb/surrealdb:2.0.0 \
  start --bind=0.0.0.0:8000 --user=root --pass=root

# Run Hippos
docker run -d \
  --name hippos \
  -p 8080:8080 \
  -e HIPPOS_DATABASE_URL=ws://host.docker.internal:8000 \
  -e HIPPOS_API_KEY=your-api-key \
  hippos/hippos:latest
```

### Using Kubernetes

Create a Kubernetes deployment:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hippos
spec:
  replicas: 3
  selector:
    matchLabels:
      app: hippos
  template:
    metadata:
      labels:
        app: hippos
    spec:
      containers:
      - name: hippos
        image: hippos/hippos:latest
        ports:
        - containerPort: 8080
        env:
        - name: HIPPOS_DATABASE_URL
          value: "ws://surrealdb:8000"
        - name: HIPPOS_API_KEY
          valueFrom:
            secretKeyRef:
              name: hippos-secrets
              key: api-key
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
---
apiVersion: v1
kind: Service
metadata:
  name: hippos
spec:
  selector:
    app: hippos
  ports:
  - port: 80
    targetPort: 8080
  type: LoadBalancer
```

---

## Production Setup

### Systemd Service

Create a systemd service file:

```ini
[Unit]
Description=Hippos Memory Service
After=network.target surrealdb.service

[Service]
Type=simple
User=hippos
Group=hippos
WorkingDirectory=/opt/hippos
ExecStart=/opt/hippos/hippos --config /etc/hippos/config.yaml
Restart=on-failure
RestartSec=5
StandardOutput=journal
StandardError=journal
Environment=HIPPOS_ENVIRONMENT=production

[Install]
WantedBy=multi-user.target
```

Install and enable the service:

```bash
# Create user
sudo useradd -r -s /sbin/nologin hippos

# Create directories
sudo mkdir -p /opt/hippos /etc/hippos /var/log/hippos

# Copy binary and config
sudo cp hippos /opt/hippos/
sudo cp config.yaml /etc/hippos/

# Set permissions
sudo chown -R hippos:hippos /opt/hippos /etc/hippos /var/log/hippos

# Copy service file
sudo cp hippos.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable hippos
sudo systemctl start hippos
```

### Nginx Reverse Proxy

Configure Nginx as a reverse proxy:

```nginx
upstream hippos {
    server 127.0.0.1:8080;
    keepalive 32;
}

server {
    listen 80;
    server_name hippos.example.com;

    location / {
        proxy_pass http://hippos;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # WebSocket support
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_read_timeout 86400;
    }

    # Health check endpoint
    location /health {
        proxy_pass http://hippos/health;
    }

    # Metrics endpoint
    location /metrics {
        proxy_pass http://hippos/metrics;
    }
}
```

### SSL/TLS Configuration

Using Let's Encrypt with Certbot:

```bash
# Install certbot
sudo apt install certbot python3-certbot-nginx

# Obtain certificate
sudo certbot --nginx -d hippos.example.com

# Auto-renewal
sudo systemctl enable certbot.timer
```

---

## Monitoring

### Health Endpoints

```bash
# Full health check
curl http://localhost:8080/health

# Liveness probe
curl http://localhost:8080/health/live

# Readiness probe
curl http://localhost:8080/health/ready
```

### Prometheus Metrics

```bash
# Get metrics
curl http://localhost:8080/metrics

# Example metrics:
# http_requests_total 1234
# http_request_duration_seconds_bucket{le="0.005"} 500
# cache_hit_total 800
# cache_miss_total 200
```

### Structured Logging

Logs are output in JSON format by default:

```json
{
  "timestamp": "2024-01-15T12:00:00Z",
  "level": "INFO",
  "message": "Memory created",
  "memory_id": "mem_abc123",
  "memory_type": "episodic",
  "request_id": "req_xyz789"
}
```

Configure log level:

```bash
HIPPOS_LOG_LEVEL=debug cargo run
```

### Grafana Dashboard

Import the pre-built dashboard from `docs/grafana-dashboard.json` for visualization.

---

## Troubleshooting

### Common Issues

#### 1. Connection Refused to SurrealDB

```bash
# Check SurrealDB is running
docker ps | grep surrealdb

# Check SurrealDB logs
docker logs surrealdb

# Verify connection
curl http://localhost:8000/status
```

**Solution**: Ensure SurrealDB is running and the URL is correct.

#### 2. Authentication Failed

```bash
# Check API key
echo $HIPPOS_API_KEY

# Verify with correct key
curl -H "Authorization: ApiKey $HIPPOS_API_KEY" \
  http://localhost:8080/health
```

**Solution**: Ensure the API key is set correctly and matches in configuration.

#### 3. Memory Issues

```bash
# Check memory usage
htop

# Increase swap if needed
sudo fallocate -l 2G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

**Solution**: Increase system memory or reduce worker threads.

#### 4. Slow Performance

```bash
# Check database performance
curl http://localhost:8080/metrics | grep search

# Enable query logging
HIPPOS_LOG_LEVEL=debug cargo run
```

**Solutions**:
- Increase cache size
- Enable batch processing
- Use SSD storage
- Scale horizontally

### Debug Mode

```bash
# Enable debug logging
HIPPOS_LOG_LEVEL=debug cargo run

# Enable tracing
HIPPOS_LOG_LEVEL=trace RUST_LOG=trace cargo run
```

### Log Files

Default log locations:
- Systemd: `journalctl -u hippos -f`
- Docker: `docker logs hippos`
- Manual: `/var/log/hippos/app.log`

---

## Performance Tuning

### Recommended Settings

For production with 10GB+ memory:

```yaml
app:
  workers: 8

database:
  max_connections: 100

cache:
  max_size: 50000
  ttl_seconds: 600

batch:
  size_threshold: 200
  timeout_ms: 100
```

### Database Tuning

SurrealDB configuration in `/etc/surrealdb/surrealdb.toml`:

```toml
[storage]
memory-cache = "2GiB"
disk-threshold = "512MiB"

[query]
max-nesting-depth = 512
max-logical-depth = 64
```

---

## Upgrades

### Rolling Update (Zero Downtime)

```bash
# Pull new image
docker pull hippos/hippos:latest

# Update services
docker-compose up -d hippos

# Verify health
curl http://localhost:8080/health
```

### Blue-Green Deployment

```bash
# Start new version on different port
HIPPOS_SERVER_PORT=8081 ./target/release/hippos

# Test new version
curl http://localhost:8081/health

# Switch traffic (using load balancer)
# ...

# Stop old version
pkill -f "hippos.*8080"
```

---

## Support

- **Documentation**: [docs.hippos.io](https://docs.hippos.io)
- **Issues**: [GitHub Issues](https://github.com/hippos/hippos/issues)
- **Discussions**: [GitHub Discussions](https://github.com/hippos/hippos/discussions)

---

**Hippos** - Empowering AI Agents with Persistent Memory
