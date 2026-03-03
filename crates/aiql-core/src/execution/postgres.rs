use crate::{ExecutionEngine, ExecutionResult};
use async_trait::async_trait;
use sqlx::postgres::PgPool;
use std::time::Instant;

pub struct PostgresExecutionEngine {
    pool: PgPool,
}

impl PostgresExecutionEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ExecutionEngine for PostgresExecutionEngine {
    async fn execute(&self, query: &str) -> anyhow::Result<ExecutionResult> {
        let start = Instant::now();
        
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await;

        let execution_time_ms = start.elapsed().as_millis() as u64;

        match rows {
            Ok(_r) => {
                // TODO: Map sqlx rows to serde_json::Value
                Ok(ExecutionResult {
                    success: true,
                    data: None, // Simplified for now
                    error: None,
                    execution_time_ms,
                })
            }
            Err(e) => {
                Ok(ExecutionResult {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                    execution_time_ms,
                })
            }
        }
    }

    async fn dry_run(&self, query: &str) -> anyhow::Result<bool> {
        let explain_query = format!("EXPLAIN {}", query);
        let result = sqlx::query(&explain_query)
            .fetch_one(&self.pool)
            .await;

        Ok(result.is_ok())
    }
}
