use crate::{QueryPlan, Schema, Translator, TranslateResult};
use async_openai::{
    types::{ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
    Client,
};
use async_trait::async_trait;

pub struct OpenAITranslator {
    client: Client<async_openai::config::OpenAIConfig>,
    model: String,
    temperature: f32,
}

impl OpenAITranslator {
    pub fn new(api_key: String, model: String) -> Self {
        let config = async_openai::config::OpenAIConfig::new().with_api_key(api_key);
        let client = Client::with_config(config);
        Self { client, model, temperature: 0.0 }
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    fn build_schema_context(&self, schema: &Schema, prompt: &str) -> String {
        let mut context = String::from("Database Schema:\n");
        let lowercase_prompt = prompt.to_lowercase();
        
        for (table_name, table) in &schema.tables {
            let table_keyword = table_name.to_lowercase();
            let is_relevant = lowercase_prompt.contains(&table_keyword) || 
                             table.columns.iter().any(|c| lowercase_prompt.contains(&c.name.to_lowercase()));
            
            if !is_relevant && schema.tables.len() > 10 {
                continue;
            }

            context.push_str(&format!("Table: {}\n", table_name));
            for col in &table.columns {
                let pk = if col.is_primary_key { " (PK)" } else { "" };
                let nullable = if col.is_nullable { "" } else { " NOT NULL" };
                context.push_str(&format!(
                    "  - {} {} {}{}\n",
                    col.name,
                    col.data_type,
                    nullable,
                    pk
                ));
            }
            if !table.foreign_keys.is_empty() {
                context.push_str("  Foreign Keys:\n");
                for fk in &table.foreign_keys {
                    context.push_str(&format!(
                        "    - {} references {}({})\n",
                        fk.column_name,
                        fk.foreign_table,
                        fk.foreign_column
                    ));
                }
            }
            if !table.indexes.is_empty() {
                context.push_str("  Indexes:\n");
                for idx in &table.indexes {
                    let unique = if idx.is_unique { " (UNIQUE)" } else { "" };
                    context.push_str(&format!(
                        "    - {} on ({}){}\n",
                        idx.name,
                        idx.columns.join(", "),
                        unique
                    ));
                }
            }
        }
        context
    }
}

#[async_trait]
impl Translator for OpenAITranslator {
    async fn translate(&self, prompt: &str, schema: &Schema, dialect: crate::DatabaseDialect, context: &crate::Context, session: Option<&crate::Session>) -> anyhow::Result<TranslateResult> {
        let schema_context = self.build_schema_context(schema, prompt);
        let system_prompt = format!(
            "You are an expert SQL/NoSQL translator. Convert natural language to {} based on the schema below.\n\
             Current Time: {}\n\
             If the prompt is ambiguous or lacks enough information, return a clarification request.\n\
             Return ONLY a JSON object.\n\
             For successful translation: {{ \"type\": \"plan\", \"query\": \"...\", \"explanation\": \"...\" }}\n\
             For ambiguity: {{ \"type\": \"clarification\", \"reason\": \"...\", \"suggestions\": [\"...\", \"...\"] }}\n\n{}",
            match dialect {
                crate::DatabaseDialect::MongoDB => "MongoDB Aggregation Pipeline JSON",
                crate::DatabaseDialect::Postgres => "PostgreSQL",
                crate::DatabaseDialect::MySQL => "MySQL",
                crate::DatabaseDialect::SQLite => "SQLite",
            },
            context.now,
            schema_context
        );

        let mut messages = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_prompt)
                .build()?
                .into(),
        ];

        if let Some(sess) = session {
            for msg in &sess.history {
                if msg.role == "user" {
                    messages.push(ChatCompletionRequestUserMessageArgs::default().content(&msg.content).build()?.into());
                }
            }
        }

        messages.push(
            ChatCompletionRequestUserMessageArgs::default()
                .content(prompt)
                .build()?
                .into()
        );

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .temperature(self.temperature)
            .messages(messages)
            .response_format(async_openai::types::ResponseFormat::JsonObject)
            .build()?;

        let response = self.client.chat().create(request).await?;
        let choice = response.choices.first().ok_or_else(|| anyhow::anyhow!("No response from OpenAI"))?;
        let content = choice.message.content.as_ref().ok_or_else(|| anyhow::anyhow!("Empty response content"))?;

        let parsed: serde_json::Value = serde_json::from_str(content)?;
        let result_type = parsed["type"].as_str().unwrap_or("plan");

        if result_type == "clarification" {
            let reason = parsed["reason"].as_str().unwrap_or("Ambiguous prompt").to_string();
            let suggestions = parsed["suggestions"].as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();
            Ok(TranslateResult::ClarificationNeeded { reason, suggestions })
        } else {
            let raw_query = parsed["query"].as_str().ok_or_else(|| anyhow::anyhow!("Missing 'query' in response"))?.to_string();
            let explanation = parsed["explanation"].as_str().unwrap_or("").to_string();

            Ok(TranslateResult::Plan(QueryPlan {
                dialect,
                raw_query,
                explanation,
                cost: None,
            }))
        }
    }

    async fn translate_migration(&self, prompt: &str, schema: &Schema, dialect: crate::DatabaseDialect) -> anyhow::Result<crate::MigrationPlan> {
        let schema_context = self.build_schema_context(schema, prompt);
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

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .temperature(self.temperature)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(system_prompt)
                    .build()?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(prompt)
                    .build()?
                    .into(),
            ])
            .response_format(async_openai::types::ResponseFormat::JsonObject)
            .build()?;

        let response = self.client.chat().create(request).await?;
        let choice = response.choices.first().ok_or_else(|| anyhow::anyhow!("No response from OpenAI"))?;
        let content = choice.message.content.as_ref().ok_or_else(|| anyhow::anyhow!("Empty response content"))?;

        let parsed: serde_json::Value = serde_json::from_str(content)?;
        let raw_sql = parsed["sql"].as_str().ok_or_else(|| anyhow::anyhow!("Missing 'sql' in response"))?.to_string();
        let explanation = parsed["explanation"].as_str().unwrap_or("").to_string();

        Ok(crate::MigrationPlan {
            dialect,
            raw_sql,
            explanation,
        })
    }

    async fn translate_vector(&self, prompt: &str, schema: &Schema, dialect: crate::DatabaseDialect, context: &crate::Context, session: Option<&crate::Session>) -> anyhow::Result<TranslateResult> {
        let schema_context = self.build_schema_context(schema, prompt);
        let system_prompt = format!(
            "You are an expert SQL/NoSQL translator specializing in Vector Search. Convert natural language to {} with vector operators.\n\
             Current Time: {}\n\
             Use '$VECTOR' as a placeholder for the generated embedding vector.\n\
             Return ONLY a JSON object with 'query' and 'explanation' fields.\n\n{}",
            match dialect {
                crate::DatabaseDialect::MongoDB => "MongoDB Aggregation Pipeline JSON",
                crate::DatabaseDialect::Postgres => "PostgreSQL (with pgvector)",
                crate::DatabaseDialect::MySQL => "MySQL (with vector extensions)",
                crate::DatabaseDialect::SQLite => "SQLite (with vector extensions)",
            },
            context.now,
            schema_context
        );

        let mut messages = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_prompt)
                .build()?
                .into(),
        ];

        if let Some(sess) = session {
            for msg in &sess.history {
                if msg.role == "user" {
                    messages.push(ChatCompletionRequestUserMessageArgs::default().content(&msg.content).build()?.into());
                }
            }
        }

        messages.push(
            ChatCompletionRequestUserMessageArgs::default()
                .content(prompt)
                .build()?
                .into()
        );

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .temperature(self.temperature)
            .messages(messages)
            .response_format(async_openai::types::ResponseFormat::JsonObject)
            .build()?;

        let response = self.client.chat().create(request).await?;
        let choice = response.choices.first().ok_or_else(|| anyhow::anyhow!("No response from OpenAI"))?;
        let content = choice.message.content.as_ref().ok_or_else(|| anyhow::anyhow!("Empty response content"))?;

        let parsed: serde_json::Value = serde_json::from_str(content)?;
        let raw_query = parsed["query"].as_str().ok_or_else(|| anyhow::anyhow!("Missing 'query' in response"))?.to_string();
        let explanation = parsed["explanation"].as_str().unwrap_or("").to_string();

        Ok(TranslateResult::Plan(QueryPlan {
            dialect,
            raw_query,
            explanation,
            cost: None,
        }))
    }
}

#[async_trait]
impl crate::MockDataGenerator for OpenAITranslator {
    async fn generate_mock_data(&self, prompt: &str, schema: &Schema, dialect: crate::DatabaseDialect) -> anyhow::Result<Vec<String>> {
        let schema_context = self.build_schema_context(schema, prompt);
        let system_prompt = format!(
            "You are an expert data engineer. Generate realistic synthetic data for {} based on the schema below.\n\
             Return ONLY a JSON object with a 'queries' field containing an array of INSERT/update statements.\n\n{}",
            match dialect {
                crate::DatabaseDialect::MongoDB => "MongoDB",
                _ => "SQL",
            },
            schema_context
        );

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(system_prompt)
                    .build()?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(prompt)
                    .build()?
                    .into(),
            ])
            .response_format(async_openai::types::ResponseFormat::JsonObject)
            .build()?;

        let response = self.client.chat().create(request).await?;
        let choice = response.choices.first().ok_or_else(|| anyhow::anyhow!("No response"))?;
        let content = choice.message.content.as_ref().ok_or_else(|| anyhow::anyhow!("Empty content"))?;

        let parsed: serde_json::Value = serde_json::from_str(content)?;
        let queries = parsed["queries"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        Ok(queries)
    }
}
