use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Clone)]
pub enum EmbeddingProvider {
    Google,
}

pub struct EmbeddingService {
    provider: EmbeddingProvider,
    api_key: String,
    model: String,
    client: Client,
}

impl EmbeddingService {
    pub fn new(provider: EmbeddingProvider, api_key: String, model: String) -> Self {
        Self {
            provider,
            api_key,
            model,
            client: Client::new(),
        }
    }

    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        match self.provider {
            EmbeddingProvider::Google => self.generate_google_embedding(text).await,
        }
    }

    async fn generate_google_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let request = json!({
            "model": self.model,
            "content": {
                "parts": [{"text": text}]
            }
        });

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:embedContent?key={}",
            self.model, self.api_key
        );

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!(
                "Google Embeddings API error: {}",
                error_text
            ));
        }

        #[derive(Deserialize)]
        struct EmbeddingResponse {
            embedding: EmbeddingData,
        }

        #[derive(Deserialize)]
        struct EmbeddingData {
            values: Vec<f32>,
        }

        let result: EmbeddingResponse = response.json().await?;
        Ok(result.embedding.values)
    }
}
