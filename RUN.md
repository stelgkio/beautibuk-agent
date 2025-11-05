# How to Run the BeautiBuk Agent

## Prerequisites

Before running the agent, make sure you have:

1. ✅ **Rust installed** - Check with: `rustc --version`
2. ✅ **PostgreSQL with pgvector** - Running and accessible
3. ✅ **Python backend MCP server** - Running on port 8002
4. ✅ **API keys configured** - In `.env` file

## Step-by-Step Setup

### 1. Check Prerequisites

```bash
# Check Rust
rustc --version

# Check PostgreSQL (should be running)
psql --version

# Check if MCP server is running
curl http://localhost:8002/
```

### 2. Configure Environment Variables

Edit `.env` file and add your API keys:

```bash
# Required: Add your actual API keys
GROQ_API_KEY=gsk_your_actual_key_here
GOOGLE_AI_API_KEY=your_actual_google_key_here

# Update database URL if needed
DATABASE_URL=postgresql://user:password@localhost:5432/beautibuk_agent
```

### 3. Start Python Backend MCP Server

In a separate terminal:

```bash
cd /home/stelg/Documents/beautibuk-back
python run_mcp.py http
```

The MCP server should start on `http://localhost:8002`

### 4. Start PostgreSQL

Make sure PostgreSQL is running with pgvector extension:

```bash
# Using Docker (recommended)
docker run -d \
  --name postgres-vector \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=beautibuk_agent \
  -p 5432:5432 \
  pgvector/pgvector:pg16

# Or if installed locally
sudo systemctl start postgresql
```

### 5. Create Database and Enable pgvector

```bash
# Connect to PostgreSQL
psql -U postgres -h localhost

# Create database
CREATE DATABASE beautibuk_agent;

# Connect to the database
\c beautibuk_agent

# Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

# Exit
\q
```

### 6. Build and Run the Agent

```bash
cd /home/stelg/Documents/beautibuk-agent

# Build the project (first time)
cargo build

# Run the agent
cargo run
```

Or use release mode for better performance:

```bash
cargo run --release
```

## Expected Output

When running successfully, you should see:

```
Starting BeautiBuk Agent...
Configuration loaded
Database connection established
Database migrations completed
MCP client initialized
Server listening on port 3000
```

## Troubleshooting

### Error: "API key not set"
- Check `.env` file exists in project root
- Verify API key variable names match exactly
- Make sure you're using actual keys, not placeholders

### Error: "MCP connection failed"
- Make sure Python backend MCP server is running
- Check `MCP_SERVER_URL` in `.env` is correct
- Test MCP server: `curl http://localhost:8002/`

### Error: "Database connection failed"
- Verify PostgreSQL is running
- Check `DATABASE_URL` in `.env` is correct
- Make sure database exists and pgvector extension is enabled

### Error: "Migration failed"
- Check database exists
- Verify pgvector extension is installed
- Check database user has CREATE permissions

## Testing the Agent

Once running, test the API:

```bash
# Health check
curl http://localhost:3000/api/health

# Chat endpoint
curl -X POST http://localhost:3000/api/chat \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Find me a salon near downtown",
    "session_id": "test-session-123"
  }'
```

## Development Workflow

1. **Terminal 1**: Run MCP server
   ```bash
   cd beautibuk-back && python run_mcp.py http
   ```

2. **Terminal 2**: Run Rust agent
   ```bash
   cd beautibuk-agent && cargo run
   ```

3. **Terminal 3**: Test API or run frontend
   ```bash
   curl http://localhost:3000/api/health
   ```

## Environment Variables Reference

| Variable | Description | Example |
|----------|-------------|---------|
| `MCP_SERVER_URL` | MCP server address | `http://localhost:8002` |
| `LLM_PROVIDER` | LLM provider (groq/google) | `groq` |
| `GROQ_API_KEY` | Groq API key | `gsk_...` |
| `GOOGLE_AI_API_KEY` | Google AI API key | `AIzaSy...` |
| `DATABASE_URL` | PostgreSQL connection string | `postgresql://...` |
| `AGENT_PORT` | Server port | `3000` |

## Quick Start (All Commands)

```bash
# 1. Start PostgreSQL (if using Docker)
docker run -d --name postgres-vector \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=beautibuk_agent \
  -p 5432:5432 \
  pgvector/pgvector:pg16

# 2. Create database and extension
psql -U postgres -h localhost -c "CREATE DATABASE beautibuk_agent;"
psql -U postgres -h localhost -d beautibuk_agent -c "CREATE EXTENSION IF NOT EXISTS vector;"

# 3. Start MCP server (in separate terminal)
cd beautibuk-back && python run_mcp.py http

# 4. Run agent
cd beautibuk-agent && cargo run
```

## Next Steps

After the agent is running:
- Test with curl commands
- Integrate with frontend
- Check logs for debugging
- Monitor API responses

