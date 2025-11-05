# Environment Variables Setup

## Where to Add API Tokens

Create a `.env` file in the project root directory (`beautibuk-agent/`) with your API keys.

## Step 1: Create `.env` file

```bash
cd /home/stelg/Documents/beautibuk-agent
cp .env.example .env
```

## Step 2: Add Your API Keys

Edit the `.env` file and add your actual API keys:

### For Groq (Recommended for Development)

```bash
# LLM Provider
LLM_PROVIDER=groq
GROQ_API_KEY=gsk_your_actual_groq_key_here
LLM_MODEL=llama-3.1-8b-instant

# Embeddings (still use Google for embeddings)
EMBEDDING_PROVIDER=google
GOOGLE_AI_API_KEY=your_actual_google_key_here
EMBEDDING_MODEL=text-embedding-004
```

### For Google AI Studio (Alternative)

```bash
# LLM Provider
LLM_PROVIDER=google
GOOGLE_AI_API_KEY=your_actual_google_key_here
LLM_MODEL=gemini-2.0-flash-exp

# Embeddings (same key)
EMBEDDING_PROVIDER=google
EMBEDDING_MODEL=text-embedding-004
```

## Getting Your API Keys

### Groq API Key
1. Go to https://console.groq.com
2. Sign up or log in
3. Navigate to API Keys section
4. Create a new API key
5. Copy the key (starts with `gsk_`)
6. Add it to `.env` as `GROQ_API_KEY`

### Google AI Studio API Key
1. Go to https://aistudio.google.com
2. Sign up or log in
3. Click "Get API Key" or go to API Keys section
4. Create a new API key
5. Copy the key
6. Add it to `.env` as `GOOGLE_AI_API_KEY`

## Complete `.env` File Example

```bash
# MCP Server Configuration
MCP_SERVER_URL=http://localhost:8002
MCP_TRANSPORT=http

# LLM Provider (choose one)
LLM_PROVIDER=groq
GROQ_API_KEY=gsk_your_actual_key_here
LLM_MODEL=llama-3.1-8b-instant

# LLM Settings
LLM_TEMPERATURE=0.7
LLM_MAX_TOKENS=2000

# Embeddings (use Google AI Studio)
EMBEDDING_PROVIDER=google
EMBEDDING_MODEL=text-embedding-004
GOOGLE_AI_API_KEY=your_actual_google_key_here

# Database
DATABASE_URL=postgresql://user:password@localhost:5432/beautibuk_agent

# Server
AGENT_PORT=3000
SESSION_TIMEOUT_MINUTES=30
LOG_LEVEL=info

# CORS
ALLOWED_ORIGINS=http://localhost:8080
```

## Important Notes

1. **Never commit `.env` to git** - It's already in `.gitignore`
2. **Use different keys for development and production**
3. **Groq is faster and free** - Good for development
4. **Google AI Studio is better quality** - Good for production
5. **Embeddings always use Google** - Groq doesn't support embeddings

## Verification

After setting up your `.env` file, you can verify it's loaded correctly:

```bash
# Check if environment variables are loaded
cargo run
# Look for error messages about missing API keys
```

The application will fail to start if required API keys are missing, showing a clear error message.

