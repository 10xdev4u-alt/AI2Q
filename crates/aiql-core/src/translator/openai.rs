use crate::{QueryPlan, Schema, Translator, TranslateResult, TypeGenerator, Refactorer, Advisor, MockDataGenerator, QueryHealer, Context, Session, DatabaseDialect};
use async_openai::{
    types::{ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
    Client,
};
use async_trait::async_trait;
use std::collections::HashMap;

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

    fn prune_schema(&self, schema: &Schema, prompt: &str) -> Schema {
        let lowercase_prompt = prompt.to_lowercase();
        let mut pruned_tables = HashMap::new();

        for (name, table) in &schema.tables {
            let name_lower = name.to_lowercase();
            let mut is_relevant = lowercase_prompt.contains(&name_lower);
            
            if !is_relevant {
                is_relevant = table.columns.iter().any(|c| lowercase_prompt.contains(&c.name.to_lowercase()));
            }

            if is_relevant {
                pruned_tables.insert(name.clone(), table.clone());
            }
        }

        let mut to_add = Vec::new();
        for table in pruned_tables.values() {
            for fk in &table.foreign_keys {
                if !pruned_tables.contains_key(&fk.foreign_table) {
                    if let Some(foreign_table) = schema.tables.get(&fk.foreign_table) {
                        to_add.push((fk.foreign_table.clone(), foreign_table.clone()));
                    }
                }
            }
        }

        for (name, table) in to_add {
            pruned_tables.insert(name, table);
        }

        Schema {
            version: schema.version.clone(),
            created_at: schema.created_at,
            tables: pruned_tables,
        }
    }

    fn build_schema_context(&self, schema: &Schema, prompt: &str) -> String {
        let pruned_schema = self.prune_schema(schema, prompt);
        let mut context = String::from("Database Schema:\n");
        
        for (table_name, table) in &pruned_schema.tables {
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
    async fn translate(&self, prompt: &str, schema: &Schema, dialect: DatabaseDialect, context: &Context, session: Option<&Session>, stream: bool) -> anyhow::Result<TranslateResult> {
        let schema_context = self.build_schema_context(schema, prompt);
        let system_prompt = format!(
            "You are an expert SQL/NoSQL translator. Convert natural language to {} based on the schema below.\n\
             Current Time: {}\n\
             Important: Use {} for date/time math.\n\
             Streaming: {}\n\
             If streaming is true, generate a cursor-based query or one suitable for large datasets.\n\
             If the prompt is ambiguous, return a clarification request.\n\
             Return ONLY a JSON object.\n\n{}",
            match dialect {
                DatabaseDialect::MongoDB => "MongoDB Aggregation Pipeline JSON",
                DatabaseDialect::Postgres => "PostgreSQL",
                DatabaseDialect::MySQL => "MySQL",
                DatabaseDialect::SQLite => "SQLite",
                DatabaseDialect::Supabase => "PostgreSQL (Supabase optimized)",
                DatabaseDialect::PostgREST => "Supabase JavaScript client (PostgREST)",
            },
            context.now,
            match dialect {
                DatabaseDialect::Postgres => "Postgres INTERVAL syntax",
                DatabaseDialect::MySQL => "MySQL DATE_SUB/DATE_ADD",
                DatabaseDialect::SQLite => "SQLite date functions",
                DatabaseDialect::MongoDB => "MongoDB date operators",
                DatabaseDialect::Supabase => "Postgres INTERVAL syntax",
                DatabaseDialect::PostgREST => "JavaScript Date objects",
            },
            stream,
            schema_context
        );

        let mut messages = vec![
            ChatCompletionRequestSystemMessageArgs::default().content(system_prompt).build()?.into(),
        ];

        if let Some(sess) = session {
            for msg in &sess.history {
                if msg.role == "user" {
                    messages.push(ChatCompletionRequestUserMessageArgs::default().content(&msg.content).build()?.into());
                }
            }
        }

        messages.push(ChatCompletionRequestUserMessageArgs::default().content(prompt).build()?.into());

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .temperature(self.temperature)
            .messages(messages)
            .response_format(async_openai::types::ResponseFormat::JsonObject)
            .build()?;

        let response = self.client.chat().create(request).await?;
        let choice = response.choices.first().ok_or_else(|| anyhow::anyhow!("No response"))?;
        let content = choice.message.content.as_ref().ok_or_else(|| anyhow::anyhow!("Empty response"))?;

        let parsed: serde_json::Value = serde_json::from_str(content)?;
        let result_type = parsed["type"].as_str().unwrap_or("plan");

        if result_type == "clarification" {
            let reason = parsed["reason"].as_str().unwrap_or("Ambiguous prompt").to_string();
            let suggestions = parsed["suggestions"].as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();
            Ok(TranslateResult::ClarificationNeeded { reason, suggestions })
        } else {
            let raw_query = parsed["query"].as_str().ok_or_else(|| anyhow::anyhow!("Missing 'query'"))?.to_string();
            let explanation = parsed["explanation"].as_str().unwrap_or("").to_string();

            Ok(TranslateResult::Plan(QueryPlan { dialect, raw_query, explanation, cost: None }))
        }
    }

    async fn translate_migration(&self, prompt: &str, schema: &Schema, dialect: DatabaseDialect) -> anyhow::Result<crate::MigrationPlan> {
        let schema_context = self.build_schema_context(schema, prompt);
        let system_prompt = format!(
            "You are an expert database architect. Convert natural language migration requests to {} DDL.\n\n{}",
            match dialect {
                DatabaseDialect::MongoDB => "MongoDB operations",
                _ => "SQL",
            },
            schema_context
        );

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default().content(system_prompt).build()?.into(),
                ChatCompletionRequestUserMessageArgs::default().content(prompt).build()?.into(),
            ])
            .response_format(async_openai::types::ResponseFormat::JsonObject)
            .build()?;

        let response = self.client.chat().create(request).await?;
        let choice = response.choices.first().ok_or_else(|| anyhow::anyhow!("No response"))?;
        let content = choice.message.content.as_ref().ok_or_else(|| anyhow::anyhow!("Empty content"))?;

        let parsed: serde_json::Value = serde_json::from_str(content)?;
        let raw_sql = parsed["sql"].as_str().ok_or_else(|| anyhow::anyhow!("Missing 'sql'"))?.to_string();
        let explanation = parsed["explanation"].as_str().unwrap_or("").to_string();

        Ok(crate::MigrationPlan { dialect, raw_sql, explanation })
    }

    async fn translate_vector(&self, prompt: &str, schema: &Schema, dialect: DatabaseDialect, context: &Context, session: Option<&Session>, stream: bool) -> anyhow::Result<TranslateResult> {
        let schema_context = self.build_schema_context(schema, prompt);
        let system_prompt = format!(
            "You are an expert SQL/NoSQL translator specializing in Vector Search. Convert natural language to {} with vector operators.\n\
             Current Time: {}\n\
             Use '$VECTOR' as a placeholder for the generated embedding vector.\n\
             Streaming: {}\n\n{}",
            match dialect {
                DatabaseDialect::MongoDB => "MongoDB Pipeline JSON",
                _ => "SQL (with vector extensions)",
            },
            context.now,
            stream,
            schema_context
        );

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default().content(system_prompt).build()?.into(),
                ChatCompletionRequestUserMessageArgs::default().content(prompt).build()?.into(),
            ])
            .response_format(async_openai::types::ResponseFormat::JsonObject)
            .build()?;

        let response = self.client.chat().create(request).await?;
        let choice = response.choices.first().ok_or_else(|| anyhow::anyhow!("No response"))?;
        let content = choice.message.content.as_ref().ok_or_else(|| anyhow::anyhow!("Empty content"))?;

        let parsed: serde_json::Value = serde_json::from_str(content)?;
        let raw_query = parsed["query"].as_str().ok_or_else(|| anyhow::anyhow!("Missing 'query'"))?.to_string();
        let explanation = parsed["explanation"].as_str().unwrap_or("").to_string();

        Ok(TranslateResult::Plan(QueryPlan { dialect, raw_query, explanation, cost: None }))
    }
}

