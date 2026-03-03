use crate::{ExecutionEngine, ExecutionResult, QueryHealer, Schema, Translator, EmbeddingEngine, SemanticCache, PrivacyGuard, AskResult, TranslateResult, Advisor};

pub struct SmartClient<T, E, H, V, C, P, A>
where
    T: Translator,
    E: ExecutionEngine,
    H: QueryHealer,
    V: EmbeddingEngine,
    C: SemanticCache,
    P: PrivacyGuard,
    A: Advisor,
{
    translator: T,
    engine: E,
    healer: H,
    embedder: V,
    cache: C,
    privacy: P,
    advisor: A,
}

impl<T, E, H, V, C, P, A> SmartClient<T, E, H, V, C, P, A>
where
    T: Translator,
    E: ExecutionEngine,
    H: QueryHealer,
    V: EmbeddingEngine,
    C: SemanticCache,
    P: PrivacyGuard,
    A: Advisor,
{
    pub fn new(translator: T, engine: E, healer: H, embedder: V, cache: C, privacy: P, advisor: A) -> Self {
        Self {
            translator,
            engine,
            healer,
            embedder,
            cache,
            privacy,
            advisor,
        }
    }

    pub async fn ask(&self, prompt: &str, schema: &Schema, dialect: crate::DatabaseDialect, session: Option<&crate::Session>, budget: Option<&crate::Budget>, policy: crate::SafetyPolicy) -> anyhow::Result<AskResult> {
        log::info!("AIQL: Received prompt: '{}'", prompt);

        let context = crate::Context { now: chrono::Utc::now() };

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
            return Ok(AskResult::Success(result));
        }

        // 3. Translate
        log::debug!("AIQL: Translating prompt for {:?}...", dialect);
        let translate_result = self.translator.translate(&scrubbed_prompt, schema, dialect, &context, session).await?;
        
        let plan = match translate_result {
            TranslateResult::ClarificationNeeded { reason, suggestions } => {
                return Ok(AskResult::ClarificationNeeded { reason, suggestions });
            }
            TranslateResult::Plan(p) => p,
        };

        let raw_query_with_explanation = if plan.dialect == crate::DatabaseDialect::MongoDB {
            plan.raw_query.clone() // Don't prepend comments to JSON pipelines
        } else {
            format!("-- {}\n{}", plan.explanation, plan.raw_query)
        };

        log::debug!("AIQL: Generated query: {}", raw_query_with_explanation);

        // 4. Safety Policy Check
        match policy {
            crate::SafetyPolicy::ReadOnly | crate::SafetyPolicy::Strict => {
                let destructive = ["DROP", "DELETE", "TRUNCATE", "ALTER", "GRANT", "REVOKE"];
                let upper_query = plan.raw_query.to_uppercase();
                for cmd in destructive {
                    if upper_query.contains(cmd) {
                        return Ok(AskResult::Error(format!("Destructive command '{}' is not allowed under policy {:?}", cmd, policy)));
                    }
                }
                if policy == crate::SafetyPolicy::Strict && !upper_query.contains("SELECT") {
                     return Ok(AskResult::Error("Only SELECT queries are allowed under Strict policy".to_string()));
                }
            }
            crate::SafetyPolicy::ReadWrite => {}
        }

        // 5. Dry Run
        log::debug!("AIQL: Performing dry-run validation...");
        if !self.engine.dry_run(&raw_query_with_explanation).await? {
             log::warn!("AIQL: Dry run failed for generated query");
             return Ok(AskResult::Error("Dry run failed for generated query".to_string()));
        }

        // 5. Budget Check
        if let Some(b) = budget {
            if let (Some(plan_cost), Some(max_cost)) = (plan.cost, b.max_cost) {
                if plan_cost > max_cost {
                    return Ok(AskResult::Error(format!("Query cost {} exceeds budget {}", plan_cost, max_cost)));
                }
            }
        }

        // 6. Store in Cache if valid
        self.cache.set(&embedding, plan.clone()).await?;

        // 7. Execute
        log::debug!("AIQL: Executing query...");
        let mut result = self.engine.execute(&raw_query_with_explanation).await?;

        // 8. Self-Healing Loop
        if !result.success {
            if let Some(error_msg) = result.error.clone() {
                log::warn!("AIQL: Query failed: {}. Attempting self-healing...", error_msg);
                
                // 9. Heal
                let healed_plan = self.healer.heal(&raw_query_with_explanation, &error_msg, schema, dialect, &context).await?;
                log::info!("AIQL: Healed query: {}", healed_plan.raw_query);
                
                // 10. Retry
                result = self.engine.execute(&healed_plan.raw_query).await?;
                if result.success {
                    log::info!("AIQL: Self-healing successful!");
                } else {
                    log::error!("AIQL: Self-healing failed: {:?}", result.error);
                }
            }
        }

        // 11. Privacy Masking on results
        if let Some(data) = result.data {
            result.data = Some(self.privacy.mask_results(data).await?);
        }

        // 12. Execution Time Check & Advice
        if let Some(b) = budget {
            if let Some(max_time) = b.max_execution_time_ms {
                if result.execution_time_ms > max_time {
                    log::warn!("AIQL: Execution time {}ms exceeds budget {}ms. Fetching advice...", result.execution_time_ms, max_time);
                    if let Ok(advice) = self.advisor.advise(&plan, schema).await {
                        for a in advice {
                            log::info!("AIQL ADVICE: {}", a);
                        }
                    }
                }
            }
        }

        log::info!("AIQL: Request completed in {}ms", result.execution_time_ms);
        Ok(AskResult::Success(result))
    }

    pub async fn vector_ask(&self, prompt: &str, schema: &Schema, dialect: crate::DatabaseDialect, session: Option<&crate::Session>, budget: Option<&crate::Budget>, policy: crate::SafetyPolicy) -> anyhow::Result<AskResult> {
        log::info!("AIQL: Received vector prompt: '{}'", prompt);

        let context = crate::Context { now: chrono::Utc::now() };

        // 1. Translate with placeholder
        log::debug!("AIQL: Translating vector prompt...");
        let translate_result = self.translator.translate_vector(prompt, schema, dialect, &context, session).await?;
        
        let plan = match translate_result {
            TranslateResult::ClarificationNeeded { reason, suggestions } => {
                return Ok(AskResult::ClarificationNeeded { reason, suggestions });
            }
            TranslateResult::Plan(p) => p,
        };
        
        // 2. Generate embedding
        log::debug!("AIQL: Generating embedding...");
        let embedding = self.embedder.embed(prompt).await?;
        let vector_str = format!("[{}]", embedding.iter().map(|f| f.to_string()).collect::<Vec<String>>().join(","));

        // 3. Replace placeholder
        let final_query = plan.raw_query.replace("$VECTOR", &vector_str);
        
        let final_query_with_explanation = if plan.dialect == crate::DatabaseDialect::MongoDB {
            final_query
        } else {
            format!("-- {}\n{}", plan.explanation, final_query)
        };

        // 4. Execute
        log::debug!("AIQL: Executing final vector query...");
        let mut result = self.engine.execute(&final_query_with_explanation).await?;

        // Privacy Masking on results
        if let Some(data) = result.data {
            result.data = Some(self.privacy.mask_results(data).await?);
        }

        Ok(AskResult::Success(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{QueryPlan, Translator, ExecutionEngine, QueryHealer, ExecutionResult, TranslateResult};
    use async_trait::async_trait;
    use std::collections::HashMap;

    struct MockTranslator;
    #[async_trait]
    impl Translator for MockTranslator {
        async fn translate(&self, _prompt: &str, _schema: &Schema, dialect: crate::DatabaseDialect, _context: &crate::Context, _session: Option<&crate::Session>) -> anyhow::Result<TranslateResult> {
            Ok(TranslateResult::Plan(QueryPlan {
                dialect,
                raw_query: "SELECT * FROM users;".to_string(),
                explanation: "Mock query".to_string(),
                cost: None,
            }))
        }

        async fn translate_migration(&self, _prompt: &str, _schema: &Schema, dialect: crate::DatabaseDialect) -> anyhow::Result<crate::MigrationPlan> {
            Ok(crate::MigrationPlan {
                dialect,
                raw_sql: "".to_string(),
                explanation: "".to_string(),
            })
        }

        async fn translate_vector(&self, _prompt: &str, _schema: &Schema, dialect: crate::DatabaseDialect, _context: &crate::Context, _session: Option<&crate::Session>) -> anyhow::Result<TranslateResult> {
            Ok(TranslateResult::Plan(QueryPlan {
                dialect,
                raw_query: "SELECT * FROM items ORDER BY embedding <=> '$VECTOR' LIMIT 1;".to_string(),
                explanation: "Vector query".to_string(),
                cost: None,
            }))
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
        async fn heal(&self, _query: &str, _error: &str, _schema: &Schema, dialect: crate::DatabaseDialect, _context: &crate::Context) -> anyhow::Result<QueryPlan> {
            Ok(QueryPlan {
                dialect,
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

    struct MockAdvisor;
    #[async_trait]
    impl Advisor for MockAdvisor {
        async fn advise(&self, _plan: &QueryPlan, _schema: &Schema) -> anyhow::Result<Vec<String>> { Ok(vec!["Add index on id".to_string()]) }
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
        let client = SmartClient::new(MockTranslator, MockEngine { fail_first: false }, MockHealer, MockEmbedder, MockCache, MockPrivacy, MockAdvisor);
        let schema = mock_schema();
        let result = client.ask("prompt", &schema, crate::DatabaseDialect::Postgres, None, None, crate::SafetyPolicy::ReadWrite).await.unwrap();
        match result {
            AskResult::Success(_) => {},
            _ => panic!("Expected Success"),
        }
    }

    #[tokio::test]
    async fn test_smart_client_healing() {
        let client = SmartClient::new(MockTranslator, MockEngine { fail_first: true }, MockHealer, MockEmbedder, MockCache, MockPrivacy, MockAdvisor);
        let schema = mock_schema();
        let result = client.ask("prompt", &schema, crate::DatabaseDialect::Postgres, None, None, crate::SafetyPolicy::ReadWrite).await.unwrap();
        match result {
            AskResult::Success(_) => {},
            _ => panic!("Expected Success"),
        }
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
        let client = SmartClient::new(MockTranslator, FailingEngine, MockHealer, MockEmbedder, MockCache, MockPrivacy, MockAdvisor);
        let schema = mock_schema();
        let result = client.ask("prompt", &schema, crate::DatabaseDialect::Postgres, None, None, crate::SafetyPolicy::ReadWrite).await.unwrap();
        match result {
            AskResult::Error(e) => assert_eq!(e, "Dry run failed for generated query"),
            _ => panic!("Expected Error"),
        }
    }

    #[tokio::test]
    async fn test_smart_client_vector_ask() {
        struct VectorTranslator;
        #[async_trait]
        impl Translator for VectorTranslator {
            async fn translate(&self, _p: &str, _s: &Schema, d: crate::DatabaseDialect, _c: &crate::Context, _sess: Option<&crate::Session>) -> anyhow::Result<TranslateResult> {
                Ok(TranslateResult::Plan(QueryPlan { dialect: d, raw_query: "".to_string(), explanation: "".to_string(), cost: None }))
            }
            async fn translate_migration(&self, _p: &str, _s: &Schema, d: crate::DatabaseDialect) -> anyhow::Result<crate::MigrationPlan> {
                Ok(crate::MigrationPlan { dialect: d, raw_sql: "".to_string(), explanation: "".to_string() })
            }
            async fn translate_vector(&self, _p: &str, _s: &Schema, d: crate::DatabaseDialect, _c: &crate::Context, _sess: Option<&crate::Session>) -> anyhow::Result<TranslateResult> {
                Ok(TranslateResult::Plan(QueryPlan { 
                    dialect: d, 
                    raw_query: "SELECT * FROM items ORDER BY embedding <=> '$VECTOR' LIMIT 1;".to_string(), 
                    explanation: "Vector query".to_string(), 
                    cost: None 
                }))
            }
        }

        let client = SmartClient::new(VectorTranslator, MockEngine { fail_first: false }, MockHealer, MockEmbedder, MockCache, MockPrivacy, MockAdvisor);
        let schema = mock_schema();
        let result = client.vector_ask("prompt", &schema, crate::DatabaseDialect::Postgres, None, None, crate::SafetyPolicy::ReadWrite).await.unwrap();
        match result {
            AskResult::Success(_) => {},
            _ => panic!("Expected Success"),
        }
    }
}
