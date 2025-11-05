# Docker Setup for BeautiBuk Agent

## Quick Start

### Build and Run with Docker Compose

```bash
# Make sure you have your API keys in .env file
cp .env.example .env
# Edit .env and add your actual API keys

# Build and start all services
docker-compose up --build

# Or run in detached mode
docker-compose up -d --build
```

### Services Included

- **postgres**: PostgreSQL with pgvector extension
- **agent**: Rust agent service
- **mcp-server**: Python backend MCP server
- **mongodb**: MongoDB for backend data

## Building the Docker Image

### Build the agent image

```bash
docker build -t beautibuk-agent:latest .
```

### Run the container

```bash
docker run -d \
  --name beautibuk-agent \
  -p 3000:3000 \
  --env-file .env \
  --network beautibuk-network \
  beautibuk-agent:latest
```

## Docker Compose Commands

### Start all services

```bash
docker-compose up -d
```

### View logs

```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f agent
```

### Stop services

```bash
docker-compose down
```

### Stop and remove volumes

```bash
docker-compose down -v
```

### Rebuild after code changes

```bash
docker-compose up --build -d
```

## Environment Variables

Create a `.env` file in the project root with:

```bash
# LLM Provider
LLM_PROVIDER=groq
GROQ_API_KEY=gsk_your_key_here
GOOGLE_AI_API_KEY=your_key_here

# MongoDB (if using)
MONGODB_URL=mongodb://admin:password@mongodb:27017
DATABASE_NAME=beautibuk
```

The docker-compose.yml will automatically use these variables.

## Development Setup

For development, you might want to run the agent locally while using Docker for PostgreSQL:

```bash
# Start only PostgreSQL
docker-compose -f docker-compose.dev.yml up -d postgres

# Run agent locally
cargo run
```

## Production Deployment

### Build for production

```bash
# Build optimized image
docker build -t beautibuk-agent:prod --target runtime .

# Or use docker-compose
docker-compose -f docker-compose.prod.yml build
```

### Health Checks

The Dockerfile includes health checks. Check status:

```bash
docker ps
# Look for "healthy" status

# Or check manually
curl http://localhost:3000/api/health
```

## Troubleshooting

### Container won't start

```bash
# Check logs
docker-compose logs agent

# Check if port is available
lsof -i :3000
```

### Database connection issues

```bash
# Check PostgreSQL is running
docker-compose ps postgres

# Check connection
docker-compose exec postgres psql -U postgres -d beautibuk_agent
```

### MCP server connection issues

```bash
# Check MCP server logs
docker-compose logs mcp-server

# Test MCP server
curl http://localhost:8002/
```

### Rebuild after dependency changes

```bash
# Remove old images
docker-compose down
docker system prune -f

# Rebuild
docker-compose build --no-cache
```

## Multi-Stage Build Details

The Dockerfile uses a multi-stage build:

1. **Builder stage**: Installs Rust and build dependencies, compiles the application
2. **Runtime stage**: Uses minimal Debian image with only runtime dependencies

This results in a smaller final image (~100MB vs ~1GB if using full Rust image).

## Port Mapping

- **3000**: Agent API (HTTP)
- **5432**: PostgreSQL
- **8002**: MCP Server
- **27017**: MongoDB

## Volumes

- `postgres_data`: PostgreSQL data persistence
- `mongodb_data`: MongoDB data persistence

## Network

All services are on the `beautibuk-network` bridge network, allowing them to communicate using service names.

## Security Notes

1. **Never commit `.env` file** - It's in `.gitignore`
2. **Use secrets management** in production (Docker secrets, Kubernetes secrets, etc.)
3. **Limit network exposure** - Only expose necessary ports
4. **Use non-root user** - The Dockerfile runs as `appuser` (UID 1000)