#[async_trait]
impl QueryHealer for OpenAITranslator {
    async fn heal(&self, query: &str, error: &str, schema: &Schema, dialect: DatabaseDialect, context: &Context) -> anyhow::Result<QueryPlan> {
        let schema_context = self.build_schema_context(schema, query);
        let system_prompt = format!(
            "You are an expert SQL/NoSQL debugger. Fix the broken query below.\n\
             Error: {}\n\
             Current Time: {}\n\n{}",
            error, context.now, schema_context
        );

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default().content(system_prompt).build()?.into(),
                ChatCompletionRequestUserMessageArgs::default().content(format!("Broken Query: {}\nError: {}", query, error)).build()?.into(),
            ])
            .response_format(async_openai::types::ResponseFormat::JsonObject)
            .build()?;

        let response = self.client.chat().create(request).await?;
        let choice = response.choices.first().ok_or_else(|| anyhow::anyhow!("No response"))?;
        let content = choice.message.content.as_ref().ok_or_else(|| anyhow::anyhow!("Empty content"))?;

        let parsed: serde_json::Value = serde_json::from_str(content)?;
        let fixed_query = parsed["query"].as_str().ok_or_else(|| anyhow::anyhow!("Missing query"))?.to_string();
        let explanation = parsed["explanation"].as_str().unwrap_or("").to_string();

        Ok(QueryPlan { dialect, raw_query: fixed_query, explanation, cost: None })
    }
}

