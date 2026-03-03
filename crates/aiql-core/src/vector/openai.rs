use crate::EmbeddingEngine;
use async_openai::{
    types::{CreateEmbeddingRequestArgs},
    Client,
};
use async_trait::async_trait;

pub struct OpenAIEmbeddingEngine {
    client: Client<async_openai::config::OpenAIConfig>,
    model: String,
}

impl OpenAIEmbeddingEngine {
    pub fn new(api_key: String, model: String) -> Self {
        let config = async_openai::config::OpenAIConfig::new().with_api_key(api_key);
        let client = Client::with_config(config);
        Self { client, model }
    }
}

#[async_trait]
impl EmbeddingEngine for OpenAIEmbeddingEngine {
    async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let request = CreateEmbeddingRequestArgs::default()
            .model(&self.model)
            .input(text)
            .build()?;

        let response = self.client.embeddings().create(request).await?;
        let data = response.data.first().ok_or_else(|| anyhow::anyhow!("No embedding data returned"))?;
        Ok(data.embedding.clone())
    }
}
