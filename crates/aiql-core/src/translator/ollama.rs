use crate::{QueryPlan, Schema, Translator};
use async_trait::async_trait;
use ollama_rs::Ollama;
use ollama_rs::generation::chat::{ChatMessage, request::ChatMessageRequest};

pub struct OllamaTranslator {
    client: Ollama,
    model: String,
}

impl OllamaTranslator {
    pub fn new(host: String, port: u16, model: String) -> Self {
        let client = Ollama::new(host, port);
        Self { client, model }
    }

    fn build_schema_context(&self, schema: &Schema) -> String {
        let mut context = String::from("Database Schema:\n");
        for (table_name, table) in &schema.tables {
            context.push_str(&format!("Table: {}\n", table_name));
            for col in &table.columns {
                context.push_str(&format!("  - {} ({})\n", col.name, col.data_type));
            }
        }
        context
    }
}

#[async_trait]
impl Translator for OllamaTranslator {
    async fn translate(&self, prompt: &str, schema: &Schema, dialect: crate::DatabaseDialect) -> anyhow::Result<QueryPlan> {
        let schema_context = self.build_schema_context(schema);
        let system_prompt = format!(
            "You are an expert SQL/NoSQL translator. Convert natural language to {} based on the schema below.\n\
             Return ONLY a JSON object with 'query' and 'explanation' fields.\n\n{}",
            match dialect {
                crate::DatabaseDialect::MongoDB => "MongoDB Aggregation Pipeline JSON",
                crate::DatabaseDialect::Postgres => "PostgreSQL",
                crate::DatabaseDialect::MySQL => "MySQL",
                crate::DatabaseDialect::SQLite => "SQLite",
            },
            schema_context
        );

        let messages = vec![
            ChatMessage::system(system_prompt),
            ChatMessage::user(prompt.to_string()),
        ];

        let res = self.client.send_chat_messages(ChatMessageRequest::new(self.model.clone(), messages)).await?;
        let content = res.message.content;

        // Simple JSON extraction for Ollama models that might not support structured output perfectly
        let re = regex::Regex::new(r"\{[\s\S]*\}")?;
        let json_str = if let Some(m) = re.find(&content) {
            m.as_str()
        } else {
            return Err(anyhow::anyhow!("Failed to find JSON in Ollama response: {}", content));
        };

        let parsed: serde_json::Value = serde_json::from_str(json_str)?;
        let raw_query = parsed["query"].as_str().ok_or_else(|| anyhow::anyhow!("Missing 'query'"))?.to_string();
        let explanation = parsed["explanation"].as_str().unwrap_or("").to_string();

        Ok(QueryPlan {
            dialect,
            raw_query,
            explanation,
            cost: None,
        })
    }

    async fn translate_migration(&self, prompt: &str, schema: &Schema, dialect: crate::DatabaseDialect) -> anyhow::Result<crate::MigrationPlan> {
        let schema_context = self.build_schema_context(schema);
        let system_prompt = format!(
            "You are an expert database architect. Convert natural language migration requests to {} DDL.\n\
             Return ONLY a JSON object with 'sql' and 'explanation' fields.\n\n{}",
            match dialect {
                crate::DatabaseDialect::MongoDB => "MongoDB Collection operations",
                crate::DatabaseDialect::Postgres => "PostgreSQL",
                crate::DatabaseDialect::MySQL => "MySQL",
                crate::DatabaseDialect::SQLite => "SQLite",
            },
            schema_context
        );

        let messages = vec![
            ChatMessage::system(system_prompt),
            ChatMessage::user(prompt.to_string()),
        ];

        let res = self.client.send_chat_messages(ChatMessageRequest::new(self.model.clone(), messages)).await?;
        let content = res.message.content;

        let re = regex::Regex::new(r"\{[\s\S]*\}")?;
        let json_str = if let Some(m) = re.find(&content) {
            m.as_str()
        } else {
            return Err(anyhow::anyhow!("Failed to find JSON in Ollama response"));
        };

        let parsed: serde_json::Value = serde_json::from_str(json_str)?;
        let raw_sql = parsed["sql"].as_str().ok_or_else(|| anyhow::anyhow!("Missing 'sql'"))?.to_string();
        let explanation = parsed["explanation"].as_str().unwrap_or("").to_string();

        Ok(crate::MigrationPlan {
            dialect,
            raw_sql,
            explanation,
        })
    }

    async fn translate_vector(&self, prompt: &str, schema: &Schema, dialect: crate::DatabaseDialect) -> anyhow::Result<QueryPlan> {
        let schema_context = self.build_schema_context(schema);
        let system_prompt = format!(
            "You are an expert SQL/NoSQL translator specializing in Vector Search. Convert natural language to {} with vector operators.\n\
             Use '$VECTOR' as a placeholder for the generated embedding vector.\n\
             Return ONLY a JSON object with 'query' and 'explanation' fields.\n\n{}",
            match dialect {
                crate::DatabaseDialect::MongoDB => "MongoDB Aggregation Pipeline JSON",
                crate::DatabaseDialect::Postgres => "PostgreSQL (with pgvector)",
                crate::DatabaseDialect::MySQL => "MySQL (with vector extensions)",
                crate::DatabaseDialect::SQLite => "SQLite (with vector extensions)",
            },
            schema_context
        );

        let messages = vec![
            ChatMessage::system(system_prompt),
            ChatMessage::user(prompt.to_string()),
        ];

        let res = self.client.send_chat_messages(ChatMessageRequest::new(self.model.clone(), messages)).await?;
        let content = res.message.content;

        let re = regex::Regex::new(r"\{[\s\S]*\}")?;
        let json_str = if let Some(m) = re.find(&content) {
            m.as_str()
        } else {
            return Err(anyhow::anyhow!("Failed to find JSON in Ollama response"));
        };

        let parsed: serde_json::Value = serde_json::from_str(json_str)?;
        let raw_query = parsed["query"].as_str().ok_or_else(|| anyhow::anyhow!("Missing 'query'"))?.to_string();
        let explanation = parsed["explanation"].as_str().unwrap_or("").to_string();

        Ok(QueryPlan {
            dialect,
            raw_query,
            explanation,
            cost: None,
        })
    }
}
