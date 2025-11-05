use anyhow::{anyhow, Result};
use std::env;

#[derive(Debug, Clone)]
pub enum LlmProvider {
    Groq,
    Google,
}

#[derive(Debug, Clone)]
pub enum EmbeddingProvider {
    Google,
}

#[derive(Debug, Clone)]
pub struct Settings {
    // MCP Server
    pub mcp_server_url: String,
    #[allow(dead_code)]
    pub mcp_transport: String,

    // LLM
    pub llm_provider: LlmProvider,
    pub llm_api_key: String,
    pub llm_model: String,
    pub llm_temperature: f32,
    pub llm_max_tokens: u32,

    // Embeddings
    pub embedding_provider: EmbeddingProvider,
    pub embedding_api_key: String,
    pub embedding_model: String,

    // Database
    pub database_url: String,

    // Server
    pub agent_port: u16,
    #[allow(dead_code)]
    pub session_timeout_minutes: u64,
    #[allow(dead_code)]
    pub log_level: String,

    // CORS
    #[allow(dead_code)]
    pub allowed_origins: Vec<String>,
}

impl Settings {
    pub fn from_env() -> Result<Self> {
        let llm_provider = match env::var("LLM_PROVIDER")
            .unwrap_or_else(|_| "groq".to_string())
            .to_lowercase()
            .as_str()
        {
            "google" => LlmProvider::Google,
            "groq" => LlmProvider::Groq,
            _ => LlmProvider::Groq,
        };

        let llm_api_key = match llm_provider {
            LlmProvider::Google => env::var("GOOGLE_AI_API_KEY")
                .or_else(|_| env::var("GOOGLE_API_KEY"))
                .map_err(|_| anyhow!("GOOGLE_AI_API_KEY not set"))?,
            LlmProvider::Groq => env::var("GROQ_API_KEY")
                .or_else(|_| env::var("GROQ_KEY"))
                .map_err(|_| anyhow!("GROQ_API_KEY not set"))?,
        };

        let embedding_provider = match env::var("EMBEDDING_PROVIDER")
            .unwrap_or_else(|_| "google".to_string())
            .to_lowercase()
            .as_str()
        {
            "google" => EmbeddingProvider::Google,
            _ => EmbeddingProvider::Google,
        };

        let embedding_api_key = match embedding_provider {
            EmbeddingProvider::Google => env::var("GOOGLE_AI_API_KEY")
                .or_else(|_| env::var("GOOGLE_API_KEY"))
                .map_err(|_| anyhow!("GOOGLE_AI_API_KEY not set for embeddings"))?,
        };

        let default_llm_model = match llm_provider {
            LlmProvider::Groq => "llama-3.1-8b-instant".to_string(),
            LlmProvider::Google => "gemini-2.0-flash-exp".to_string(),
        };

        let allowed_origins = env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:8080".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        Ok(Settings {
            mcp_server_url: env::var("MCP_SERVER_URL")
                .unwrap_or_else(|_| "http://localhost:8002".to_string()),
            mcp_transport: env::var("MCP_TRANSPORT").unwrap_or_else(|_| "http".to_string()),
            llm_provider,
            llm_api_key,
            llm_model: env::var("LLM_MODEL").unwrap_or(default_llm_model),
            llm_temperature: env::var("LLM_TEMPERATURE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.7),
            llm_max_tokens: env::var("LLM_MAX_TOKENS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(2000),
            embedding_provider,
            embedding_api_key,
            embedding_model: env::var("EMBEDDING_MODEL")
                .unwrap_or_else(|_| "text-embedding-004".to_string()),
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgresql://user:password@localhost:5432/beautibuk_agent".to_string()
            }),
            agent_port: env::var("AGENT_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3000),
            session_timeout_minutes: env::var("SESSION_TIMEOUT_MINUTES")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            allowed_origins,
        })
    }
}
