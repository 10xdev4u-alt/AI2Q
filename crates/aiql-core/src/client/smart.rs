use crate::{ExecutionEngine, ExecutionResult, QueryHealer, Schema, Translator};

pub struct SmartClient<T, E, H>
where
    T: Translator,
    E: ExecutionEngine,
    H: QueryHealer,
{
    translator: T,
    engine: E,
    healer: H,
}

impl<T, E, H> SmartClient<T, E, H>
where
    T: Translator,
    E: ExecutionEngine,
    H: QueryHealer,
{
    pub fn new(translator: T, engine: E, healer: H) -> Self {
        Self {
            translator,
            engine,
            healer,
        }
    }

    pub async fn ask(&self, prompt: &str, schema: &Schema) -> anyhow::Result<ExecutionResult> {
        log::info!("AIQL: Received prompt: '{}'", prompt);

        // 1. Translate
        log::debug!("AIQL: Translating prompt...");
        let plan = self.translator.translate(prompt, schema).await?;
        log::debug!("AIQL: Generated query: {}", plan.raw_query);

        // 2. Dry Run
        log::debug!("AIQL: Performing dry-run validation...");
        if !self.engine.dry_run(&plan.raw_query).await? {
             log::warn!("AIQL: Dry run failed for generated query");
             return Err(anyhow::anyhow!("Dry run failed for generated query"));
        }

        // 3. Execute
        log::debug!("AIQL: Executing query...");
        let mut result = self.engine.execute(&plan.raw_query).await?;

        // 4. Self-Healing Loop
        if !result.success {
            if let Some(error_msg) = result.error.clone() {
                log::warn!("AIQL: Query failed: {}. Attempting self-healing...", error_msg);
                
                // 5. Heal
                let healed_plan = self.healer.heal(&plan.raw_query, &error_msg, schema).await?;
                log::info!("AIQL: Healed query: {}", healed_plan.raw_query);
                
                // 6. Retry
                result = self.engine.execute(&healed_plan.raw_query).await?;
                if result.success {
                    log::info!("AIQL: Self-healing successful!");
                } else {
                    log::error!("AIQL: Self-healing failed: {:?}", result.error);
                }
            }
        }

        log::info!("AIQL: Request completed in {}ms", result.execution_time_ms);
        Ok(result)
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
        async fn translate(&self, _prompt: &str, _schema: &Schema) -> anyhow::Result<QueryPlan> {
            Ok(QueryPlan {
                raw_query: "SELECT * FROM users;".to_string(),
                explanation: "Mock query".to_string(),
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
                raw_query: "Healed SELECT * FROM users;".to_string(),
                explanation: "Healed".to_string(),
                cost: None,
            })
        }
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
        let client = SmartClient::new(MockTranslator, MockEngine { fail_first: false }, MockHealer);
        let schema = mock_schema();
        let result = client.ask("prompt", &schema).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_smart_client_healing() {
        let client = SmartClient::new(MockTranslator, MockEngine { fail_first: true }, MockHealer);
        let schema = mock_schema();
        let result = client.ask("prompt", &schema).await.unwrap();
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
        let client = SmartClient::new(MockTranslator, FailingEngine, MockHealer);
        let schema = mock_schema();
        let result = client.ask("prompt", &schema).await;
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), "Dry run failed for generated query");
    }
}
