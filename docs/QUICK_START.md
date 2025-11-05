# Quick Start Guide

## Step 1: Create `.env` File

Create a `.env` file in the project root:

```bash
cd /home/stelg/Documents/beautibuk-agent
cp .env.example .env
```

## Step 2: Add Your API Keys

Edit `.env` and replace the placeholder values:

### For Groq (Fast, Free - Recommended)

```bash
LLM_PROVIDER=groq
GROQ_API_KEY=gsk_YOUR_ACTUAL_GROQ_KEY_HERE
```

**Get your Groq key:**
1. Go to https://console.groq.com
2. Sign up/login
3. Create API key
4. Copy key (starts with `gsk_`)

### For Google AI Studio (Quality, Free)

```bash
LLM_PROVIDER=google
GOOGLE_AI_API_KEY=YOUR_ACTUAL_GOOGLE_KEY_HERE
```

**Get your Google key:**
1. Go to https://aistudio.google.com
2. Sign up/login
3. Click "Get API Key"
4. Copy the key

### Embeddings (Always Google)

```bash
EMBEDDING_PROVIDER=google
GOOGLE_AI_API_KEY=YOUR_ACTUAL_GOOGLE_KEY_HERE
EMBEDDING_MODEL=text-embedding-004
```

**Note:** If using Groq for LLM, you still need Google AI Studio key for embeddings.

## Step 3: Configure Database

Update the database URL in `.env`:

```bash
DATABASE_URL=postgresql://user:password@localhost:5432/beautibuk_agent
```

Make sure PostgreSQL with pgvector is running!

## Step 4: Configure MCP Server

Make sure your Python backend MCP server is running:

```bash
# In beautibuk-back directory
python run_mcp.py http
```

The MCP server should be on `http://localhost:8002`

## Step 5: Run the Agent

```bash
cargo run
```

The agent will:
- Load environment variables from `.env`
- Connect to MCP server
- Connect to PostgreSQL
- Start on port 3000 (or configured port)

## Example Complete `.env`

```bash
# MCP Server
MCP_SERVER_URL=http://localhost:8002

# LLM (Groq)
LLM_PROVIDER=groq
GROQ_API_KEY=gsk_abc123xyz789
LLM_MODEL=llama-3.1-8b-instant
LLM_TEMPERATURE=0.7
LLM_MAX_TOKENS=2000

# Embeddings (Google)
EMBEDDING_PROVIDER=google
GOOGLE_AI_API_KEY=AIzaSyAbc123Xyz789
EMBEDDING_MODEL=text-embedding-004

# Database
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/beautibuk_agent

# Server
AGENT_PORT=3000
LOG_LEVEL=info
```

## Troubleshooting

### "API key not set" error
- Check that `.env` file exists in project root
- Verify the variable name matches exactly (case-sensitive)
- Make sure you copied the actual key, not the placeholder

### "MCP connection failed"
- Make sure Python backend MCP server is running
- Check `MCP_SERVER_URL` is correct
- Verify MCP server is accessible at the URL

### "Database connection failed"
- Check PostgreSQL is running
- Verify `DATABASE_URL` is correct
- Make sure pgvector extension is enabled

