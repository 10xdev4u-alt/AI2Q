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
        // 1. Translate
        let plan = self.translator.translate(prompt, schema).await?;

        // 2. Dry Run
        if !self.engine.dry_run(&plan.raw_query).await? {
             return Err(anyhow::anyhow!("Dry run failed for generated query"));
        }

        // 3. Execute
        let mut result = self.engine.execute(&plan.raw_query).await?;

        // 4. Self-Healing Loop
        if !result.success {
            if let Some(error_msg) = result.error.clone() {
                // 5. Heal
                let healed_plan = self.healer.heal(&plan.raw_query, &error_msg, schema).await?;
                
                // 6. Retry
                result = self.engine.execute(&healed_plan.raw_query).await?;
            }
        }

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
                });
            }
            if self.fail_first {
                return Ok(ExecutionResult {
                    success: false,
                    data: None,
                    error: Some("Syntax error".to_string()),
                    execution_time_ms: 10,
                });
            }
            Ok(ExecutionResult {
                success: true,
                data: None,
                error: None,
                execution_time_ms: 10,
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

    #[tokio::test]
    async fn test_smart_client_success() {
        let client = SmartClient::new(MockTranslator, MockEngine { fail_first: false }, MockHealer);
        let schema = Schema { tables: HashMap::new() };
        let result = client.ask("prompt", &schema).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_smart_client_dry_run_failure() {
        struct FailingEngine;
        #[async_trait]
        impl ExecutionEngine for FailingEngine {
            async fn execute(&self, _query: &str) -> anyhow::Result<ExecutionResult> {
                Ok(ExecutionResult { success: true, data: None, error: None, execution_time_ms: 0 })
            }
            async fn dry_run(&self, _query: &str) -> anyhow::Result<bool> {
                Ok(false)
            }
        }
        let client = SmartClient::new(MockTranslator, FailingEngine, MockHealer);
        let schema = Schema { tables: HashMap::new() };
        let result = client.ask("prompt", &schema).await;
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), "Dry run failed for generated query");
    }
}
