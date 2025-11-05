mod agent;
mod api;
mod config;
mod database;
mod mcp;
mod models;
mod session;
mod vector;

use anyhow::Result;
use axum::Router;
use tracing::info;

use config::Settings;
use database::get_pool;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting BeautiBuk Agent...");

    // Load configuration
    let settings = Settings::from_env()?;
    info!("Configuration loaded");

    // Initialize database
    let db_pool = get_pool(&settings.database_url).await?;
    info!("Database connection established");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await?;
    info!("Database migrations completed");

    // Initialize services
    let mcp_client = mcp::McpClient::new(settings.mcp_server_url.clone());
    
    // Initialize MCP connection
    mcp_client.initialize().await?;
    info!("MCP client initialized");

    // Initialize LLM client
    let llm_provider = match settings.llm_provider {
        config::LlmProvider::Groq => agent::llm::LlmProvider::Groq,
        config::LlmProvider::Google => agent::llm::LlmProvider::Google,
    };
    
    let llm_client = agent::llm::LlmClient::new(
        llm_provider,
        settings.llm_api_key.clone(),
        settings.llm_model.clone(),
        settings.llm_temperature,
        settings.llm_max_tokens,
    );

    // Initialize embedding service
    let embedding_provider = match settings.embedding_provider {
        config::EmbeddingProvider::Google => agent::embeddings::EmbeddingProvider::Google,
    };
    
    let embedding_service = agent::embeddings::EmbeddingService::new(
        embedding_provider,
        settings.embedding_api_key.clone(),
        settings.embedding_model.clone(),
    );

    // Initialize vector service
    let vector_service = vector::VectorService::new(db_pool.clone());

    // Initialize session manager
    let session_manager = session::SessionManager::new(db_pool.clone());

    // Initialize orchestrator
    let orchestrator = agent::orchestrator::Orchestrator::new(
        llm_client,
        mcp_client,
        session_manager,
        vector_service,
        embedding_service,
    );

    // Build application
    let app = api::create_router(orchestrator);

    // Start server
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", settings.agent_port))
        .await?;
    info!("Server listening on port {}", settings.agent_port);

    axum::serve(listener, app).await?;

    Ok(())
}

