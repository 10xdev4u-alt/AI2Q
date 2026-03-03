use crate::{QueryPlan, Schema, Translator};
use async_openai::{
    types::{ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
    Client,
};
use async_trait::async_trait;

pub struct OpenAITranslator {
    client: Client<async_openai::config::OpenAIConfig>,
    model: String,
}

impl OpenAITranslator {
    pub fn new(api_key: String, model: String) -> Self {
        let config = async_openai::config::OpenAIConfig::new().with_api_key(api_key);
        let client = Client::with_config(config);
        Self { client, model }
    }

    fn build_schema_context(&self, schema: &Schema, prompt: &str) -> String {
        let mut context = String::from("Database Schema:\n");
        let lowercase_prompt = prompt.to_lowercase();
        
        for (table_name, table) in &schema.tables {
            // Context Pruning: Only include tables that appear in the prompt or have foreign keys to tables in the prompt.
            let table_keyword = table_name.to_lowercase();
            let is_relevant = lowercase_prompt.contains(&table_keyword) || 
                             table.columns.iter().any(|c| lowercase_prompt.contains(&c.name.to_lowercase()));
            
            if !is_relevant && schema.tables.len() > 10 {
                continue; // Skip irrelevant tables for huge schemas
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
            // ... (FK and Index code)
        }
        context
    }
}

#[async_trait]
impl Translator for OpenAITranslator {
    async fn translate(&self, prompt: &str, schema: &Schema) -> anyhow::Result<QueryPlan> {
        let schema_context = self.build_schema_context(schema, prompt);
        let system_prompt = format!(
            "You are an expert SQL translator. Convert natural language to SQL based on the schema below.\n\
             Return ONLY a JSON object with 'query' and 'explanation' fields.\n\n{}",
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
        let choice = response.choices.first().ok_or_else(|| anyhow::anyhow!("No response from OpenAI"))?;
        let content = choice.message.content.as_ref().ok_or_else(|| anyhow::anyhow!("Empty response content"))?;

        let parsed: serde_json::Value = serde_json::from_str(content)?;
        let raw_query = parsed["query"].as_str().ok_or_else(|| anyhow::anyhow!("Missing 'query' in response"))?.to_string();
        let explanation = parsed["explanation"].as_str().unwrap_or("").to_string();

        Ok(QueryPlan {
            raw_query,
            explanation,
            cost: None,
        })
    }
}
