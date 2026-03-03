use crate::{ExecutionEngine, ExecutionResult, QueryHealer, Schema, Translator, EmbeddingEngine, SemanticCache, PrivacyGuard};

pub struct SmartClient<T, E, H, V, C, P>
where
    T: Translator,
    E: ExecutionEngine,
    H: QueryHealer,
    V: EmbeddingEngine,
    C: SemanticCache,
    P: PrivacyGuard,
{
    translator: T,
    engine: E,
    healer: H,
    embedder: V,
    cache: C,
    privacy: P,
}

impl<T, E, H, V, C, P> SmartClient<T, E, H, V, C, P>
where
    T: Translator,
    E: ExecutionEngine,
    H: QueryHealer,
    V: EmbeddingEngine,
    C: SemanticCache,
    P: PrivacyGuard,
{
    pub fn new(translator: T, engine: E, healer: H, embedder: V, cache: C, privacy: P) -> Self {
        Self {
            translator,
            engine,
            healer,
            embedder,
            cache,
            privacy,
        }
    }

    pub async fn ask(&self, prompt: &str, schema: &Schema, dialect: crate::DatabaseDialect, session: Option<&crate::Session>) -> anyhow::Result<ExecutionResult> {
        log::info!("AIQL: Received prompt: '{}'", prompt);

        // 1. Privacy Scrubbing
        let scrubbed_prompt = self.privacy.scrub_prompt(prompt).await?;
        if scrubbed_prompt != prompt {
            log::info!("AIQL: PII detected and scrubbed from prompt");
        }

        // 2. Semantic Cache Lookup
        log::debug!("AIQL: Checking semantic cache...");
        let embedding = self.embedder.embed(&scrubbed_prompt).await?;
        if let Some(cached_plan) = self.cache.get(&embedding).await? {
            log::info!("AIQL: Semantic cache hit!");
            let mut result = self.engine.execute(&cached_plan.raw_query).await?;
            if let Some(data) = result.data {
                result.data = Some(self.privacy.mask_results(data).await?);
            }
            return Ok(result);
        }

        // 3. Translate
        log::debug!("AIQL: Translating prompt for {:?}...", dialect);
        let plan = self.translator.translate(&scrubbed_prompt, schema, dialect, session).await?;
        log::debug!("AIQL: Generated query: {}", plan.raw_query);

        // 4. Dry Run
        log::debug!("AIQL: Performing dry-run validation...");
        if !self.engine.dry_run(&plan.raw_query).await? {
             log::warn!("AIQL: Dry run failed for generated query");
             return Err(anyhow::anyhow!("Dry run failed for generated query"));
        }

        // 5. Store in Cache if valid
        self.cache.set(&embedding, plan.clone()).await?;

        // 6. Execute
        log::debug!("AIQL: Executing query...");
        let mut result = self.engine.execute(&plan.raw_query).await?;

        // 7. Self-Healing Loop
        if !result.success {
            if let Some(error_msg) = result.error.clone() {
                log::warn!("AIQL: Query failed: {}. Attempting self-healing...", error_msg);
                
                // 8. Heal
                let healed_plan = self.healer.heal(&plan.raw_query, &error_msg, schema).await?;
                log::info!("AIQL: Healed query: {}", healed_plan.raw_query);
                
                // 9. Retry
                result = self.engine.execute(&healed_plan.raw_query).await?;
                if result.success {
                    log::info!("AIQL: Self-healing successful!");
                } else {
                    log::error!("AIQL: Self-healing failed: {:?}", result.error);
                }
            }
        }

        // 10. Privacy Masking on results
        if let Some(data) = result.data {
            result.data = Some(self.privacy.mask_results(data).await?);
        }

        log::info!("AIQL: Request completed in {}ms", result.execution_time_ms);
        Ok(result)
    }

    pub async fn vector_ask(&self, prompt: &str, schema: &Schema, dialect: crate::DatabaseDialect, session: Option<&crate::Session>) -> anyhow::Result<ExecutionResult> {
        log::info!("AIQL: Received vector prompt: '{}'", prompt);

        // 1. Translate with placeholder
        log::debug!("AIQL: Translating vector prompt...");
        let plan = self.translator.translate_vector(prompt, schema, dialect, session).await?;
        
        // 2. Generate embedding
        log::debug!("AIQL: Generating embedding...");
        let embedding = self.embedder.embed(prompt).await?;
        let vector_str = format!("[{}]", embedding.iter().map(|f| f.to_string()).collect::<Vec<String>>().join(","));

        // 3. Replace placeholder
        let final_query = plan.raw_query.replace("$VECTOR", &vector_str);
        
        // 4. Execute
        log::debug!("AIQL: Executing final vector query...");
        self.engine.execute(&final_query).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{QueryPlan, Translator, ExecutionEngine, QueryHealer, ExecutionResult};
    use async_trait::async_trait;
    use std::collections::HashMap;

    struct MockTranslator;
    #[async_trait]
    impl Translator for MockTranslator {
        async fn translate(&self, _prompt: &str, _schema: &Schema, dialect: crate::DatabaseDialect, _session: Option<&crate::Session>) -> anyhow::Result<QueryPlan> {
            Ok(QueryPlan {
                dialect,
                raw_query: "SELECT * FROM users;".to_string(),
                explanation: "Mock query".to_string(),
                cost: None,
            })
        }

        async fn translate_migration(&self, _prompt: &str, _schema: &Schema, dialect: crate::DatabaseDialect) -> anyhow::Result<crate::MigrationPlan> {
            Ok(crate::MigrationPlan {
                dialect,
                raw_sql: "".to_string(),
                explanation: "".to_string(),
            })
        }

        async fn translate_vector(&self, _prompt: &str, _schema: &Schema, dialect: crate::DatabaseDialect, _session: Option<&crate::Session>) -> anyhow::Result<QueryPlan> {
            Ok(QueryPlan {
                dialect,
                raw_query: "SELECT * FROM items ORDER BY embedding <=> '$VECTOR' LIMIT 1;".to_string(),
                explanation: "Vector query".to_string(),
                cost: None,
            })
        }
    }

    struct MockEngine {
        fail_first: bool,
    }
    #[async_trait]
    impl ExecutionEngine for MockEngine {
        async fn execute(&self, query: &str) -> anyhow::Result<ExecutionResult> {
            if query.contains("Healed") {
                return Ok(ExecutionResult {
                    success: true,
                    data: None,
                    error: None,
                    execution_time_ms: 10,
                    timestamp: chrono::Utc::now(),
                });
            }
            if self.fail_first {
                return Ok(ExecutionResult {
                    success: false,
                    data: None,
                    error: Some("Syntax error".to_string()),
                    execution_time_ms: 10,
                    timestamp: chrono::Utc::now(),
                });
            }
            Ok(ExecutionResult {
                success: true,
                data: None,
                error: None,
                execution_time_ms: 10,
                timestamp: chrono::Utc::now(),
            })
        }
        async fn dry_run(&self, _query: &str) -> anyhow::Result<bool> {
            Ok(true)
        }
    }

    struct MockHealer;
    #[async_trait]
    impl QueryHealer for MockHealer {
        async fn heal(&self, _query: &str, _error: &str, _schema: &Schema) -> anyhow::Result<QueryPlan> {
            Ok(QueryPlan {
                dialect: crate::DatabaseDialect::Postgres,
                raw_query: "Healed SELECT * FROM users;".to_string(),
                explanation: "Healed".to_string(),
                cost: None,
            })
        }
    }

    struct MockEmbedder;
    #[async_trait]
    impl EmbeddingEngine for MockEmbedder {
        async fn embed(&self, _text: &str) -> anyhow::Result<Vec<f32>> {
            Ok(vec![0.1, 0.2, 0.3])
        }
    }

    struct MockCache;
    #[async_trait]
    impl SemanticCache for MockCache {
        async fn get(&self, _embedding: &[f32]) -> anyhow::Result<Option<QueryPlan>> { Ok(None) }
        async fn set(&self, _embedding: &[f32], _plan: QueryPlan) -> anyhow::Result<()> { Ok(()) }
    }

    struct MockPrivacy;
    #[async_trait]
    impl PrivacyGuard for MockPrivacy {
        async fn scrub_prompt(&self, prompt: &str) -> anyhow::Result<String> { Ok(prompt.to_string()) }
        async fn mask_results(&self, data: serde_json::Value) -> anyhow::Result<serde_json::Value> { Ok(data) }
    }

    fn mock_schema() -> Schema {
        Schema {
            version: "1.0".to_string(),
            created_at: chrono::Utc::now(),
            tables: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_smart_client_success() {
        let client = SmartClient::new(MockTranslator, MockEngine { fail_first: false }, MockHealer, MockEmbedder, MockCache, MockPrivacy);
        let schema = mock_schema();
        let result = client.ask("prompt", &schema, crate::DatabaseDialect::Postgres, None).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_smart_client_healing() {
        let client = SmartClient::new(MockTranslator, MockEngine { fail_first: true }, MockHealer, MockEmbedder, MockCache, MockPrivacy);
        let schema = mock_schema();
        let result = client.ask("prompt", &schema, crate::DatabaseDialect::Postgres, None).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_smart_client_dry_run_failure() {
        struct FailingEngine;
        #[async_trait]
        impl ExecutionEngine for FailingEngine {
            async fn execute(&self, _query: &str) -> anyhow::Result<ExecutionResult> {
                Ok(ExecutionResult { success: true, data: None, error: None, execution_time_ms: 0, timestamp: chrono::Utc::now() })
            }
            async fn dry_run(&self, _query: &str) -> anyhow::Result<bool> {
                Ok(false)
            }
        }
        let client = SmartClient::new(MockTranslator, FailingEngine, MockHealer, MockEmbedder, MockCache, MockPrivacy);
        let schema = mock_schema();
        let result = client.ask("prompt", &schema, crate::DatabaseDialect::Postgres, None).await;
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), "Dry run failed for generated query");
    }

    #[tokio::test]
    async fn test_smart_client_vector_ask() {
        struct VectorTranslator;
        #[async_trait]
        impl Translator for VectorTranslator {
            async fn translate(&self, _p: &str, _s: &Schema, d: crate::DatabaseDialect, _sess: Option<&crate::Session>) -> anyhow::Result<QueryPlan> {
                Ok(QueryPlan { dialect: d, raw_query: "".to_string(), explanation: "".to_string(), cost: None })
            }
            async fn translate_migration(&self, _p: &str, _s: &Schema, d: crate::DatabaseDialect) -> anyhow::Result<crate::MigrationPlan> {
                Ok(crate::MigrationPlan { dialect: d, raw_sql: "".to_string(), explanation: "".to_string() })
            }
            async fn translate_vector(&self, _p: &str, _s: &Schema, d: crate::DatabaseDialect, _sess: Option<&crate::Session>) -> anyhow::Result<QueryPlan> {
                Ok(QueryPlan { 
                    dialect: d, 
                    raw_query: "SELECT * FROM items ORDER BY embedding <=> '$VECTOR' LIMIT 1;".to_string(), 
                    explanation: "Vector query".to_string(), 
                    cost: None 
                })
            }
        }

        let client = SmartClient::new(VectorTranslator, MockEngine { fail_first: false }, MockHealer, MockEmbedder, MockCache, MockPrivacy);
        let schema = mock_schema();
        let result = client.vector_ask("prompt", &schema, crate::DatabaseDialect::Postgres, None).await.unwrap();
        assert!(result.success);
    }
}
