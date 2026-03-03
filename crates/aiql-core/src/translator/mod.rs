pub mod mock;
pub mod openai;
pub mod ollama;

pub use mock::MockTranslator;
pub use openai::OpenAITranslator;
pub use ollama::OllamaTranslator;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::Translator;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider", content = "config")]
pub enum TranslatorConfig {
    Mock,
    OpenAI { api_key: String, model: String, temperature: Option<f32> },
    Ollama { host: String, port: u16, model: String },
}

pub fn create_translator(config: TranslatorConfig) -> Arc<dyn Translator + Send + Sync> {
    match config {
        TranslatorConfig::Mock => Arc::new(MockTranslator),
        TranslatorConfig::OpenAI { api_key, model, temperature } => {
            let mut t = OpenAITranslator::new(api_key, model);
            if let Some(temp) = temperature {
                t = t.with_temperature(temp);
            }
            Arc::new(t)
        }
        TranslatorConfig::Ollama { host, port, model } => {
            Arc::new(OllamaTranslator::new(host, port, model))
        }
    }
}