#[async_trait]
impl Advisor for OpenAITranslator {
    async fn advise(&self, plan: &QueryPlan, schema: &Schema) -> anyhow::Result<Vec<String>> {
        let schema_context = self.build_schema_context(schema, &plan.raw_query);
        let system_prompt = format!(
            "You are an expert database performance tuner. Analyze the query and schema and suggest optimizations.\n\n{}",
            schema_context
        );

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default().content(system_prompt).build()?.into(),
                ChatCompletionRequestUserMessageArgs::default().content(format!("Query: {}", plan.raw_query)).build()?.into(),
            ])
            .response_format(async_openai::types::ResponseFormat::JsonObject)
            .build()?;

        let response = self.client.chat().create(request).await?;
        let choice = response.choices.first().ok_or_else(|| anyhow::anyhow!("No response"))?;
        let content = choice.message.content.as_ref().ok_or_else(|| anyhow::anyhow!("Empty content"))?;

        let parsed: serde_json::Value = serde_json::from_str(content)?;
        let advice = parsed["advice"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        Ok(advice)
    }
}

#[async_trait]
impl MockDataGenerator for OpenAITranslator {
    async fn generate_mock_data(&self, prompt: &str, schema: &Schema, dialect: DatabaseDialect) -> anyhow::Result<Vec<String>> {
        let schema_context = self.build_schema_context(schema, prompt);
        let system_prompt = format!(
            "You are an expert data engineer. Generate realistic synthetic data.\n\n{}",
            schema_context
        );

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default().content(system_prompt).build()?.into(),
                ChatCompletionRequestUserMessageArgs::default().content(prompt).build()?.into(),
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

#[async_trait]
impl Refactorer for OpenAITranslator {
    async fn refactor(&self, code: &str) -> anyhow::Result<String> {
        let system_prompt = "You are an expert code refactorer. Find manual SQL strings and replace them with ai(\"...\") calls.";

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default().content(system_prompt).build()?.into(),
                ChatCompletionRequestUserMessageArgs::default().content(code).build()?.into(),
            ])
            .build()?;

        let response = self.client.chat().create(request).await?;
        let choice = response.choices.first().ok_or_else(|| anyhow::anyhow!("No response"))?;
        let content = choice.message.content.as_ref().ok_or_else(|| anyhow::anyhow!("Empty content"))?;

        Ok(content.clone())
    }
}

#[async_trait]
impl crate::DocGenerator for OpenAITranslator {
    async fn generate_docs(&self, schema: &Schema) -> anyhow::Result<String> {
        let schema_context = self.build_schema_context(schema, "generate documentation");
        let system_prompt = "You are an expert technical writer. Generate a comprehensive Markdown documentation site for the database schema provided.\n\
                             Include table descriptions, column details, and relationship diagrams (in mermaid syntax).";

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default().content(system_prompt).build()?.into(),
                ChatCompletionRequestUserMessageArgs::default().content(schema_context).build()?.into(),
            ])
            .build()?;

        let response = self.client.chat().create(request).await?;
        let choice = response.choices.first().ok_or_else(|| anyhow::anyhow!("No response"))?;
        let content = choice.message.content.as_ref().ok_or_else(|| anyhow::anyhow!("Empty content"))?;

        Ok(content.clone())
    }
}

#[async_trait]
impl TypeGenerator for OpenAITranslator {
    async fn generate_types(&self, name: &str, plan: &QueryPlan, schema: &Schema, language: &str) -> anyhow::Result<String> {
        let system_prompt = format!(
            "You are an expert software architect. Generate a type-safe {} container named '{}' for the result of the query.",
            language, name
        );

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default().content(system_prompt).build()?.into(),
                ChatCompletionRequestUserMessageArgs::default().content(format!("Query: {}\nSchema Context: {:?}", plan.raw_query, schema)).build()?.into(),
            ])
            .build()?;

        let response = self.client.chat().create(request).await?;
        let choice = response.choices.first().ok_or_else(|| anyhow::anyhow!("No response"))?;
        let content = choice.message.content.as_ref().ok_or_else(|| anyhow::anyhow!("Empty content"))?;

        Ok(content.clone())
    }
}
