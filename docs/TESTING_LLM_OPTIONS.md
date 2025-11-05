# LLM Provider Setup: Groq & Google AI Studio

This document focuses on setting up **Groq** and **Google AI Studio** for the booking agent.

## Primary Providers

### 1. **Google AI Studio / Gemini API** ⭐ (Primary)

**Why it's great for testing:**
- ✅ Generous free tier
- ✅ Easy to get started
- ✅ Good performance
- ✅ Supports embeddings

**Free Tier Limits:**
- **Requests**: 60 requests per minute
- **Tokens**: 300,000 tokens per day
- **Models**: Gemini 2.5 Pro, Gemini 1.5 Flash
- **Valid until**: June 30, 2026 (for verified accounts)

**Setup:**
1. Visit [Google AI Studio](https://aistudio.google.com)
2. Sign in with Google account
3. Go to [API Keys](https://aistudio.google.com/app/apikey)
4. Click "Create API key"
5. Copy the API key

**Configuration:**
```bash
# .env
GOOGLE_AI_API_KEY=your_api_key_here
LLM_PROVIDER=google
LLM_MODEL=gemini-2.0-flash-exp  # or gemini-1.5-pro
```

**Note**: Data in free tier may be used by Google to improve products. For production, consider paid tier for data privacy.

### 2. **Groq API** ⭐ (Primary - Fast & Free Tier)

**Why it's great:**
- ✅ Completely free
- ✅ No API limits
- ✅ Runs locally (privacy)
- ✅ No internet required
- ✅ Multiple models available

**Setup:**
```bash
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Download a model (recommended for agent tasks)
ollama pull llama3.2:3b          # Small, fast
ollama pull mistral:7b            # Better quality
ollama pull llama3.1:8b           # Good balance
ollama pull qwen2.5:7b            # Multilingual support
```

**Configuration:**
```bash
# .env
OLLAMA_BASE_URL=http://localhost:11434
LLM_PROVIDER=ollama
LLM_MODEL=llama3.2:3b  # or mistral:7b
```

**Pros:**
- No API costs
- No rate limits
- Complete privacy
- Works offline

**Cons:**
- Requires local installation
- Uses local resources (CPU/GPU)
- Smaller models may have lower quality than cloud APIs

### 3. **Hugging Face Inference API**

**Free Tier:**
- Limited requests per month
- Free tier available
- Many models available

**Setup:**
1. Create account at [Hugging Face](https://huggingface.co)
2. Get API token from [Settings](https://huggingface.co/settings/tokens)
3. Use Inference API

**Configuration:**
```bash
HUGGINGFACE_API_KEY=your_token_here
LLM_PROVIDER=huggingface
LLM_MODEL=meta-llama/Llama-3.1-8B-Instruct
```

### 4. **Groq API** (Fast & Free Tier)

**Free Tier:**
- Very fast responses (GPU-powered)
- Free tier available
- Good for testing

**Setup:**
1. Sign up at [Groq](https://console.groq.com)
2. Get API key
3. Use in application

**Configuration:**
```bash
GROQ_API_KEY=your_key_here
LLM_PROVIDER=groq
LLM_MODEL=llama-3.1-70b-versatile  # or llama-3.1-8b-instant
```

### 5. **OpenAI (Limited Free Tier)**

**Free Tier:**
- Very limited credits
- Not recommended for extensive testing
- Good API compatibility

**Setup:**
1. Sign up at [OpenAI](https://platform.openai.com)
2. Get API key
3. Use credits (limited)

**Note**: Free tier is very limited. Better for production with paid account.

## Comparison Table

| Provider | Free Tier | Speed | Quality | Privacy | Setup Difficulty |
|----------|-----------|-------|---------|---------|------------------|
| **Google AI Studio** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐ Easy |
| **Ollama (Local)** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ Medium |
| **Hugging Face** | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐ Easy |
| **Groq** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐ Easy |
| **OpenAI** | ⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐ Easy |

## Recommended Setup Strategy

### For Development & Testing

**Use Groq** for:
- Fast iteration (sub-second responses)
- Generous free tier
- Good for rapid development
- Multiple model options

```bash
# In your .env
LLM_PROVIDER=groq
GROQ_API_KEY=your_key_here
LLM_MODEL=llama-3.1-8b-instant  # Fast for testing
```

**Use Google AI Studio** for:
- Better model quality (Gemini)
- Embeddings support
- Production-like testing
- Multimodal capabilities (future)

```bash
# In your .env
LLM_PROVIDER=google
GOOGLE_AI_API_KEY=your_key_here
LLM_MODEL=gemini-2.0-flash-exp  # Fast, good quality
# or
LLM_MODEL=gemini-1.5-pro  # Best quality
```

### Production
Both providers offer paid tiers with higher limits:
- **Groq**: Pay-as-you-go, very affordable ($0.10-0.27 per 1M tokens)
- **Google AI Studio**: Pay-as-you-go, competitive pricing ($0.10-0.50 per 1M tokens)

**Recommendation**: Start with Groq for speed, use Google for quality when needed.

## Implementation: Groq & Google AI Studio

### Rust Configuration

```rust
// src/config/settings.rs
#[derive(Debug, Clone)]
pub enum LlmProvider {
    Google,
    Groq,
}

#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: Option<String>,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl LlmConfig {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        let provider = match std::env::var("LLM_PROVIDER")
            .unwrap_or_else(|_| "groq".to_string())
            .to_lowercase()
            .as_str()
        {
            "google" => LlmProvider::Google,
            "groq" => LlmProvider::Groq,
            _ => LlmProvider::Groq, // Default to Groq
        };

        // Get API key based on provider
        let api_key = match provider {
            LlmProvider::Google => std::env::var("GOOGLE_AI_API_KEY")
                .or_else(|_| std::env::var("GOOGLE_API_KEY"))
                .ok(),
            LlmProvider::Groq => std::env::var("GROQ_API_KEY")
                .or_else(|_| std::env::var("GROQ_KEY"))
                .ok(),
        };

        Ok(LlmConfig {
            provider,
            api_key,
            model: std::env::var("LLM_MODEL")
                .unwrap_or_else(|_| match provider {
                    LlmProvider::Groq => "llama-3.1-8b-instant".to_string(),
                    LlmProvider::Google => "gemini-2.0-flash-exp".to_string(),
                }),
            base_url: None, // Not needed for Groq/Google
            temperature: std::env::var("LLM_TEMPERATURE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.7),
            max_tokens: std::env::var("LLM_MAX_TOKENS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(2000),
        })
    }
}
```

### LLM Client with Multi-Provider Support

```rust
// src/agent/llm.rs
pub struct LlmClient {
    config: LlmConfig,
    http_client: reqwest::Client,
}

impl LlmClient {
    pub async fn generate_response(
        &self,
        messages: &[ChatMessage],
    ) -> Result<String, LlmError> {
        match self.config.provider {
            LlmProvider::Google => self.call_google_gemini(messages).await,
            LlmProvider::Groq => self.call_groq(messages).await,
        }
    }

    async fn call_groq(&self, messages: &[ChatMessage]) -> Result<String, LlmError> {
        let api_key = self.config.api_key
            .as_ref()
            .ok_or_else(|| LlmError::Config("Groq API key not set".to_string()))?;

        let request = serde_json::json!({
            "model": self.config.model,
            "messages": messages.iter().map(|m| {
                json!({
                    "role": m.role,
                    "content": m.content
                })
            }).collect::<Vec<_>>(),
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens,
        });

        let response = self
            .http_client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(LlmError::Api(format!("Groq API error: {}", error_text)));
        }

        let result: GroqResponse = response.json().await?;
        Ok(result.choices[0].message.content.clone())
    }

    async fn call_google_gemini(&self, messages: &[ChatMessage]) -> Result<String, LlmError> {
        let api_key = self.config.api_key
            .as_ref()
            .ok_or_else(|| LlmError::Config("Google API key not set".to_string()))?;

        // Convert messages to Gemini format
        let contents: Vec<serde_json::Value> = messages.iter().map(|m| {
            let role = match m.role.as_str() {
                "user" => "user",
                "assistant" => "model",
                "system" => "user", // Gemini doesn't have system role
                _ => "user",
            };
            json!({
                "role": role,
                "parts": [{"text": m.content}]
            })
        }).collect();

        let request = serde_json::json!({
            "contents": contents,
            "generationConfig": {
                "temperature": self.config.temperature,
                "maxOutputTokens": self.config.max_tokens,
            }
        });

        let response = self
            .http_client
            .post(format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                self.config.model, api_key
            ))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(LlmError::Api(format!("Google API error: {}", error_text)));
        }

        let result: GeminiResponse = response.json().await?;
        
        if let Some(candidate) = result.candidates.first() {
            if let Some(part) = candidate.content.parts.first() {
                return Ok(part.text.clone());
            }
        }
        
        Err(LlmError::Api("No content in Gemini response".to_string()))
    }
}
```

## Environment Configuration Examples

### For Groq (Recommended - Fast & Free)
```bash
# .env
LLM_PROVIDER=groq
GROQ_API_KEY=gsk_your_key_here
LLM_MODEL=llama-3.1-8b-instant
LLM_TEMPERATURE=0.7
LLM_MAX_TOKENS=2000
```

### For Google AI Studio (Quality & Embeddings)
```bash
# .env
LLM_PROVIDER=google
GOOGLE_AI_API_KEY=your_key_here
LLM_MODEL=gemini-2.0-flash-exp
LLM_TEMPERATURE=0.7
LLM_MAX_TOKENS=2000
```

### Switching Between Providers
Simply change `LLM_PROVIDER` in your `.env` file:
```bash
# Use Groq for fast development
LLM_PROVIDER=groq

# Switch to Google for better quality
LLM_PROVIDER=google
```

## Embeddings for Vector Database

### Google AI Studio (Gemini)
- ✅ **Recommended** - Supports embeddings
- ✅ Free tier includes embedding generation
- ✅ Model: `text-embedding-004` or `embedding-001`
- ✅ Good quality embeddings

**Configuration:**
```bash
EMBEDDING_PROVIDER=google
EMBEDDING_MODEL=text-embedding-004
GOOGLE_AI_API_KEY=your_key_here
```

### Groq
- ❌ Does not support embeddings API
- Use Google AI Studio for embeddings
- Or use OpenAI embeddings API (separate)

### Recommendation
Use **Google AI Studio** for embeddings since:
- Free tier includes embedding generation
- Good quality
- Same API key as LLM

## Quick Setup Checklist

### Groq Setup (5 minutes)
- [ ] Sign up at https://console.groq.com
- [ ] Create API key at https://console.groq.com/keys
- [ ] Copy API key (starts with `gsk_...`)
- [ ] Add to `.env`: `GROQ_API_KEY=gsk_your_key_here`
- [ ] Set `LLM_PROVIDER=groq` in `.env`
- [ ] Test API connection

### Google AI Studio Setup (5 minutes)
- [ ] Visit https://aistudio.google.com
- [ ] Sign in with Google account
- [ ] Get API key at https://aistudio.google.com/app/apikey
- [ ] Copy API key
- [ ] Add to `.env`: `GOOGLE_AI_API_KEY=your_key_here`
- [ ] Set `LLM_PROVIDER=google` in `.env` (or use for embeddings)
- [ ] Test API connection

### Testing
- [ ] Test Groq API (fast responses, sub-second latency)
- [ ] Test Google AI Studio (quality responses)
- [ ] Test embeddings with Google AI Studio
- [ ] Compare response quality between providers
- [ ] Test rate limits and error handling
- [ ] Verify embeddings generation works
- [ ] Choose preferred provider for different use cases

## Cost Considerations

### Development Phase:
- **Groq**: $0 (free tier: 14,400 requests/day)
- **Google AI Studio**: $0 (free tier: 60 req/min, 300K tokens/day)
- **Total**: $0

### Production Phase:
- **Groq**: ~$0.10-0.27 per 1M tokens (very affordable)
- **Google AI Studio**: ~$0.10-0.50 per 1M tokens (competitive)
- **Hybrid Approach**: Use Groq for speed, Google for quality-sensitive tasks

## Recommendation

**For Development:**
1. **Start with Groq** - Fast, free tier, good for rapid iteration
2. **Use Google AI Studio** - For embeddings and when you need better quality
3. **Switch easily** - Just change `LLM_PROVIDER` in `.env`

**For Production:**
- **Groq** - Fast, affordable ($0.10-0.27 per 1M tokens)
- **Google AI Studio** - Quality, competitive pricing ($0.10-0.50 per 1M tokens)
- **Hybrid**: Use Groq for speed, Google for quality-sensitive tasks

## Quick Start

### 1. Get Groq API Key (5 minutes)
```bash
# Visit https://console.groq.com
# Sign up and create API key
# Copy key (starts with gsk_...)
```

### 2. Get Google AI Studio Key (5 minutes)
```bash
# Visit https://aistudio.google.com
# Sign in with Google account
# Get API key from https://aistudio.google.com/app/apikey
```

### 3. Configure in .env
```bash
# Start with Groq (fast, free)
LLM_PROVIDER=groq
GROQ_API_KEY=gsk_your_key_here
LLM_MODEL=llama-3.1-8b-instant

# For embeddings, use Google
EMBEDDING_PROVIDER=google
EMBEDDING_MODEL=text-embedding-004
GOOGLE_AI_API_KEY=your_key_here
```

### 4. Test!
- Groq: Fast responses, generous free tier
- Google: Quality responses, embeddings support
- Switch between them easily!

